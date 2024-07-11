# rust-chat-server

Understanding async rust better by implementing a chat-server from scratch

## How to run

Run each of the following commands in a separate terminal window.

`cargo run` - Run the server

`telnet localhost 8080` - Connect to the server (run this command several times in different terminal windows to simulate multiple clients)

## Branch 1 - Echo server

Currently implemented and heavily commented is an echo server.

Will slowly remove the comments but for now, it's a good reference for understanding the code.

## Branch 2 - Chat server

Now implemented a chat server that will be able to handle multiple clients, and broadcast messages to all connected clients.

## References

For the initial implementation, I followed the tutorial [Lily Mara -Creating a Chat Server with async Rust and Tokio](https://www.youtube.com/watch?v=T2mWg91sx-o).

Once I get past the basic implementation, I will be implementing more features and making it more robust.
