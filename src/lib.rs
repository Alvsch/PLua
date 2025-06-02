use std::sync::Arc;

use pumpkin::{plugin::Context, server::Server};
use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin_util::PermissionLvl;
use std::sync::OnceLock;

mod commands;
mod config;
mod lua;

pub static SERVER: OnceLock<Arc<Server>> = OnceLock::new();

impl PLuaPlugin {
    pub fn new() -> Self {
        PLuaPlugin {}
    }

    fn setup_lua(&mut self, context: &Context) -> Result<(), String> {
        lua::init_lua_manager(context.get_data_folder())
            .map_err(|e| format!("Failed to initialize Lua manager: {}", e))
    }

    async fn register_plua_command(&self, context: &Context) -> Result<(), String> {
        let command = commands::plua::init_command_tree();
        context.register_command(command, PermissionLvl::Four).await;
        Ok(())
    }
}

#[plugin_method]
async fn on_load(&mut self, context: &Context) -> Result<(), String> {
    pumpkin::init_log!();

    let _ = SERVER.set(context.server.clone());

    self.setup_lua(context)?;

    self.register_plua_command(context).await?;

    lua::events::register_events(context).await?;

    Ok(())
}

impl Default for PLuaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[plugin_impl]
pub struct PLuaPlugin {}
