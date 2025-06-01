-- Hello World Example Plugin for PLua
-- This is a simple example of a Lua plugin for the Pumpkin Minecraft server

-- Plugin metadata (required)
PLUGIN_INFO = {
    name = "HelloWorld",
    description = "A simple hello world plugin for PLua",
    version = "1.0.0",
    author = "PLua Team"
}

-- Called when the plugin is enabled
function on_enable()
    pumpkin.log.info("Hello World plugin enabled!")
end

-- Called when the plugin is disabled
function on_disable()
    pumpkin.log.info("Hello World plugin disabled!")
end

-- Example of a custom function
function greet_player(player_name)
    local message = "Hello, " .. player_name .. "! Welcome to the server!"
    pumpkin.server.broadcast_message(message)
end
