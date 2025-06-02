# Event Logger Plugin

This example plugin demonstrates how to use the PLua event system to listen for various Minecraft events.

## Features

- Listens for all available event types:
  - Player Join
  - Player Leave
  - Player Chat
  - Block Place
  - Block Break
- Logs detailed information about each event
- Demonstrates proper event listener registration and cleanup

## Installation

1. Copy `event_logger.lua` to your server's `plugins/plua/plugins` directory
2. Enable the plugin with `/plua enable EventLogger`

## Event Information

### Player Join Event
Triggered when a player joins the server.

Data provided:
- `player_name`: The name of the player
- `player_uuid`: The UUID of the player
- `join_message`: The message displayed when joining

### Player Leave Event
Triggered when a player leaves the server.

Data provided:
- `player_name`: The name of the player
- `player_uuid`: The UUID of the player
- `leave_message`: The message displayed when leaving

### Player Chat Event
Triggered when a player sends a chat message.

Data provided:
- `player_name`: The name of the player
- `player_uuid`: The UUID of the player
- `message`: The content of the chat message
- `recipients`: The number of players who will receive the message

### Block Place Event
Triggered when a player places a block.

Data provided:
- `player_name`: The name of the player
- `player_uuid`: The UUID of the player
- `block_placed`: The type of block being placed
- `block_against`: The type of block being placed against
- `can_build`: Whether the player is allowed to build in this location

### Block Break Event
Triggered when a block is broken.

Data provided:
- `player_name`: The name of the player (if a player broke it)
- `player_uuid`: The UUID of the player (if a player broke it)
- `block_type`: The type of block that was broken
- `position_x`, `position_y`, `position_z`: The coordinates of the block
- `experience`: The amount of experience that will drop
- `drop_items`: Whether items will drop from this block

## Code Structure

The plugin demonstrates:
1. Registering listeners for all event types
2. Storing listener IDs for proper cleanup
3. Properly handling event data with type checking
4. Logging detailed event information
5. Cleaning up listeners when the plugin is disabled

## License

This example is provided under the same license as PLua.