use std::sync::Arc;

use async_trait::async_trait;
use mlua::{Function, Lua, Table, Value};
use pumpkin::{
    plugin::{Context, EventHandler, EventPriority, player::player_chat::PlayerChatEvent},
    server::Server,
};
use pumpkin_api_macros::with_runtime;

use crate::lua::worker::{LuaCommand, send_event_command};

pub struct PlayerChatEventHandler;

#[with_runtime(global)]
#[async_trait]
impl EventHandler<PlayerChatEvent> for PlayerChatEventHandler {
    async fn handle_blocking(&self, _server: &Arc<Server>, event: &mut PlayerChatEvent) {
        let event_data = EventData {
            player_name: event.player.gameprofile.name.clone(),
            player_uuid: event.player.gameprofile.id.to_string(),
            message: event.message.clone(),
            recipients: event.recipients.len(),
        };

        if let Err(e) = send_event_command(LuaCommand::TriggerEvent {
            event_type: "player_chat".to_string(),
            event_data: serde_json::to_string(&event_data).unwrap_or_default(),
        }) {
            log::error!("Failed to send player chat event to Lua: {}", e);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EventData {
    player_name: String,
    player_uuid: String,
    message: String,
    recipients: usize,
}

pub async fn register(context: &Context) -> Result<(), String> {
    context
        .register_event(
            Arc::new(PlayerChatEventHandler),
            EventPriority::Lowest,
            true,
        )
        .await;

    Ok(())
}

pub fn setup_lua_event(lua: &Lua, events_table: &Table) -> mlua::Result<()> {
    let player_chat_listeners = lua.create_table()?;
    events_table.set("player_chat", player_chat_listeners)?;

    Ok(())
}

pub fn trigger_event(lua: &Lua, event_data_json: &str) -> mlua::Result<()> {
    let event_data: EventData = match serde_json::from_str(event_data_json) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to parse player chat event data: {}", e);
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

    let player_chat_listeners: Table = match events.get("player_chat") {
        Ok(listeners) => listeners,
        Err(_) => {
            return Ok(());
        }
    };

    let event_table = lua.create_table()?;
    event_table.set("player_name", event_data.player_name)?;
    event_table.set("player_uuid", event_data.player_uuid)?;
    event_table.set("message", event_data.message)?;
    event_table.set("recipients", event_data.recipients)?;

    for (_, callback) in player_chat_listeners.pairs::<Value, Function>().flatten() {
        if let Err(e) = callback.call::<()>(event_table.clone()) {
            log::error!("Error in player_chat event handler: {}", e);
        }
    }

    Ok(())
}
