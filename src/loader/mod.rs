use anyhow::anyhow;
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use anyhow::{Context as AnyhowContext, Result};
use async_trait::async_trait;
use mlua::Lua;
use pumpkin::plugin::{
    Context,
    api::{Plugin, PluginMetadata},
    loader::{LoaderError, PluginLoader},
};

use crate::SERVER;
use crate::lua::events;
use crate::lua::manifest::LuaPluginManifest;
use crate::lua::runtime::LuaRuntime;
use crate::lua::worker::{EVENT_SENDER, LuaCommand};
use mlua::{Function, RegistryKey};

static LUA_PLUGINS: OnceLock<Arc<RwLock<HashMap<String, Arc<Mutex<LuaPlugin>>>>>> = OnceLock::new();

fn get_lua_plugins() -> &'static Arc<RwLock<HashMap<String, Arc<Mutex<LuaPlugin>>>>> {
    LUA_PLUGINS.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

pub struct LuaPluginLoader {
    runtime: Arc<Mutex<LuaRuntime>>,
}

impl LuaPluginLoader {
    pub fn new() -> Result<Self> {
        let data_dir = PathBuf::from("plugins/plua");
        fs::create_dir_all(&data_dir).context("Failed to create PLua data directory")?;

        let runtime = LuaRuntime::new(&data_dir)?;
        runtime.init_api()?;

        Ok(Self {
            runtime: Arc::new(Mutex::new(runtime)),
        })
    }

    fn extract_metadata(&self, path: &Path) -> Result<PluginMetadata<'static>> {
        let script = fs::read_to_string(path)
            .with_context(|| format!("Failed to read plugin file: {:?}", path))?;

        let temp_lua = Lua::new();

        let lua_manifest = temp_lua
            .load(&script)
            .set_name(path.file_name().unwrap().to_string_lossy().as_ref())
            .eval::<LuaPluginManifest>()?;

        let static_name = Box::leak(lua_manifest.name.into_boxed_str());
        let static_desc = Box::leak(lua_manifest.description.into_boxed_str());
        let static_version = Box::leak(lua_manifest.version.into_boxed_str());
        let static_author = Box::leak(lua_manifest.author.into_boxed_str());

        Ok(PluginMetadata {
            name: static_name,
            description: static_desc,
            version: static_version,
            authors: static_author,
        })
    }

    fn create_lua_plugin(
        &self,
        path: &Path,
        metadata: PluginMetadata<'static>,
    ) -> Result<LuaPlugin> {
        let script = fs::read_to_string(path)
            .with_context(|| format!("Failed to read plugin file: {:?}", path))?;

        let runtime_clone = self.runtime.clone();

        let plugin = LuaPlugin {
            name: metadata.name.to_string(),
            file_path: path.to_path_buf(),
            script,
            runtime: runtime_clone.clone(),
            on_enable_called: false,
            on_enable_key: Mutex::new(None),
            on_disable_key: Mutex::new(None),
        };

        unsafe {
            #[allow(static_mut_refs)]
            if let Some(sender) = &EVENT_SENDER {
                let mut subscriber = sender.subscribe();
                let runtime_clone = runtime_clone.clone();
                tokio::spawn(async move {
                    while let Ok(cmd) = subscriber.recv().await {
                        match cmd {
                            LuaCommand::TriggerEvent {
                                event_type,
                                event_data,
                            } => {
                                handle_event(&runtime_clone, &event_type, &event_data);
                            }
                            _ => {}
                        }
                    }
                })
            } else {
                return Err(anyhow!("Event sender not initialized"));
            }
        };

        Ok(plugin)
    }
}

// TODO: Merge with worker.rs
fn handle_event(manager: &Mutex<LuaRuntime>, event_type: &str, event_data: &str) {
    match manager.lock() {
        Ok(lock) => match event_type {
            "player_join" => {
                if let Err(e) = events::player_join::trigger_event(&lock.lua, event_data) {
                    log::error!("Error triggering player_join event: {}", e);
                }
            }
            "player_leave" => {
                if let Err(e) = events::player_leave::trigger_event(&lock.lua, event_data) {
                    log::error!("Error triggering player_leave event: {}", e);
                }
            }
            "player_chat" => {
                if let Err(e) = events::player_chat::trigger_event(&lock.lua, event_data) {
                    log::error!("Error triggering player_chat event: {}", e);
                }
            }
            "block_place" => {
                if let Err(e) = events::block_place::trigger_event(&lock.lua, event_data) {
                    log::error!("Error triggering block_place event: {}", e);
                }
            }
            "block_break" => {
                if let Err(e) = events::block_break::trigger_event(&lock.lua, event_data) {
                    log::error!("Error triggering block_break event: {}", e);
                }
            }
            _ => {
                log::warn!("Unknown event type: {}", event_type);
            }
        },
        Err(e) => {
            log::error!("Failed to acquire lock for event handling: {:?}", e);
        }
    }
}

#[async_trait]
impl PluginLoader for LuaPluginLoader {
    async fn load(
        &self,
        path: &std::path::Path,
    ) -> Result<
        (
            Box<dyn Plugin>,
            PluginMetadata<'static>,
            Box<dyn Any + Send + Sync>,
        ),
        LoaderError,
    > {
        let path_buf = path.to_path_buf();
        log::info!("Loading plugin using PLua loader...");

        let metadata = self
            .extract_metadata(&path_buf)
            .map_err(|e| LoaderError::InitializationFailed(e.to_string()))?;

        let plugin = self
            .create_lua_plugin(&path_buf, metadata.clone())
            .map_err(|e| LoaderError::RuntimeError(e.to_string()))?;

        let plugin_name = plugin.name.clone();
        let plugin_arc = Arc::new(Mutex::new(plugin));

        {
            let plugins_arc = get_lua_plugins();
            let mut plugins = plugins_arc.write().unwrap();
            plugins.insert(plugin_name.clone(), plugin_arc.clone());
        }

        Ok((
            Box::new(LuaPluginWrapper { name: plugin_name }) as Box<dyn Plugin>,
            metadata,
            Box::new(plugin_arc.clone()) as Box<dyn Any + Send + Sync>,
        ))
    }

