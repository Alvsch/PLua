use std::sync::Arc;

use async_trait::async_trait;
use mlua::{Function, Lua, Table, Value};
use pumpkin::{
    plugin::{player::player_join::PlayerJoinEvent, Context, EventHandler, EventPriority},
    server::Server,
};
use pumpkin_api_macros::with_runtime;

use crate::lua::worker::{send_event_command, LuaCommand};

pub struct PlayerJoinEventHandler;

#[with_runtime(global)]
#[async_trait]
impl EventHandler<PlayerJoinEvent> for PlayerJoinEventHandler {
    async fn handle_blocking(&self, _server: &Arc<Server>, event: &mut PlayerJoinEvent) {
        let event_data = EventData {
            player_name: event.player.gameprofile.name.clone(),
            player_uuid: event.player.gameprofile.id.to_string(),
            join_message: event.join_message.clone().get_text(),
        };

        if let Err(e) = send_event_command(LuaCommand::TriggerEvent {
            event_type: "player_join".to_string(),
            event_data: serde_json::to_string(&event_data).unwrap_or_default(),
        }) {
            log::error!("Failed to send player join event to Lua: {}", e);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EventData {
    player_name: String,
    player_uuid: String,
    join_message: String,
}

pub async fn register(context: &Context) -> Result<(), String> {
    context
        .register_event(
            Arc::new(PlayerJoinEventHandler),
            EventPriority::Lowest,
            true,
        )
        .await;

    Ok(())
}

pub fn setup_lua_event(lua: &Lua, events_table: &Table) -> mlua::Result<()> {
    let player_join_listeners = lua.create_table()?;
    events_table.set("player_join", player_join_listeners)?;

    Ok(())
}

pub fn trigger_event(lua: &Lua, event_data_json: &str) -> mlua::Result<()> {
    let event_data: EventData = match serde_json::from_str(event_data_json) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse player join event data: {}", e);
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

    let player_join_listeners: Table = match events.get("player_join") {
        Ok(listeners) => listeners,
        Err(_) => {
            return Ok(());
        }
    };

    let event_table = lua.create_table()?;
    event_table.set("player_name", event_data.player_name)?;
    event_table.set("player_uuid", event_data.player_uuid)?;
    event_table.set("join_message", event_data.join_message)?;

    for pair in player_join_listeners.pairs::<Value, Function>() {
        if let Ok((_, callback)) = pair {
            if let Err(e) = callback.call::<_, ()>(event_table.clone()) {
                log::error!("Error in player_join event handler: {}", e);
            }
        }
    }

    Ok(())
}
