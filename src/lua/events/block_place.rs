use std::sync::Arc;

use async_trait::async_trait;
use mlua::{Function, Lua, Table, Value};
use pumpkin::{
    plugin::{block::block_place::BlockPlaceEvent, Context, EventHandler, EventPriority},
    server::Server,
};
use pumpkin_api_macros::with_runtime;

use crate::lua::worker::{send_event_command, LuaCommand};

pub struct BlockPlaceEventHandler;

#[with_runtime(global)]
#[async_trait]
impl EventHandler<BlockPlaceEvent> for BlockPlaceEventHandler {
    async fn handle_blocking(&self, _server: &Arc<Server>, event: &mut BlockPlaceEvent) {
        let event_data = EventData {
            player_name: event.player.gameprofile.name.clone(),
            player_uuid: event.player.gameprofile.id.to_string(),
            block_placed: event.block_placed.name.to_string(),
            block_against: event.block_placed_against.name.to_string(),
            can_build: event.can_build,
        };

        if let Err(e) = send_event_command(LuaCommand::TriggerEvent {
            event_type: "block_place".to_string(),
            event_data: serde_json::to_string(&event_data).unwrap_or_default(),
        }) {
            log::error!("Failed to send block place event to Lua: {}", e);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EventData {
    player_name: String,
    player_uuid: String,
    block_placed: String,
    block_against: String,
    can_build: bool,
}

pub async fn register(context: &Context) -> Result<(), String> {
    context
        .register_event(
            Arc::new(BlockPlaceEventHandler),
            EventPriority::Lowest,
            true,
        )
        .await;

    Ok(())
}

pub fn setup_lua_event(lua: &Lua, events_table: &Table) -> mlua::Result<()> {
    let block_place_listeners = lua.create_table()?;
    events_table.set("block_place", block_place_listeners)?;

    Ok(())
}

pub fn trigger_event(lua: &Lua, event_data_json: &str) -> mlua::Result<()> {
    let event_data: EventData = match serde_json::from_str(event_data_json) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse block place event data: {}", e);
            return Ok(());
        }
    };

    let globals = lua.globals();
    let pumpkin: Table = globals.get("pumpkin")?;

    let events: Table = match pumpkin.get("events") {
        Ok(events) => events,
        Err(_) => {
            return Ok(());
        }
    };

    let block_place_listeners: Table = match events.get("block_place") {
        Ok(listeners) => listeners,
        Err(_) => {
            return Ok(());
        }
    };

    let event_table = lua.create_table()?;
    event_table.set("player_name", event_data.player_name)?;
    event_table.set("player_uuid", event_data.player_uuid)?;
    event_table.set("block_placed", event_data.block_placed)?;
    event_table.set("block_against", event_data.block_against)?;
    event_table.set("can_build", event_data.can_build)?;

    for pair in block_place_listeners.pairs::<Value, Function>() {
        if let Ok((_, callback)) = pair {
            if let Err(e) = callback.call::<()>(event_table.clone()) {
                log::error!("Error in block_place event handler: {}", e);
            }
        }
    }

    Ok(())
}
