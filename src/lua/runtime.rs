use anyhow::{Context as AnyhowContext, Result};
use mlua::{Function, Lua, Table};
use pumpkin_util::text::TextComponent;
use rand::{Rng, rng};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::SERVER;
use crate::config::ConfigManager;
use crate::lua::events;
use crate::lua::manifest::LuaPluginManifest;

pub struct LuaPlugin {
    pub manifest: LuaPluginManifest,
    pub file_path: PathBuf,
    pub enabled: bool,
}

pub struct LuaRuntime {
    pub lua: Lua,
    pub plugins_dir: PathBuf,
    pub plugins: HashMap<String, LuaPlugin>,
}

impl LuaRuntime {
    pub fn new(data_dir: &Path) -> Result<Self> {
        let plugins_dir = data_dir.join("plugins");
        fs::create_dir_all(&plugins_dir).context("Failed to create plugins directory")?;

        let lua = Lua::new();

        Ok(Self {
            lua,
            plugins_dir,
            plugins: HashMap::new(),
        })
    }

    pub fn discover_plugins(&mut self) -> Result<()> {
        self.plugins.clear();

        let entries =
            fs::read_dir(&self.plugins_dir).context("Failed to read plugins directory")?;

        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "lua") {
                self.load_plugin_metadata(&path)?;
            }
        }

        Ok(())
    }

    fn load_plugin_metadata(&mut self, path: &Path) -> Result<()> {
        let script = fs::read_to_string(path)
            .with_context(|| format!("Failed to read plugin file: {:?}", path))?;

        // Only used for metadata extraction
        let temp_lua = Lua::new();

        let manifest = temp_lua
            .load(&script)
            .set_name(path.file_name().unwrap().to_string_lossy().as_ref())
            .eval::<LuaPluginManifest>()?;

        let plugin = LuaPlugin {
            manifest,
            file_path: path.to_path_buf(),
            enabled: false,
        };

        self.plugins.insert(plugin.manifest.name.clone(), plugin);

        Ok(())
    }

    pub fn init_api(&self) -> Result<()> {
        let lua = &self.lua;

        let pumpkin_table = lua.create_table()?;
        lua.globals().set("pumpkin", pumpkin_table.clone())?;

        {
            let log_table = lua.create_table()?;

            log_table.set(
                "info",
                lua.create_function(|_, message: String| {
                    log::info!("[Lua] {}", message);
                    Ok(())
                })?,
            )?;

            log_table.set(
                "warn",
                lua.create_function(|_, message: String| {
                    log::warn!("[Lua] {}", message);
                    Ok(())
                })?,
            )?;

            log_table.set(
                "error",
                lua.create_function(|_, message: String| {
                    log::error!("[Lua] {}", message);
                    Ok(())
                })?,
            )?;

            log_table.set(
                "debug",
                lua.create_function(|_, message: String| {
                    log::debug!("[Lua] {}", message);
                    Ok(())
                })?,
            )?;

            pumpkin_table.set("log", log_table)?;
        }

        {
            let server_table = lua.create_table()?;

            server_table.set(
                "broadcast_message",
                lua.create_async_function(move |_, message: String| async move {
                    if let Some(server) = SERVER.get() {
                        for p in server.get_all_players().await {
                            p.send_system_message(&TextComponent::text(message.clone()))
                                .await;
                        }
                    }
                    Ok(())
                })?,
            )?;

            pumpkin_table.set("server", server_table)?;
        }

        {
            let events_table = lua.create_table()?;

            events_table.set(
                "register_listener",
                lua.create_function(|lua_ctx, (event_type, callback): (String, Function)| {
                    let globals = lua_ctx.globals();
                    let pumpkin: Table = globals.get("pumpkin")?;
                    let events: Table = pumpkin.get("events")?;

                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis();

                    let random = rng().random::<u32>();

                    let plugin_name = lua_ctx
                        .globals()
                        .get::<Table>("PLUGIN_INFO")
                        .and_then(|t| t.get::<String>("name"))
                        .unwrap_or_else(|_| "unknown".to_string());

                    let callback_name = callback
                        .info()
                        .name
                        .unwrap_or_else(|| event_type.clone())
                        .replace(|c: char| !c.is_alphanumeric(), "");

                    let listener_id = format!(
                        "listener_{}_{}_{}_{}",
                        plugin_name, callback_name, timestamp, random
                    );

                    match event_type.as_str() {
                        "player_join" => {
                            let listeners: Table = events.get("player_join")?;
                            listeners.set(listener_id.clone(), callback)?;
                            Ok(listener_id)
                        }
                        "player_leave" => {
                            let listeners: Table = events.get("player_leave")?;
                            listeners.set(listener_id.clone(), callback)?;
                            Ok(listener_id)
                        }
                        "player_chat" => {
                            let listeners: Table = events.get("player_chat")?;
                            listeners.set(listener_id.clone(), callback)?;
                            Ok(listener_id)
                        }
                        "block_place" => {
                            let listeners: Table = events.get("block_place")?;
                            listeners.set(listener_id.clone(), callback)?;
                            Ok(listener_id)
                        }
                        "block_break" => {
                            let listeners: Table = events.get("block_break")?;
                            listeners.set(listener_id.clone(), callback)?;
                            Ok(listener_id)
                        }
                        _ => Err(mlua::Error::RuntimeError(format!(
                            "Unknown event type: {}",
                            event_type
                        ))),
                    }
                })?,
            )?;

            events_table.set(
                "unregister_listener",
                lua.create_function(|lua_ctx, (event_type, listener_id): (String, String)| {
                    let globals = lua_ctx.globals();
                    let pumpkin: Table = globals.get("pumpkin")?;
                    let events: Table = pumpkin.get("events")?;

                    match event_type.as_str() {
                        "player_join" => {
                            let listeners: Table = events.get("player_join")?;
                            listeners.set(listener_id, mlua::Value::Nil)?;
                            Ok(true)
                        }
                        "player_leave" => {
                            let listeners: Table = events.get("player_leave")?;
                            listeners.set(listener_id, mlua::Value::Nil)?;
                            Ok(true)
                        }
                        "player_chat" => {
                            let listeners: Table = events.get("player_chat")?;
                            listeners.set(listener_id, mlua::Value::Nil)?;
                            Ok(true)
                        }
                        "block_place" => {
                            let listeners: Table = events.get("block_place")?;
                            listeners.set(listener_id, mlua::Value::Nil)?;
                            Ok(true)
                        }
                        "block_break" => {
                            let listeners: Table = events.get("block_break")?;
                            listeners.set(listener_id, mlua::Value::Nil)?;
                            Ok(true)
                        }
                        _ => Err(mlua::Error::RuntimeError(format!(
                            "Unknown event type: {}",
                            event_type
                        ))),
                    }
                })?,
            )?;

            events::player_join::setup_lua_event(lua, &events_table)?;
            events::player_leave::setup_lua_event(lua, &events_table)?;
            events::player_chat::setup_lua_event(lua, &events_table)?;
            events::block_place::setup_lua_event(lua, &events_table)?;
            events::block_break::setup_lua_event(lua, &events_table)?;

            pumpkin_table.set("events", events_table)?;
        }

        Ok(())
    }

    pub fn enable_plugin(&mut self, name: &str) -> Result<bool> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            if plugin.enabled {
                return Ok(false);
            }

            let script = fs::read_to_string(&plugin.file_path)
                .with_context(|| format!("Failed to read plugin file: {:?}", plugin.file_path))?;

            self.lua
                .load(&script)
                .set_name(
                    plugin
                        .file_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .as_ref(),
                )
                .exec()
                .with_context(|| {
                    format!("Failed to execute plugin script: {:?}", plugin.file_path)
                })?;

            if let Some(on_enable) = &plugin.manifest.on_enable {
                on_enable
                    .call::<()>(())
                    .with_context(|| format!("Failed to call on_enable for plugin {}", name))?;
            }

            plugin.enabled = true;
            Ok(true)
        } else {
            log::warn!("Attempted to enable unknown plugin: {}", name);
            Ok(false)
        }
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<bool> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            if !plugin.enabled {
                return Ok(false);
            }

            if let Some(on_disable) = &plugin.manifest.on_disable {
                on_disable
                    .call::<()>(())
                    .with_context(|| format!("Failed to call on_disable for plugin {}", name))?;
            }

            plugin.enabled = false;
            Ok(true)
        } else {
            log::warn!("Attempted to disable unknown plugin: {}", name);
            Ok(false)
        }
    }

    pub fn load_enabled_plugins(&mut self, config_manager: &ConfigManager) -> Result<()> {
        for plugin_name in &config_manager.config.enabled_plugins {
            if let Some(plugin) = self.plugins.get(plugin_name) {
                if !plugin.enabled {
                    if let Err(e) = self.enable_plugin(plugin_name) {
                        log::error!("Failed to enable plugin {}: {}", plugin_name, e);
                    }
                }
            } else {
                log::warn!("Enabled plugin {} not found", plugin_name);
            }
        }

        Ok(())
    }

    pub fn disable_all_plugins(&mut self) -> Result<()> {
        let mut to_disable = vec![];

        for (name, plugin) in &mut self.plugins {
            if plugin.enabled {
                to_disable.push(name.clone());
            }
        }

        for name in to_disable {
            if let Err(e) = self.disable_plugin(name.as_str()) {
                log::error!("Failed to disable plugin {}: {}", name, e);
            }
        }

        Ok(())
    }

    pub fn reload_plugin(&mut self, name: &str) -> Result<bool> {
        let was_enabled = if let Some(plugin) = self.plugins.get(name) {
            plugin.enabled
        } else {
            return Ok(false);
        };

        if was_enabled {
            self.disable_plugin(name)?;
        }

        let fp = {
            let plugin = self.plugins.get(name);
            if plugin.is_none() {
                return Ok(false);
            }
            plugin.unwrap().file_path.clone()
        };

        self.load_plugin_metadata(&fp)?;

        if was_enabled {
            self.enable_plugin(name)?;
        }

        Ok(true)
    }
}
