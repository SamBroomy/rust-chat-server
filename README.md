# Rust Chat Server

Understanding rust, tokio, threads, channels & async better by implementing a multi-client chat server from scratch using Rust and Tokio.

## Current Features

- Multi-client server chat
- Channels-based architecture for shared state management (avoiding `Arc<Mutex>`)
- Global chat messaging
- Private/Direct Messaging
- Global Notifications
- User authentication (basic implementation)
- Task-based architecture:
  - Client connection handling (send and receive)
  - Core message processing
  - User management (new users, user channels, user removal)
- Ping functionality for testing connection

## Project Structure

The project is organized into several modules:

- `connection`: Handles the low-level connection details and frame encoding/decoding
- `server`: Implements the server-side logic, including client handling and message processing
- `client`: Implements the client-side logic and user interface
- `common`: Contains shared data structures and message types

## To-Do

- [x] Send and receive frames (encode and decode data over the network as bytes)
- [x] Implement basic echo server
- [x] Create chat server handling multiple clients
- [x] Implement broadcasting messages to all connected clients
- [x] Implement private/direct messaging
- [ ] Better handling of user input
- [ ] Some terminal UI for the client
- [ ] Add support for multiple chat rooms
- [ ] Implement more robust authentication and user management
- [ ] Implement end-to-end encryption for messages
- [ ] Save chat history to a database

## How to Run

1. Start the server:

    `cargo run` or `just run`

2. Connect a client:

    `cargo run --bin client` or `just client`

    > Run this command in multiple terminal windows to simulate multiple clients.

## Available Client Commands

- `:quit` - Disconnect from the server
- `:ping` - Send a ping to the server
- `:pm <username> <message>` - Send a private message to a specific user
- `:users` - List all connected users

## References

For the initial implementation, I followed the tutorial [Lily Mara -Creating a Chat Server with async Rust and Tokio](https://www.youtube.com/watch?v=T2mWg91sx-o).

Once I get past the basic implementation, I will be implementing more features and making it more robust.