    fn can_load(&self, path: &std::path::Path) -> bool {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        ext.eq_ignore_ascii_case("lua") || ext.eq_ignore_ascii_case("luau")
    }

    async fn unload(&self, data: Box<dyn Any + Send + Sync>) -> Result<(), LoaderError> {
        let plugin_arc = match data.downcast::<Arc<Mutex<LuaPlugin>>>() {
            Ok(plugin) => plugin,
            Err(_) => return Err(LoaderError::InvalidLoaderData),
        };

        let plugin_name;
        {
            let plugin = plugin_arc.lock().unwrap();
            plugin_name = plugin.name.clone();

            if plugin.on_enable_called {
                if let Err(e) = plugin.call_on_disable() {
                    log::error!("Error calling on_disable for plugin {}: {}", plugin.name, e);
                }
            }
        }

        {
            let plugins_arc = get_lua_plugins();
            let mut plugins = plugins_arc.write().unwrap();
            plugins.remove(&plugin_name);
        }

        Ok(())
    }

    fn can_unload(&self) -> bool {
        true
    }
}

struct LuaPluginWrapper {
    name: String,
}

#[async_trait]
impl Plugin for LuaPluginWrapper {
    async fn on_load(&mut self, context: &Context) -> Result<(), String> {
        let plugin_arc = {
            let plugins_arc = get_lua_plugins();
            let plugins = plugins_arc.read().unwrap();
            match plugins.get(&self.name) {
                Some(arc) => arc.clone(),
                None => return Err(format!("Plugin {} not found in registry", self.name)),
            }
        };

        {
            let mut plugin = plugin_arc.lock().unwrap();
            if let Err(e) = plugin.prepare_plugin(context) {
                return Err(e);
            }
        }

        {
            let mut plugin = plugin_arc.lock().unwrap();
            plugin.complete_load().map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    async fn on_unload(&mut self, _context: &Context) -> Result<(), String> {
        let plugin_arc = {
            let plugins_arc = get_lua_plugins();
            let plugins = plugins_arc.read().unwrap();
            match plugins.get(&self.name) {
                Some(arc) => arc.clone(),
                None => return Err(format!("Plugin {} not found in registry", self.name)),
            }
        };

        let plugin = plugin_arc.lock().unwrap();
        if plugin.on_enable_called {
            plugin.call_on_disable().map_err(|e| e.to_string())
        } else {
            Ok(())
        }
    }
}

struct LuaPlugin {
    name: String,
    file_path: PathBuf,
    script: String,
    runtime: Arc<Mutex<LuaRuntime>>,
    on_enable_called: bool,
    on_enable_key: Mutex<Option<RegistryKey>>,
    on_disable_key: Mutex<Option<RegistryKey>>,
}

impl LuaPlugin {
    fn prepare_plugin(&mut self, context: &Context) -> Result<(), String> {
        let _ = SERVER.set(context.server.clone());

        let runtime = self.runtime.lock().unwrap();

        runtime
            .lua
            .load(&self.script)
            .set_name(
                self.file_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .as_ref(),
            )
            .exec()
            .map_err(|e| format!("Failed to execute plugin script: {}", e))?;

        let lua_manifest = runtime
            .lua
            .load(&self.script)
            .set_name(
                self.file_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .as_ref(),
            )
            .eval::<LuaPluginManifest>()
            .map_err(|e| format!("Failed to extract plugin manifest: {}", e))?;

        {
            let mut on_enable_key = self.on_enable_key.lock().unwrap();
            *on_enable_key = lua_manifest.on_enable.map(|f| {
                runtime.lua.create_registry_value(f).unwrap_or_else(|e| {
                    log::error!("Failed to store on_enable function: {}", e);
                    panic!("Failed to store on_enable function");
                })
            });
        }

        {
            let mut on_disable_key = self.on_disable_key.lock().unwrap();
            *on_disable_key = lua_manifest.on_disable.map(|f| {
                runtime.lua.create_registry_value(f).unwrap_or_else(|e| {
                    log::error!("Failed to store on_disable function: {}", e);
                    panic!("Failed to store on_disable function");
                })
            });
        }

        Ok(())
    }

    fn complete_load(&mut self) -> Result<()> {
        self.call_on_enable()?;
        self.on_enable_called = true;
        Ok(())
    }

    fn call_on_enable(&self) -> Result<()> {
        let runtime = self.runtime.lock().unwrap();
        let on_enable_key = self.on_enable_key.lock().unwrap();

        if let Some(key) = &*on_enable_key {
            let on_enable: Function = runtime.lua.registry_value(key)?;
            on_enable.call::<()>(())?;
        }

        Ok(())
    }

    fn call_on_disable(&self) -> Result<()> {
        let runtime = self.runtime.lock().unwrap();
        let on_disable_key = self.on_disable_key.lock().unwrap();

        if let Some(key) = &*on_disable_key {
            let on_disable: Function = runtime.lua.registry_value(key)?;
            on_disable.call::<()>(())?;
        }

        Ok(())
    }
}
