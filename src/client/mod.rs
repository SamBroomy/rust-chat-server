mod error;

use crate::common::messages::{ClientMessage, Handshake, ServerInternal};
use crate::common::UserName;
use crate::connection::{Connection, ConnectionError, FrameType, OwnedReader, OwnedWriter};
pub use error::ClientError;
use error::Result;

use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::execute;
use crossterm::style::Stylize;
use crossterm::terminal::{Clear, ClearType};
use std::process::exit;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::ToSocketAddrs;
use tokio::sync::mpsc;
use tracing::{error, info, instrument, warn};

pub struct Client {
    user: UserName,
}

impl Client {
    pub async fn new(user: impl Into<UserName>) -> Self {
        Self { user: user.into() }
    }

    async fn authenticate(&self, connection: &mut Connection) -> Result<()> {
        connection
            .write_frame(&Handshake(self.user.clone()))
            .await?;
        Ok(())
    }

    #[instrument(skip(input_sender), level = "debug")]
    async fn handle_user_input(input_sender: mpsc::Sender<ClientMessage>) {
        let reader = BufReader::new(tokio::io::stdin());
        let mut lines = reader.lines();
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
    }

    #[instrument(skip_all, level = "debug")]
    pub async fn run(self, addr: impl ToSocketAddrs) -> Result<()> {
        let mut connection = Connection::init(addr).await?;

        self.authenticate(&mut connection).await?;

        let (input_sender, mut input_receiver) = mpsc::channel(16);

        tokio::spawn(async move {
            Self::handle_user_input(input_sender).await;
        });
        let (mut reader, mut writer) = connection.split_into();

        loop {
            tokio::select! {
                Some(frame) = input_receiver.recv() =>{
                    Self::process_frame(frame, &mut reader, &mut writer).await?;
                }
                frame = ServerInternal::read_frame_from(&mut reader) => {
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

    #[instrument(skip(reader, writer), level = "debug")]
    async fn process_frame(
        frame: ClientMessage,
        reader: &mut OwnedReader,
        writer: &mut OwnedWriter,
    ) -> Result<()> {
        match frame {
            ClientMessage::Ping(nonce) => {
                info!("Sending ping frame");
                frame.write_frame_to(writer).await?;
                let server_frame = ServerInternal::read_frame_from(reader).await?;
                let n = match server_frame {
                    ServerInternal::Pong(n) => {
                        println!("{}", server_frame.to_string().yellow());
                        info!("Received pong frame: {}", n);
                        n
                    }
                    _ => {
                        error!("Received invalid frame: {:?}", server_frame);
                        return Ok(());
                    }
                };
                assert_eq!(nonce, n);
                println!("{}", "Ping-pong successful".green());
            }
            ClientMessage::Disconnect => {
                info!("Sending disconnect frame");
                frame.write_frame_to(writer).await?;
                return Err(ConnectionError::ConnectionDropped.into());
            }
            _ => {
                frame.write_frame_to(writer).await?;
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
            ":quit" => Some(ClientMessage::Disconnect),
            // ":create" => {
            //     let mut parts = line.splitn(2, ' ');
            //     parts.next();
            //     let room = parts.next().unwrap_or_default();
            //     info!("Creating room: {}", room);
            //     Some(ClientMessage::CreateRoom(room.into()))
            // }
            // ":join" => {
            //     let mut parts = line.splitn(2, ' ');
            //     parts.next();
            //     let room = parts.next().unwrap_or_default();
            //     info!("Joining room: {}", room);
            //     Some(ClientMessage::Join(room.into()))
            // }
            // ":room" => {
            //     let mut parts = line.splitn(3, ' ');
            //     parts.next();
            //     let room = parts.next().unwrap_or_default();
            //     let content = parts.next().unwrap_or_default();
            //     info!("Sending room message to {}", room);
            //     info!("Message: {}", content);
            //     Some(ClientMessage::RoomMessage {
            //         room: room.into(),
            //         content: content.to_string(),
            //     })
            // }
            // ":rooms" => {
            //     info!("Requesting list of rooms");
            //     Some(ClientMessage::ListRooms)
            // }
            ":users" => Some(ClientMessage::ListUsers),
            ":ping" => {
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
                println!("List of valid commands: :quit, :ping, :pm, :users");
                None
            }
        }
    } else {
        info!("Sending chat message frame");
        Some(ClientMessage::GlobalChatMessage(line))
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
