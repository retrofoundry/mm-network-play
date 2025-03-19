# Network Sync API for Zelda 64: Recompiled

A multiplayer networking framework for Zelda 64: Recompiled that enables Actor synchronization across different game instances.


> [!WARNING]
> This project is still in active development and the API is subject to change. It represents an experimental implementation of networking capabilities and should be considered a work-in-progress.


## Project Overview

This project consists of several components that work together to provide network functionality:

1. **Network Sync API** (`network-sync`) - An API mod that exposes networking functionality to other mods
2. **Network Sync Runtime** (`network-sync-runtime`) - A Rust-based dynamic library that implements the networking logic
3. **Network Server** (`network-sync-server`) - A webSocket server that handles player connections and data relay
4. **Test Mod** (`network-sync-test`) - A sample implementation that demonstrates the networking functionality

## API Reference

See the full [API Documentation](API.md) for detailed information about available functions and data structures.

The Network Play API provides functionality for:

- Connecting to network servers
- Joining and leaving multiplayer sessions
- Synchronizing actor data across game instances
- Retrieving information about other players in a session

### Architecture

The system uses a client-server architecture where:
- A central websocket server manages connections and relays player state
- Each game instance connects to the server as a client
- Players can join "sessions" where their data is synchronized
- Actor attributes are synchronized across game instances

## Limitations

- Currently only synchronizes limited player data (position, rotation)
- Limited error handling and reconnection logic

### Test Implementation

The test mod demonstrates:
- Connecting to the server at game startup
- Joining a predefined session
- Synchronizing the player's position
- Rendering remote player models

## Setup & Building

### Prerequisites

- Clang and Make for building the C components
- Cargo/Rust for building the Rust components
- Zelda 64: Recompiled installation

### Building the Project

1. Clone this repository:
   ```
   git clone https://github.com/yourusername/mm-network-sync.git
   cd mm-network-sync
   ```

2. Build the runtime, C API mod, and test mod:
   ```
   make
   ```
   This will:
   - Compile the Rust WebSocket library
   - Build the C API mod
   - Build the test mod

3. Copy the built files to your Zelda 64: Recompiled mods directory:
   ```
   make install
   ```
   Or manually copy `build/main/mm_network_sync.nrm`, `build/test/mm_network_sync_test.nrm`, and the compiled `build/network_sync_runtime.dylib` (or `.so`/`.dll`) to your mods folder.

4. Run the server:
Using Cargo:
   ```
   cd network-server
   cargo run
   ```

   Using Docker Compose:
   ```
   docker compose up -d
   ```
   This will start the network server in a detached mode, which is useful for running it in the background.

5. Launch Zelda 64: Recompiled with the mods enabled.
