# Hello Event Plugin

This is a simple example plugin for PLua that demonstrates how to use the events system.

## Features

- Listens for player join events
- Sends a welcome message to the player
- Demonstrates proper event listener cleanup on plugin disable

## Installation

1. Copy `hello_event.lua` to your server's `plugins/plua/plugins` directory
2. Enable the plugin with `/plua enable HelloEvent`

## Usage

The plugin will automatically welcome players when they join the server.

## Code Overview

This plugin demonstrates:

1. Proper plugin metadata declaration
2. Event listener registration with `pumpkin.events.register_listener`
3. Handling event data in Lua
4. Proper cleanup of event listeners when the plugin is disabled

## API Reference

### Event System

PLua provides an event system that allows Lua plugins to listen for Minecraft events. The following events are currently supported:

- `player_join`: Triggered when a player joins the server
  - Event data:
    - `player_name`: The name of the player
    - `player_uuid`: The UUID of the player
    - `join_message`: The join message

### Event Methods

- `pumpkin.events.register_listener(event_type, callback)`: Registers a listener for the specified event type
  - Returns a listener ID that can be used to unregister the listener
- `pumpkin.events.unregister_listener(event_type, listener_id)`: Unregisters a listener for the specified event type

## License

This example is provided under the same license as PLua.