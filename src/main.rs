use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    // New tcp listener bound to localhost:8080
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    // This accepts incoming connections.
    // If you run two instances of this program, the second one will block here until the first connection is closed as the port is already in use by the first instance.
    let (mut socket, _addr) = listener.accept().await.unwrap();
    // Split the socket into a reader and a writer so that we can read and write to the socket concurrently.
    let (reader, mut writer) = socket.split();
    // Wrap the reader in a BufReader so that we can read line by line.
    let mut reader = BufReader::new(reader);
    // Store the line read from the socket.
    let mut line = String::new();

    // Enter a loop to read from the socket and write back to it.
    // If there was no loop here, the program would read only once and then exit.
    loop {
        // Read a line from the socket.
        let bytes_read = reader.read_line(&mut line).await.unwrap();
        // If no bytes were read, it means the other side has closed the connection.
        if bytes_read == 0 {
            break;
        }
        println!("Received: {}", line.trim());
        // Write the line back to the socket.
        writer.write_all(line.as_bytes()).await.unwrap();
        // Clear the line so that we can read the next line.
        line.clear();
    }
}
