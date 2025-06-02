# PLua - Lua Plugin Loader for Pumpkin

PLua is a plugin for the Pumpkin Minecraft server that enables loading and managing plugins written in Lua. This is a translation layer/runtime that allows server administrators to extend their Pumpkin server with Lua scripting capabilities.

## Features

- Load and manage Lua plugins
- Enable/disable plugins via in-game commands
- Hot reload plugins without restarting the server
- Simplified API for Lua plugins to interact with the Pumpkin server

## Installation

1. Build the plugin:
   ```
   cargo build --release
   ```

2. Copy the compiled `.so` or `.dll` file from `target/release/` to your Pumpkin server's plugins directory.

3. Restart your Pumpkin server.

## Commands

PLua provides the following in-game commands:

- `/plua list` - Lists all available Lua plugins and their status
- `/plua enable <plugin_name>` - Enables a plugin
- `/plua disable <plugin_name>` - Disables a plugin
- `/plua reload` - Reloads all plugins
- `/plua reload <plugin_name>` - Reloads a specific plugin
- `/plua info <plugin_name>` - Shows detailed information about a plugin

## Writing Lua Plugins

### Plugin Structure

Create a `.lua` file in the `plugins` folder in the PLua data directory. Each plugin must have:

1. A `PLUGIN_INFO` table with metadata
2. `on_enable` and `on_disable` functions (optional but recommended)

Basic example:

```lua
-- Plugin metadata (required)
PLUGIN_INFO = {
    name = "MyPlugin",
    description = "An awesome plugin",
    version = "1.0.0",
    author = "Your Name"
}

-- Called when the plugin is enabled
function on_enable()
    pumpkin.log.info("MyPlugin enabled!")
    -- Your initialization code here
end

-- Called when the plugin is disabled
function on_disable()
    pumpkin.log.info("MyPlugin disabled!")
    -- Your cleanup code here
end
```

### Available API

PLua exposes a global `pumpkin` table with the following functionality:

#### Logging
```lua
pumpkin.log.info("Information message")
pumpkin.log.warn("Warning message")
pumpkin.log.error("Error message")
pumpkin.log.debug("Debug message")
```

#### Server
```lua
-- Send a message to all players
pumpkin.server.broadcast_message("Hello everyone!")
```

#### Events
```lua
-- Register event listeners
local join_listener = pumpkin.events.register_listener("player_join", function(event)
    pumpkin.log.info("Player joined: " .. event.player_name)
    -- Access event data: event.player_name, event.player_uuid, event.join_message
end)

local chat_listener = pumpkin.events.register_listener("player_chat", function(event)
    pumpkin.log.info("Chat message: " .. event.message)
    -- Access event data: event.player_name, event.player_uuid, event.message, event.recipients
end)

-- Unregister event listeners
pumpkin.events.unregister_listener("player_join", join_listener)
pumpkin.events.unregister_listener("player_chat", chat_listener)
```

## Plugin Lifecycle

1. PLua scans the `plugins` directory for `.lua` files
2. It loads plugin metadata from each file
3. Enabled plugins are initialized by:
   a. Loading the plugin script
   b. Calling its `on_enable` function
4. When plugins are disabled, their `on_disable` function is called

## Event System

PLua includes an event system that allows Lua plugins to respond to game events. Currently supported events:

### Player Join Event
Triggered when a player joins the server.

Event data:
- `player_name`: The name of the player
- `player_uuid`: The UUID of the player
- `join_message`: The join message

### Player Leave Event
Triggered when a player leaves the server.

Event data:
- `player_name`: The name of the player
- `player_uuid`: The UUID of the player
- `leave_message`: The leave message

### Player Chat Event
Triggered when a player sends a chat message.

Event data:
- `player_name`: The name of the player
- `player_uuid`: The UUID of the player
- `message`: The content of the chat message
- `recipients`: The number of players who will receive the message

### Block Place Event
Triggered when a player places a block.

Event data:
- `player_name`: The name of the player
- `player_uuid`: The UUID of the player
- `block_placed`: The type of block being placed
- `block_against`: The type of block being placed against
- `can_build`: Whether the player is allowed to build in this location

### Block Break Event
Triggered when a block is broken.

Event data:
- `player_name`: The name of the player (if a player broke it, otherwise nil)
- `player_uuid`: The UUID of the player (if a player broke it, otherwise nil)
- `block_type`: The type of block that was broken
- `position_x`, `position_y`, `position_z`: The coordinates of the block
- `experience`: The amount of experience that will drop
- `drop_items`: Whether items will drop from this block

See the `examples/hello_event` and `examples/event_logger` directories for sample plugins that use the event system.

## Future Enhancements

- More events (entity interactions, inventory actions, etc.)
- Command registration API
- Player and world manipulation
- Task scheduling
- Configuration file API for Lua plugins

## License

Same license as the Pumpkin server.