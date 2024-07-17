use chat_app::{init, Client, Result};
use tracing::Level;

fn get_username() -> Result<String> {
    println!("Enter your username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    let username = username.trim();
    if username.is_empty() {
        println!("Username cannot be empty, please try again");
        get_username()
    } else {
        Ok(username.trim().to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let address = init(None);
    let username = get_username()?;

    let client = Client::new(username).await;

    Ok(client.run(address).await?)
}
