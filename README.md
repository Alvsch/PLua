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

## Plugin Lifecycle

1. PLua scans the `plugins` directory for `.lua` files
2. It loads plugin metadata from each file
3. Enabled plugins are initialized by:
   a. Loading the plugin script
   b. Calling its `on_enable` function
4. When plugins are disabled, their `on_disable` function is called

## Future Enhancements

- Event system to let Lua plugins respond to game events
- Command registration API
- Player and world manipulation
- Task scheduling
- Configuration file API for Lua plugins

## License

Same license as the Pumpkin server.