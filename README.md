# rust-chat-server

Understanding async rust better by implementing a chat-server from scratch

## Current features

- Global Server chat
- Chat rooms (soon)
- 'Private'/Direct Messaging
- Global Notifications
- 'lose' authentication (want to intergrate some form of encryption, maybe even encrypted messages). 

Todo: Move the request processing to the server thread rather than (how it currently is implemented) by the connection handler.

## Will update the below soon!!!

## How to run

Run each of the following commands in a separate terminal window.

`cargo run` or `just run` - Run the server

`just client` - Connect to the server (run this command several times in different terminal windows to simulate multiple clients)

## Branch 1 - Echo server

Currently implemented and heavily commented is an echo server.

Will slowly remove the comments but for now, it's a good reference for understanding the code.

## Branch 2 - Chat server

Now implemented a chat server that will be able to handle multiple clients, and broadcast messages to all connected clients.

## Branch 3 - Send & Receive Frames

Its starting to get a bit messy but will clean things up later. Now what we have done is to send and receive frames. Essentially, deserializing some data (in this case an Enum) to bytes, sending the bytes from the client to the server and then serializing the bytes back to the original data.

run the server with `cargo run --bin rust-chat-server`

run the client `cargo run --bin client`

## References

For the initial implementation, I followed the tutorial [Lily Mara -Creating a Chat Server with async Rust and Tokio](https://www.youtube.com/watch?v=T2mWg91sx-o).

Once I get past the basic implementation, I will be implementing more features and making it more robust.
