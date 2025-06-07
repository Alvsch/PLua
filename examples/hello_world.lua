--!strict
-- Hello World Example Plugin for PLua
-- This is a simple example of a Lua plugin for the Pumpkin Minecraft server

-- Example of a custom function
function _greet_player(player_name: string)
    local message = "Hello, " .. player_name .. "! Welcome to the server!"
    pumpkin.server.broadcast_message(message)
end

-- Plugin metadata (required)
local plugin: Plugin = {
    name = "HelloWorld",
    description = "A simple hello world plugin for PLua",
    version = "1.0.0",
    author = "PLua Team",
    -- Called when the plugin is enabled
    on_enable = function()
        pumpkin.log.info("Hello World plugin enabled!")
    end,
    -- Called when the plugin is disabled
    on_disable = function()
        pumpkin.log.info("Hello World plugin disabled!")
    end,
};

return plugin