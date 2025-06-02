use std::sync::Arc;

use async_trait::async_trait;
use mlua::{Function, Lua, Table, Value};
use pumpkin::{
    plugin::{player::player_leave::PlayerLeaveEvent, Context, EventHandler, EventPriority},
    server::Server,
};
use pumpkin_api_macros::with_runtime;

use crate::lua::worker::send_event_command;
use crate::lua::worker::LuaCommand;

pub struct PlayerLeaveEventHandler;

#[with_runtime(global)]
#[async_trait]
impl EventHandler<PlayerLeaveEvent> for PlayerLeaveEventHandler {
    async fn handle_blocking(&self, _server: &Arc<Server>, event: &mut PlayerLeaveEvent) {
        let event_data = EventData {
            player_name: event.player.gameprofile.name.clone(),
            player_uuid: event.player.gameprofile.id.to_string(),
            leave_message: event.leave_message.clone().get_text(),
        };

        if let Err(e) = send_event_command(LuaCommand::TriggerEvent {
            event_type: "player_leave".to_string(),
            event_data: serde_json::to_string(&event_data).unwrap_or_default(),
        }) {
            log::error!("Failed to send player leave event to Lua: {}", e);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EventData {
    player_name: String,
    player_uuid: String,
    leave_message: String,
}

pub async fn register(context: &Context) -> Result<(), String> {
    context
        .register_event(
            Arc::new(PlayerLeaveEventHandler),
            EventPriority::Lowest,
            true,
        )
        .await;

    Ok(())
}

pub fn setup_lua_event(lua: &Lua, events_table: &Table) -> mlua::Result<()> {
    let player_leave_listeners = lua.create_table()?;
    events_table.set("player_leave", player_leave_listeners)?;

    Ok(())
}

pub fn trigger_event(lua: &Lua, event_data_json: &str) -> mlua::Result<()> {
    let event_data: EventData = match serde_json::from_str(event_data_json) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse player leave event data: {}", e);
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

    let player_leave_listeners: Table = match events.get("player_leave") {
        Ok(listeners) => listeners,
        Err(_) => {
            return Ok(());
        }
    };

    let event_table = lua.create_table()?;
    event_table.set("player_name", event_data.player_name)?;
    event_table.set("player_uuid", event_data.player_uuid)?;
    event_table.set("leave_message", event_data.leave_message)?;

    for pair in player_leave_listeners.pairs::<Value, Function>() {
        if let Ok((_, callback)) = pair {
            if let Err(e) = callback.call::<()>(event_table.clone()) {
                log::error!("Error in player_leave event handler: {}", e);
            }
        }
    }

    Ok(())
}