--!strict
-- Event Logger Plugin
-- A comprehensive plugin that logs all available events

-- Store listener IDs for cleanup
local listeners = {} :: {[any]: any}

local plugin: Plugin = {
    name = "EventLogger",
    description = "Logs all supported events in the PLua system",
    version = "1.0.0",
    author = "PLua",
    -- This function is called when the plugin is enabled
    on_enable = function()
        pumpkin.log.info("Event Logger plugin enabled!")
        
        -- Register player join event listener
        listeners.player_join = pumpkin.events.register_listener("player_join", function(event)
            pumpkin.log.info(string.format(
                "[JOIN] Player %s (%s) joined with message: %s",
                event.player_name,
                event.player_uuid,
                event.join_message
            ))
        end)
        
        -- Register player leave event listener
        listeners.player_leave = pumpkin.events.register_listener("player_leave", function(event)
            pumpkin.log.info(string.format(
                "[LEAVE] Player %s (%s) left with message: %s",
                event.player_name,
                event.player_uuid,
                event.leave_message
            ))
        end)
        
        -- Register player chat event listener
        listeners.player_chat = pumpkin.events.register_listener("player_chat", function(event)
            pumpkin.log.info(string.format(
                "[CHAT] Player %s (%s) said: '%s' to %d recipients",
                event.player_name,
                event.player_uuid,
                event.message,
                event.recipients
            ))
        end)
        
        -- Register block place event listener
        listeners.block_place = pumpkin.events.register_listener("block_place", function(event)
            pumpkin.log.info(string.format(
                "[BLOCK_PLACE] Player %s (%s) placed %s against %s (can build: %s)",
                event.player_name,
                event.player_uuid,
                event.block_placed,
                event.block_against,
                event.can_build and "yes" or "no"
            ))
        end)
        
        -- Register block break event listener
        listeners.block_break = pumpkin.events.register_listener("block_break", function(event)
            local player_info = "unknown"
            if event.player_name then
                player_info = string.format("%s (%s)", event.player_name, event.player_uuid or "unknown")
            end
            
            pumpkin.log.info(string.format(
                "[BLOCK_BREAK] Player %s broke %s at (%d,%d,%d) with %d exp (drop items: %s)",
                player_info,
                event.block_type,
                event.position_x,
                event.position_y,
                event.position_z,
                event.experience,
                event.drop_items and "yes" or "no"
            ))
        end)
        
        pumpkin.log.info("All event listeners registered successfully")
        pumpkin.server.broadcast_message("§aEvent Logger is now active - all events will be logged!")
    end,
    -- This function is called when the plugin is disabled
    on_disable = function()
        pumpkin.log.info("Event Logger plugin disabled!")
    
        -- Unregister all event listeners
        for event_type, listener_id in pairs(listeners) do
            pumpkin.events.unregister_listener(event_type, listener_id)
            pumpkin.log.info("Unregistered " .. event_type .. " listener: " .. listener_id)
        end
        
        -- Clear the listeners table
        listeners = {}
    end,
}

return plugin