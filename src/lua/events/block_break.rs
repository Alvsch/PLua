use std::sync::Arc;

use async_trait::async_trait;
use mlua::{Function, Lua, Table, Value};
use pumpkin::{
    plugin::{block::block_break::BlockBreakEvent, Context, EventHandler, EventPriority},
    server::Server,
};
use pumpkin_api_macros::with_runtime;

use crate::lua::worker::{send_event_command, LuaCommand};

pub struct BlockBreakEventHandler;

#[with_runtime(global)]
#[async_trait]
impl EventHandler<BlockBreakEvent> for BlockBreakEventHandler {
    async fn handle_blocking(&self, _server: &Arc<Server>, event: &mut BlockBreakEvent) {
        let event_data = EventData {
            player_name: event.player.as_ref().map(|p| p.gameprofile.name.clone()),
            player_uuid: event.player.as_ref().map(|p| p.gameprofile.id.to_string()),
            block_type: event.block.name.to_string(),
            position_x: event.block_position.0.x,
            position_y: event.block_position.0.y,
            position_z: event.block_position.0.z,
            experience: event.exp,
            drop_items: event.drop,
        };

        if let Err(e) = send_event_command(LuaCommand::TriggerEvent {
            event_type: "block_break".to_string(),
            event_data: serde_json::to_string(&event_data).unwrap_or_default(),
        }) {
            log::error!("Failed to send block break event to Lua: {}", e);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EventData {
    player_name: Option<String>,
    player_uuid: Option<String>,
    block_type: String,
    position_x: i32,
    position_y: i32,
    position_z: i32,
    experience: u32,
    drop_items: bool,
}

pub async fn register(context: &Context) -> Result<(), String> {
    context
        .register_event(
            Arc::new(BlockBreakEventHandler),
            EventPriority::Lowest,
            true,
        )
        .await;

    Ok(())
}

pub fn setup_lua_event(lua: &Lua, events_table: &Table) -> mlua::Result<()> {
    let block_break_listeners = lua.create_table()?;
    events_table.set("block_break", block_break_listeners)?;

    Ok(())
}

pub fn trigger_event(lua: &Lua, event_data_json: &str) -> mlua::Result<()> {
    let event_data: EventData = match serde_json::from_str(event_data_json) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse block break event data: {}", e);
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

    let block_break_listeners: Table = match events.get("block_break") {
        Ok(listeners) => listeners,
        Err(_) => {
            return Ok(());
        }
    };

    let event_table = lua.create_table()?;
    if let Some(player_name) = &event_data.player_name {
        event_table.set("player_name", player_name.clone())?;
    }
    if let Some(player_uuid) = &event_data.player_uuid {
        event_table.set("player_uuid", player_uuid.clone())?;
    }
    event_table.set("block_type", event_data.block_type)?;
    event_table.set("position_x", event_data.position_x)?;
    event_table.set("position_y", event_data.position_y)?;
    event_table.set("position_z", event_data.position_z)?;
    event_table.set("experience", event_data.experience)?;
    event_table.set("drop_items", event_data.drop_items)?;

    for pair in block_break_listeners.pairs::<Value, Function>() {
        if let Ok((_, callback)) = pair {
            if let Err(e) = callback.call::<_, ()>(event_table.clone()) {
                log::error!("Error in block_break event handler: {}", e);
            }
        }
    }

    Ok(())
}
