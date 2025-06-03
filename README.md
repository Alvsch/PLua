# PLua - Lua Plugin Loader for Pumpkin

PLua is a plugin for the Pumpkin Minecraft server that enables loading and managing plugins written in Lua. This is a translation layer/runtime that allows server administrators to extend their Pumpkin server with Lua scripting capabilities.

## Installation

### Pre-built Binaries

Pre-built binaries for the most common architectures are available from GitHub Releases:

| Platform | Architecture | Download |
| -------- | ------------ | -------- |
| Linux    | amd64        | [Download](https://github.com/PumpkinPlugins/PLua/releases/latest/download/libplua_x86_64_linux.so) |
| Linux    | arm64        | [Download](https://github.com/PumpkinPlugins/PLua/releases/latest/download/libplua_aarch64_linux.so) |
| Windows  | amd64        | [Download](https://github.com/PumpkinPlugins/PLua/releases/latest/download/plua_x86_64_windows.dll) |

Download the appropriate file for your platform and place it in the `plugins` directory of your Pumpkin server.

You can also download all platform builds from the [latest release page](https://github.com/PumpkinPlugins/PLua/releases/latest).

### Building from Source

If you prefer to build from source:

```bash
# Clone the repository
git clone https://github.com/PumpkinPlugins/PLua.git
cd PLua

# Build the plugin
cargo build --release

# The compiled plugin will be in target/release/
```

For cross-compilation, you can specify the target platform:

```bash
# For Linux ARM64
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# For Windows
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## Features

- Load and manage Lua plugins
- Enable/disable plugins via in-game commands
- Hot reload plugins without restarting the server
- Simplified API for Lua plugins to interact with the Pumpkin server

## Getting Started

1. Install the plugin by downloading a pre-built binary or building from source.

2. Copy the compiled `.so` or `.dll` file to your Pumpkin server's plugins directory.

3. Start or restart your Pumpkin server.

4. The plugin will create a `plugins/plua` directory with a `plugins` subdirectory where Lua plugins will be stored.

5. Install some Lua plugins or create your own and place them in the `plugins/plua/plugins` directory.

6. Enable plugins using the in-game commands (see below).

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

Create a `.lua` file in the `plugins/plua/plugins` folder. Each plugin must have:

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

For more complex examples, check the `examples` directory in this repository.

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

-- Multiple plugins can register for the same event without conflicts
-- Each listener gets a unique ID that combines plugin name, timestamp and random value
print(join_listener) -- e.g. "listener_MyPlugin_player_join_1683724592123_3829572093"

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
