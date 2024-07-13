use chat_app::Frame;

use bytes::BytesMut;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let socket = TcpStream::connect("localhost:8080").await?;

    let (reader, mut writer) = socket.into_split();

    let mut reader = tokio::io::BufReader::new(reader);
    let mut writer = tokio::io::BufWriter::new(writer);

    let mut buf = BytesMut::with_capacity(1024 * 4);
    let mut line = String::new();

    reader.read_line(&mut line).await?;

    println!("{:?}", line);

    reader.read_buf(&mut buf).await?;

    println!("{:?}", buf);

    //Take user input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let frame = Frame::Text {
        sender: "Me".to_string(),
        content: input.trim().to_string(),
    };

    let s = frame.serialize();

    println!("{:?}", s);

    writer.write_all(&s).await?;
    writer.flush().await?;

    println!("Wrote frame to server");

    Ok(())
}
