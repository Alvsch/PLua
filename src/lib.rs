use std::sync::Arc;

use pumpkin::{plugin::Context, server::Server};
use pumpkin_api_macros::{plugin_impl, plugin_method};
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault},
};
use std::sync::OnceLock;

mod commands;
mod config;
mod loader;
mod lua;

use loader::LuaPluginLoader;

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
        let permission = Permission::new(
            crate::commands::plua::PERMISSION_NODE,
            "Allow running the /plua command",
            PermissionDefault::Op(PermissionLvl::Four),
        );
        context.register_permission(permission).await?;
        context
            .register_command(command, crate::commands::plua::PERMISSION_NODE)
            .await;
        Ok(())
    }

    async fn register_lua_loader(&self, context: &Context) -> Result<(), String> {
        let lua_loader = match LuaPluginLoader::new() {
            Ok(loader) => loader,
            Err(e) => return Err(format!("Failed to create Lua plugin loader: {}", e)),
        };

        lua::events::register_events(context).await?;

        let plugin_manager = context.plugin_manager.clone();
        let loader = Arc::new(lua_loader);
        tokio::spawn(async move {
            let plugin_manager = plugin_manager.clone();
            let loader = loader.clone();
            let mut manager = plugin_manager.write().await;
            manager.add_loader(loader).await;
        });

        Ok(())
    }
}

#[plugin_method]
async fn on_load(&mut self, context: &Context) -> Result<(), String> {
    pumpkin::init_log!();

    let _ = SERVER.set(context.server.clone());

    self.setup_lua(context)?;
    self.register_plua_command(context).await?;

    self.register_lua_loader(context).await?;

    Ok(())
}

impl Default for PLuaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[plugin_impl]
pub struct PLuaPlugin {}
