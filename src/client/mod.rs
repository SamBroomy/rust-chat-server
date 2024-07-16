mod error;

use crate::connection::ConnectionError;
use crate::{ClientMessage, Connection, FrameType, ServerMessage, User};
pub use error::ClientError;
use error::Result;

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
        ClientMessage::Handshake(user.clone())
            .write_frame_to(&mut writer)
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
            let _user = user.clone();
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
                        let line = line.trim();

                        if line.is_empty() {
                            continue;
                        }
                        let frame = match parse_user_input(line) {
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
            None => return Err(ConnectionError::ConnectionDropped.into()),
        };

        let (mut reader, mut writer) = connection.split_into();

        loop {
            tokio::select! {
                Some(frame) = input_receiver.recv() =>{
                    if let ClientMessage::Ping(nonce) = frame {
                        info!("Sending ping frame");
                        frame.write_frame_to(&mut writer).await?;
                        let server_frame = ServerMessage::read_frame_from(&mut reader).await?;

                        let n = match server_frame {
                            ServerMessage::Pong(n) => {
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


                    if frame.write_frame_to(&mut writer).await.is_err() {
                        eprintln!("Failed to send frame");
                        break;
                    }

                }
                // Log messages to the console here.
                frame = ServerMessage::read_frame_from(&mut reader) => {
                    match frame {
                        Ok(frame) => {
                            handle_and_print_frame(frame)?
                        }
                        Err(e) => {
                            match e {
                                ConnectionError::ConnectionDropped => {
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

fn parse_user_input(input: impl Into<String>) -> Option<ClientMessage> {
    let line: String = input.into();
    println!("{:<10}: {}", "You".blue(), line);

    if line.starts_with(':') {
        match line.split(' ').next().unwrap().to_lowercase().as_str() {
            ":quit" => {
                info!("Sending quit frame");
                Some(ClientMessage::Disconnect)
            }
            ":ping" => {
                info!("Sending ping frame");
                let frame = ClientMessage::Ping(rand::random());
                println!("{}", frame.to_string().blue());
                Some(frame)
            }
            ":pm" => {
                let mut parts = line.splitn(3, ' ');
                parts.next();
                let user = parts.next().unwrap_or_default();
                info!("Sending private message to {}", user);
                let content = parts.next().unwrap_or_default();
                info!("Message: {}", content);
                Some(ClientMessage::PrivateMessage {
                    to_user: user.into(),
                    content: content.to_string(),
                })
            }
            _ => {
                warn!("Invalid command: {}.", line);
                println!("List of valid commands: <:quit>, <:ping>");
                None
            }
        }
    } else {
        info!("Sending chat message frame");
        Some(ClientMessage::ChatMessage { content: line })
    }
}

enum Commands {
    Quit,
    Ping,
}

impl TryFrom<&str> for Commands {
    type Error = ClientError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            ":quit" => Ok(Self::Quit),
            ":ping" => Ok(Self::Ping),
            _ => Err(ClientError::InvalidCommand),
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
