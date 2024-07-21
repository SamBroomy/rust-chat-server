use super::{Result, ServerError};
use crate::common::{
    messages::{
        ClientMessage, Handshake, ProcessInternal, ProcessMessage, ServerInternal, ServerMessage,
        UserInternal, UserMessage,
    },
    UserName,
};
use crate::connection::{Connection, FrameType};

use crossterm::style::Stylize;
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc, oneshot},
};
use tracing::{debug, error, info, warn};

/// Handles the client connection, reading and writing messages to the stream.
pub struct ClientHandler {
    user: UserName,
    connection: Connection,
    client_rx: mpsc::Receiver<ServerMessage>,
    server_command_tx: mpsc::Sender<ProcessMessage>,
    server_broadcast_rx: broadcast::Receiver<ServerMessage>,
}

impl ClientHandler {
    pub async fn init(
        connection: TcpStream,
        server_broadcast_rx: broadcast::Receiver<ServerMessage>,
        mut server_command_tx: mpsc::Sender<ProcessMessage>,
    ) -> Result<Self> {
        let mut connection = Connection::from_stream(connection);

        let (user, client_rx) = Self::authenticate(&mut connection, &mut server_command_tx).await?;

        Ok(Self {
            user,
            connection,
            client_rx,
            server_command_tx,
            server_broadcast_rx,
        })
    }

    // TODO: ewww clean this up
    async fn authenticate(
        connection: &mut Connection,
        server_command_tx: &mut mpsc::Sender<ProcessMessage>,
    ) -> Result<(UserName, mpsc::Receiver<ServerMessage>)> {
        debug!("Waiting for handshake frame");
        let user = match connection.read_frame().await {
            Ok(Handshake(user)) => user,
            _ => {
                error!("Expected Handshake frame");
                return Err(ServerError::InvalidHandshake);
            }
        };

        debug!("Handshake received: {}", user);

        debug!("Add user to server");
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        // Send the user to the server processor
        server_command_tx
            .send(ProcessMessage::Internal(ProcessInternal::UserMessage(
                UserMessage {
                    from_user: user.clone(),
                    message: UserInternal::NewUser(oneshot_tx),
                },
            )))
            .await?;

        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(5)) => {
                error!("Handshake timeout");
            }
            Ok(client_rx_result) = oneshot_rx => {
                match client_rx_result {
                    Ok(client_rx) => {

                        debug!("User added to server");
                        // Send back a response to the client
                        connection
                            .write_frame(&ServerInternal::ServerMessage(format!(
                            "Welcome, {}!",
                            user.to_string().green()
                        ))).await?;
                        debug!("Handshake complete");
                        return Ok((user, client_rx))
                    }
                    Err(e) => {
                        error!("Handshake failed: {}", e);
                    }
                }
            }
        }
        connection
            .write_frame(&ServerInternal::ServerMessage(
                "Handshake timeout".to_string(),
            ))
            .await?;
        Err(ServerError::HandshakeTimeout)
    }

    pub async fn run(&mut self) -> Result<()> {
        let (mut reader, mut writer) = self.connection.split();

        loop {
            tokio::select! {
            frame = ClientMessage::read_frame_from(&mut reader) => {
                match frame {
                    Ok(frame) => {
                        let message = ProcessMessage::ClientMessage {
                            from_user: self.user.clone(),
                            message: frame,
                        };
                        self.server_command_tx.send(message).await?;}
                    Err(e) => {
                        error!("Error reading frame: {}", e);
                        self.server_command_tx.send(ProcessMessage::Internal(
                            ProcessInternal::UserMessage(UserMessage {
                                from_user: self.user.clone(),
                                message: UserInternal::DisconnectUser,
                            }),
                        )).await?;
                        break;
                    }
            }},

            Ok(ServerMessage { from_user, content }) = self.server_broadcast_rx.recv() => {
                if self.user != from_user {
                    info!("Sending from server_broadcast_rx");
                    content.write_frame_to(&mut writer).await?;
                }
            },

            Some(ServerMessage { from_user, content }) = self.client_rx.recv() => {
                if self.user == from_user {
                    debug!("Message from self");
                }
                info!("Sending from client_rx send user: {} current user: {}", from_user, self.user);
                content.write_frame_to(&mut writer).await?;
            },
            else => break




            }
        }

        warn!(
            "Connection closed for {} due to breaking connection loop",
            self.user
        );

        Ok(())
    }
}
