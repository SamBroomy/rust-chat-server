mod connection;
mod error;
mod frame;

use bytes::BytesMut;
pub use error::{Error, Result};
pub use frame::Frame;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    // New tcp listener bound to localhost:8080
    let listener = TcpListener::bind("localhost:8080").await.unwrap();

    println!("Listening on: {}", listener.local_addr().unwrap());

    let (tx, _rx) = broadcast::channel(10);
    // This accepts incoming connections.
    // If you run two instances of this program, the second one will block here until the first connection is closed as the port is already in use by the first instance.
    // To avoid this issue you can put the code below in a loop and spawn a new task for each incoming connection.
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("Accepted connection from: {}", addr);
        // Clone the sender & receiver so that we can send it to the task that will handle the connection.
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        // Spawn a new task to handle the connection. The issue here is that each connection is isolated and can't communicate with each other.
        tokio::spawn(async move {
            // Split the socket into a reader and a writer so that we can read and write to the socket concurrently.
            let (reader, writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            println!("Split socket into reader and writer");

            let welcome = "Welcome to the chat server!\n".to_string();

            writer.write_all(welcome.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
            println!("Wrote welcome message");
            //Its kinda silly the server requesting a username and then the client sending the username back to the server.
            // But this is just a test to see if we can send and receive frames.
            // This type of thing will be handled when starting the client and is information sent to the server on connection.
            writer.write_all(b"Pick a username: ").await.unwrap();
            writer.flush().await.unwrap();
            println!("Wrote username prompt");

            let mut buff = BytesMut::with_capacity(1024 * 4);

            reader.read_buf(&mut buff).await.unwrap();

            println!("{:?}", buff);

            let frame = Frame::deserialize(&buff.freeze()).unwrap();

            println!("{:?}", frame);

            println!("Read FRAME!!!!");

            return;

            // Wrap the reader in a BufReader so that we can read line by line.
            //let mut reader = BufReader::new(reader);
            // Store the line read from the socket.
            let mut line = String::new();

            // Enter a loop to read from the socket and write back to it.
            // If there was no loop here, the program would read only once and then exit.
            loop {
                // The tokio select will wait for both futures to complete and then execute whichever one completed first.
                // This is useful because we want to read from the socket and also get messages from other clients.
                tokio::select! {
                    // Read a line from the socket.
                    result = reader.read_line(&mut line) => {
                        // If no bytes were read, it means the other side has closed the connection.
                        if result.unwrap() == 0 {
                            break;
                        }
                        println!("Received: {}", line.trim());
                        // Broadcast the received line to all connected clients.
                        tx.send((addr,line.clone())).unwrap();
                        // Clear the line so that we can read the next line.
                        line.clear();
                    }
                    // Get the message from other clients.
                    result = rx.recv() => {
                        let (sender_addr, msg) = result.unwrap();
                        // Don't send the message back to the client that sent it.
                        if addr != sender_addr {
                            // Write the line back to the socket.
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
