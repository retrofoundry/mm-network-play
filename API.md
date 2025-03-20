# Network Play API Documentation

This document provides a detailed reference for the Network Sync API, which allows modders to add multiplayer functionality to Zelda 64: Recompiled mods.

## Core Functions

### Initialization & Connection

#### `void NS_Init()`
Initializes the network play system. This must be called before using any other networking functions.

- **Parameters:** None
- **Returns:** None
- **Usage:** Call this once at the beginning of your mod's execution.

#### `u8 NS_Connect(const char* host)`
Establishes a connection to the network server.

- **Parameters:**
  - `host`: String containing the WebSocket URL of the server (e.g., "ws://localhost:9002")
- **Returns:**
  - `1` if connection was successful
  - `0` if connection failed
- **Usage:** Call after initialization to connect to your network server.

### Session Management

#### `u8 NS_JoinSession(const char* session)`
Joins a multiplayer session on the server.

- **Parameters:**
  - `session`: String identifier for the session to join (e.g., "test-session")
- **Returns:**
  - `1` if joining was successful
  - `0` if joining failed
- **Usage:** Players must join the same session to see and interact with each other.

#### `u8 NS_LeaveSession()`
Leaves the current multiplayer session.

- **Parameters:** None
- **Returns:**
  - `1` if leaving was successful
  - `0` if leaving failed
- **Usage:** Call this when you want to disconnect from the current session.

### Actor Synchronization

#### `void NS_SyncActor(Actor* actor, const char* playerID, int isOwnedLocally)`
Registers an actor for network synchronization.

- **Parameters:**
  - `actor`: Pointer to the Actor to be synchronized
  - `playerID`: String identifier for this actor
    - if actor->id == 0, we use the server defined id (identifies players)
  - `isOwnedLocally`:
    - `1` if this client should send updates for this actor
    - `0` if this client should only receive updates
- **Returns:** None
- **Usage:** Call this to start synchronizing an actor across the network.

#### `const char* NS_GetActorNetworkId(Actor *actor)`
Gets the network identifier for a registered actor.

- **Parameters:**
  - `actor`: Pointer to the Actor to query
- **Returns:**
  - String containing the network ID if the actor is registered
  - `NULL` if the actor is not registered for synchronization
- **Usage:** Use to retrieve the unique network identifier for an actor.

### Remote Player Data

#### `u32 NS_GetRemoteActorIDs(u32 maxPlayers, char* idsBuffer, u32 idBufferSize)`
Gets a list of connected remote player IDs.

- **Parameters:**
  - `maxPlayers`: Maximum number of player IDs to retrieve
  - `idsBuffer`: Buffer to store player IDs (should be at least maxPlayers * idBufferSize)
  - `idBufferSize`: Size of each player ID string buffer
- **Returns:** Number of remote player IDs retrieved
- **Usage:** Call this to get a list of all other players in the session.

#### `u32 NS_GetRemoteActorData(const char* playerID, void* dataBuffer)`
Retrieves the most recent data for a specific remote player.

- **Parameters:**
  - `playerID`: String identifier of the remote player
  - `dataBuffer`: Buffer to store the player's data (should be a `PlayerSyncData` struct)
- **Returns:**
  - `1` if data was successfully retrieved
  - `0` if data could not be retrieved
- **Usage:** Call this to get the latest position, animation, and state data for a remote player.

### Custom Message Handling

#### `u8 NS_RegisterMessageHandler(const char* messageId, u32 payloadSize, void* callback)`
Registers a callback function to handle custom messages of a specific type.

- **Parameters:**
  - `messageId`: String identifier for the message type
  - `payloadSize`: Size of the expected message payload in bytes
  - `callback`: Function pointer to the callback that will handle messages of this type
    - Callback signature: `void (*callback)(void* data)`
- **Returns:**
  - `0` if registration was successful
  - `1` if registration failed
- **Usage:** Call during initialization to set up handlers for custom message types.

#### `u8 NS_EmitMessage(const char* messageId, void* data)`
Sends a custom message to all other clients in the session.

- **Parameters:**
  - `messageId`: String identifier for the message type (must match a registered handler)
  - `data`: Pointer to the message payload data
- **Returns:**
  - `0` if message was sent successfully
  - `1` if sending failed
- **Usage:** Call to broadcast custom messages to other clients.

## Data Structures

### `PlayerSyncData`
Structure containing synchronized data for players.

```c
typedef struct {
    Vec3s shapeRotation;  // Actor shape rotation
    Vec3f worldPosition;  // Actor world position

    // Player Actor properties
    s8 currentBoots;      // Current boots equipped by the player
    s8 currentShield;     // Current shield equipped by the player
    u8 _padding[2];       // Padding for alignment
    Vec3s jointTable[24]; // Animation joint positions
    Vec3s upperLimbRot;   // Upper body rotation
} PlayerSyncData;
```

## Best Practices

1. **Call NS_Init() early**: Initialize the network system before attempting to use other functions.

2. **Error handling**: Always check return values from connect/join functions.

3. **Actor ownership**: For each actor, only one client should set `isOwnedLocally` to 1.

4. **Remote player rendering**: Create separate actor instances for remote players and update them with `NS_GetRemoteActorData()`.

5. **Session design**: Use meaningful session names to separate different multiplayer groups.
