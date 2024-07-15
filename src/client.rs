use crate::{ClientFrame, Connection, Error, FrameType, Result, ServerFrame, User};
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::execute;
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType};
use std::process::exit;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub struct Client {
    connection: Option<Connection>,
    user: User,
}

impl Client {
    pub async fn new(addr: impl ToSocketAddrs, user: impl Into<User>) -> Result<Self> {
        let mut stream = TcpStream::connect(addr).await?;
        info!("Connected to server {}", stream.peer_addr()?);

        let user = user.into();

        let (_, mut writer) = Connection::split(&mut stream);

        info!("Sending handshake frame");
        ClientFrame::Handshake(user.clone())
            .write_to(&mut writer)
            .await?;
        info!("Handshake sent");

        Ok(Self {
            connection: Some(Connection::new(stream)),
            user,
        })
    }

    pub async fn run(self) -> Result<()> {
        let (input_sender, mut input_receiver) = mpsc::channel(16);

        let user = self.user.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(tokio::io::stdin());
            let mut lines = reader.lines();
            let user = user.clone();
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

                        if line.trim().is_empty() {
                            continue;
                        }
                        let frame = match parse_user_input(&line) {
                            Some(frame) => frame,
                            None => continue,
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
                Some(frame) = input_receiver.recv() =>{
                    if let ClientFrame::Ping(nonce) = frame {
                        info!("Sending ping frame");
                        frame.write_to(&mut writer).await?;
                        let server_frame = ServerFrame::read_from(&mut reader).await?;

                        let n = match server_frame {
                            ServerFrame::Pong(n) => {
                                println!("{}", server_frame.to_string().yellow());
                                info!("Received pong frame: {}", n);
                                n
                            }
                            _ => {
                                error!("Received invalid frame: {:?}", server_frame);
                                continue;
                            }
                        };

                        assert_eq!(nonce, n);
                        println!("{}", "Ping-pong successful".green());

                        continue;
                    }


                    if frame.write_to(&mut writer).await.is_err() {
                        eprintln!("Failed to send frame");
                        break;
                    }

                }
                // Log messages to the console here.
                frame = ServerFrame::read_from(&mut reader) => {
                    match frame {
                        Ok(frame) => {
                            handle_and_print_frame(frame)?
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
                else => {
                    error!("Connection dropped");
                    break;
                }

            }
        }
        Ok(())
    }
}

fn parse_user_input(input: impl Into<String>) -> Option<ClientFrame> {
    let line = input.into();
    println!("{:<10}: {}", "You".blue(), line);

    if line.starts_with(':') {
        match line.as_str() {
            ":" => {
                warn!("Empty command.");
                println!("List of valid commands: <:quit>, <:ping>");
                None
            }
            ":quit" => {
                info!("Sending quit frame");
                Some(ClientFrame::Disconnect)
            }
            ":ping" => {
                info!("Sending ping frame");
                let frame = ClientFrame::Ping(rand::random());
                println!("{}", frame.to_string().blue());
                Some(frame)
            }
            _ => {
                warn!("Invalid command: {}.", line);
                println!("List of valid commands: <:quit>, <:ping>");
                None
            }
        }
    } else {
        info!("Sending chat message frame");
        Some(ClientFrame::ChatMessage { content: line })
    }
}

enum Commands {
    Quit,
    Ping,
}

impl TryFrom<&str> for Commands {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            ":quit" => Ok(Self::Quit),
            ":ping" => Ok(Self::Ping),
            _ => Err(Error::InvalidCommand),
        }
    }
}

// impl From<Commands> for Frame {
//     fn from(command: Commands) -> Self {
//         match command {
//             Commands::Quit => Frame::Disconnect,
//             Commands::Ping => Frame::Ping(rand::random()),
//         }
//     }
// }

fn handle_and_print_frame<F: FrameType>(frame: F) -> Result<()> {
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
