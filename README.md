# Rust Chat Server

Understanding rust, tokio, threads, channels & async better by implementing a multi-client chat server from scratch using Rust and Tokio.

## Current Features

- Multi-client server chat
- Channels-based architecture for shared state management (avoiding `Arc<Mutex>` to use channels instead (simply for learning purposes))
- Global chat messaging
- Private/Direct Messaging
- Chat Rooms (Create, Join, Leave, List)
- Room-specific messaging
- Global Notifications
- User authentication (basic implementation)
- Task-based architecture:
  - Client connection handling (send and receive)
  - Core message processing
  - User management (new users, user channels, user removal)
  - Room management (create, join, leave, message routing)
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
- [x] Add support for multiple chat rooms
- [ ] Better handling of user input
- [ ] Some terminal UI for the client, ratatui?
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
- `:cr <room_name>` - Create a new chat room
- `:jr <room_name>` - Join a chat room
- `:lr <room_name>` - Leave a chat room
- `:lrs` - List all available rooms
- `:lru <room_name>` - List users in a specific room
- `:rm <room_name> <message>` - Send a message to a specific room

## Detailed Code Explanation

### Server Startup

When you run the server, the following sequence of events occurs:

1. The `main` function in `src/bin/server.rs` is executed.
2. It calls `init()` to set up logging and read the server address from environment variables.
3. A new `Server` instance is created and its `run()` method is called.
4. Inside `run()`:
   - A `TcpListener` is bound to the specified address.
   - Several channels are created for inter-component communication.
   - Three main components are initialized as separate Tokio tasks:
     - `UserProcessor`
     - `RoomProcessor`
     - `ServerProcessor`
   - The server enters a loop, accepting new client connections.

### Client Connection Handling

When a new client connects:

1. A `ClientHandler` is initialized for the new connection.
2. The `ClientHandler` performs authentication by exchanging a `Handshake` message.
3. If successful, a new Tokio task is spawned to handle this client's messages.

### Server-side Message Processing

The `ServerProcessor` is the central component for routing messages:

1. It receives messages from clients via the `ProcessMessage` enum.
2. Based on the message type, it routes the message to the appropriate handler:
   - User-related messages go to the `UserProcessor`
   - Room-related messages go to the `RoomProcessor`
   - Global messages are broadcast to all clients

### Room Management

Rooms are managed by the `RoomProcessor` and individual `RoomManager` instances:

1. The `RoomProcessor` maintains a HashMap of room names to `RoomManager` instances.
2. When a new room is created:
   - A new `RoomManager` is instantiated
   - A new Tokio task is spawned to run this `RoomManager`
   - The `RoomManager` is added to the HashMap
3. Room operations (join, leave, message) are handled by sending messages to the appropriate `RoomManager` task.
4. Each `RoomManager` maintains its own set of users and handles room-specific messaging.

This approach allows each room to operate independently and concurrently.

### User Management

User management is similar to room management but simpler:

1. The `UserProcessor` maintains a `UserManager` instance.
2. User operations (add, remove, list) are processed by the `UserProcessor`.
3. Unlike rooms, individual users don't have their own tasks. Instead, the `UserProcessor` handles all user-related operations.

### User Input Handling

User input is handled in the `Client` struct (`src/client/mod.rs`):

1. The `run()` method sets up a channel for user input.
2. A separate Tokio task is spawned to read from stdin continuously.
3. User input is parsed in the `parse_user_input()` function, which converts text commands to `ClientMessage` variants.
4. These messages are sent through the channel and processed in the main client loop.
5. Depending on the message type, it's either handled locally (e.g., `:ping`) or sent to the server.

### Asynchronous Design

The entire system is built on Tokio's asynchronous runtime:

1. Each major component (Server, UserProcessor, RoomProcessor, ClientHandler) runs in its own Tokio task.
2. Communication between components is primarily done through channels (mpsc and broadcast).
3. This design allows the system to handle many concurrent connections and operations efficiently.
4. Oneshot channels are used  to provide a clean end efficient way to handle one-time request-response interactions.

## References

For the initial implementation, I followed the tutorial [Lily Mara - Creating a Chat Server with async Rust and Tokio](https://www.youtube.com/watch?v=T2mWg91sx-o).
