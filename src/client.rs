use crate::{Connection, Error, Frame, FrameType, Result};
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::execute;
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType};
use std::process::exit;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

pub struct Client {
    connection: Option<Connection>,
    username: String,
}

impl Client {
    pub async fn new(addr: impl ToSocketAddrs, username: impl Into<String>) -> Result<Self> {
        let mut stream = TcpStream::connect(addr).await?;
        debug!("Connected to server {}", stream.peer_addr()?);

        let username = username.into();

        let (_, mut writer) = Connection::split(&mut stream);

        info!("Sending handshake frame");
        Frame::Handshake {
            username: username.clone(),
        }
        .write_to(&mut writer)
        .await?;
        info!("Handshake sent");

        Ok(Self {
            connection: Some(Connection::new(stream)),
            username,
        })
    }

    pub async fn run(self) -> Result<()> {
        let (input_sender, mut input_receiver) = mpsc::channel(16);

        let username = self.username.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(tokio::io::stdin());
            let mut lines = reader.lines();
            let username = username.clone();
            while let Ok(line) = lines.next_line().await {
                match line {
                    Some(line) => {
                        execute!(
                            std::io::stdout(),
                            MoveUp(1),
                            MoveToColumn(0),
                            Clear(ClearType::CurrentLine)
                        )
                        .unwrap();
                        println!("{}  {}", "You:".blue(), line);
                        let frame = if line.starts_with(":ping") {
                            info!("Sending ping frame");
                            Frame::Ping(rand::random())
                        } else {
                            info!("Sending chat message frame");
                            Frame::ChatMessage {
                                username: username.clone(),
                                content: line,
                            }
                        };
                        if let Err(e) = input_sender.send(frame).await {
                            error!("Failed to send frame: {:?}", e);
                            break;
                        }
                    }
                    None => {
                        error!("Failed to read line");
                        break;
                    }
                }
            }
        });

        let connection = match self.connection {
            Some(connection) => connection,
            None => return Err(Error::ConnectionDropped),
        };

        let (mut reader, mut writer) = connection.split_into();

        loop {
            tokio::select! {
                Some(frame) = input_receiver.recv() => {
                    if frame.write_to(&mut writer).await.is_err() {
                        eprintln!("Failed to send frame");
                        break;
                    }

                }
                // Log messages to the console here.
                frame = Frame::read_from(&mut reader) => {
                    match frame {
                        Ok(frame) => {
                            handle_and_print_frame(frame)?;
                        }
                        Err(e) => {
                            match e {
                                Error::ConnectionDropped => {
                                    exit(0);
                                }
                                _ => {
                                    error!("Failed to read frame: {:?}", e);
                                    break;
                                }
                            }

                        }

                    }

                }

            }
        }
        Ok(())
    }
}

fn handle_and_print_frame(frame: Frame) -> Result<()> {
    println!("{}", frame.to_string().red());
    // execute!(
    //     std::io::stdout(),
    //     SetForegroundColor(Color::Blue),
    //     SetBackgroundColor(Color::Red),
    //     Print(frame.to_string()),
    //     ResetColor
    // )?;

    // match frame {
    //     todo!()
    //         error!("Received invalid frame: {:?}", frame);
    //     }
    // }

    Ok(())
}
