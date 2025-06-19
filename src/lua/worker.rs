use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Mutex, Once};
use tokio::sync::broadcast::{Receiver, Sender};

use anyhow::{Result, anyhow};

use super::events;
use super::runtime::LuaRuntime;
use crate::config::ConfigManager;

pub type PluginInfo = (String, String, String, String, bool, PathBuf);

pub struct LuaManager {
    pub runtime: LuaRuntime,
    pub config_manager: ConfigManager,
    initialized: bool,
    registry_refs: RefCell<Vec<String>>,
}

impl LuaManager {
    pub fn new(data_path: &Path) -> Result<Self> {
        let config_manager = ConfigManager::new(data_path)
            .map_err(|e| anyhow!("Failed to initialize config manager: {}", e))?;

        let runtime = LuaRuntime::new(data_path)
            .map_err(|e| anyhow!("Failed to initialize Lua runtime: {}", e))?;

        Ok(Self {
            runtime,
            config_manager,
            initialized: false,
            registry_refs: RefCell::new(Vec::new()),
        })
    }

    pub fn register_plugin_ref(&self, plugin_name: &str) {
        let mut refs = self.registry_refs.borrow_mut();
        refs.push(plugin_name.to_string());
    }

    pub fn clear_plugin_ref(&self, plugin_name: &str) {
        let mut refs = self.registry_refs.borrow_mut();
        refs.retain(|name| name != plugin_name);
    }

    pub fn get_registered_plugins(&self) -> Vec<String> {
        self.registry_refs.borrow().clone()
    }
}

#[derive(Clone)]
pub enum LuaCommand {
    Reload {
        response: mpsc::Sender<Result<()>>,
    },
    GetPluginList {
        response: mpsc::Sender<Vec<(String, bool)>>,
    },
    EnablePlugin {
        name: String,
        response: mpsc::Sender<Result<bool>>,
    },
    DisablePlugin {
        name: String,
        response: mpsc::Sender<Result<bool>>,
    },
    ReloadPlugin {
        name: String,
        response: mpsc::Sender<Result<bool>>,
    },
    GetPluginInfo {
        name: String,
        response: mpsc::Sender<Option<PluginInfo>>,
    },
    TriggerEvent {
        event_type: String,
        event_data: String,
    },
}

pub async fn run_lua_worker(
    mut rx: Receiver<LuaCommand>,
    tx: Sender<LuaCommand>,
    data_dir: String,
) {
    let data_path = PathBuf::from(data_dir);

    init_event_sender(tx);

    let manager = match LuaManager::new(&data_path) {
        Ok(m) => Mutex::new(m),
        Err(e) => {
            eprintln!("Failed to initialize LuaManager: {}", e);
            return;
        }
    };

    {
        let mut lock = manager.lock().unwrap();

        if let Err(e) = lock.runtime.init_api() {
            eprintln!("Failed to initialize Lua API: {}", e);
            return;
        }

        lock.initialized = true;

        if let Err(e) = lock.runtime.discover_plugins() {
            eprintln!("Failed to discover plugins at startup: {}", e);
        }
    }

    let config_manager_clone = {
        let lock = manager.lock().unwrap();
        lock.config_manager.clone()
    };

    {
        let mut lock = manager.lock().unwrap();
        if let Err(e) = lock.runtime.load_enabled_plugins(&config_manager_clone) {
            eprintln!("Failed to load enabled plugins at startup: {}", e);
        }
    }

    while let Ok(cmd) = rx.recv().await {
        match cmd {
            LuaCommand::Reload { response } => {
                let result = reload_lua(&manager);
                let _ = response.send(result);
            }
            LuaCommand::GetPluginList { response } => {
                let result = get_plugin_list(&manager);
                let _ = response.send(result);
            }
            LuaCommand::EnablePlugin { name, response } => {
                let result = enable_plugin(&manager, name);
                let _ = response.send(result);
            }
            LuaCommand::DisablePlugin { name, response } => {
                let result = disable_plugin(&manager, name);
                let _ = response.send(result);
            }
            LuaCommand::ReloadPlugin { name, response } => {
                let result = reload_plugin(&manager, &name);
                let _ = response.send(result);
            }
            LuaCommand::GetPluginInfo { name, response } => {
                let result = get_plugin_info(&manager, &name);
                let _ = response.send(result);
            }
            LuaCommand::TriggerEvent {
                event_type,
                event_data,
            } => {
                handle_event(&manager, &event_type, &event_data);
            }
        }
    }
}

fn reload_lua(manager: &Mutex<LuaManager>) -> Result<()> {
    let is_initialized = {
        let lock = manager.lock().unwrap();
        lock.initialized
    };

    if !is_initialized {
        return Err(anyhow!("Cannot reload: Lua runtime not initialized"));
    }

    let disable_result = {
        let mut lock = manager.lock().unwrap();

        let plugins = lock.get_registered_plugins();
        for plugin in plugins {
            lock.clear_plugin_ref(&plugin);
        }

        lock.runtime.disable_all_plugins()
    };

    if let Err(e) = disable_result {
        return Err(anyhow!("Failed to disable plugins: {}", e));
    }

    let discovery_result = {
        let mut lock = manager.lock().unwrap();
        lock.runtime.discover_plugins()
    };

    if let Err(e) = discovery_result {
        return Err(anyhow!("Failed to rediscover plugins: {}", e));
    }

    let config_manager_clone = {
        let lock = manager.lock().unwrap();
        lock.config_manager.clone()
    };

    let result = {
        let mut lock = manager.lock().unwrap();
        lock.runtime.load_enabled_plugins(&config_manager_clone)
    };

    if let Err(e) = result {
        return Err(anyhow!("Failed to reload enabled plugins: {}", e));
    }
    Ok(())
}

fn get_plugin_list(manager: &Mutex<LuaManager>) -> Vec<(String, bool)> {
    match manager.lock() {
        Ok(lock) => lock
            .runtime
            .plugins
            .iter()
            .map(|(name, plugin)| (name.clone(), plugin.enabled))
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn reload_plugin(manager: &Mutex<LuaManager>, name: &str) -> Result<bool> {
    let is_initialized = {
        match manager.lock() {
            Ok(lock) => lock.initialized,
            Err(_) => return Err(anyhow!("Failed to acquire lock")),
        }
    };

    if !is_initialized {
        return Err(anyhow!("Cannot reload plugin: Lua runtime not initialized"));
    }

    match manager.lock() {
        Ok(mut lock) => {
            lock.clear_plugin_ref(name);

            let result = lock.runtime.reload_plugin(name);

            if result.is_ok() {
                lock.register_plugin_ref(name);
            } else {
                println!("Failed to reload plugin {}: {:?}", name, result);
            }

            result
        }
        Err(_) => Err(anyhow!("Failed to acquire lock")),
    }
}

fn get_plugin_info(manager: &Mutex<LuaManager>, name: &str) -> Option<PluginInfo> {
    let is_initialized = match manager.lock() {
        Ok(lock) => lock.initialized,
        Err(_) => return None,
    };

    if !is_initialized {
        return None;
    }

    match manager.lock() {
        Ok(lock) => lock.runtime.plugins.get(name).map(|plugin| {
            (
                plugin.manifest.name.clone(),
                plugin.manifest.description.clone(),
                plugin.manifest.version.clone(),
                plugin.manifest.author.clone(),
                plugin.enabled,
                plugin.file_path.clone(),
            )
        }),
        Err(_) => None,
    }
}

fn enable_plugin(manager: &Mutex<LuaManager>, name: String) -> Result<bool> {
    let is_initialized = {
        match manager.lock() {
            Ok(lock) => lock.initialized,
            Err(_) => return Err(anyhow!("Failed to acquire lock")),
        }
    };

    if !is_initialized {
        return Err(anyhow!("Cannot enable plugin: Lua runtime not initialized"));
    }

    let config_result = {
        match manager.lock() {
            Ok(mut lock) => lock.config_manager.enable_plugin(name.clone()),
            Err(_) => return Err(anyhow!("Failed to acquire lock")),
        }
    };

    let added_to_config = match config_result {
        Ok(result) => result,
        Err(e) => return Err(anyhow!("Failed to enable plugin in config: {}", e)),
    };

    let runtime_result = {
        match manager.lock() {
            Ok(mut lock) => {
                lock.register_plugin_ref(&name);

                let result = lock.runtime.enable_plugin(&name);

                if result.is_err() {
                    println!("Failed to enable plugin {}: {:?}", &name, result);
                }

                result
            }
            Err(_) => return Err(anyhow!("Failed to acquire lock")),
        }
    };

    if let Err(e) = runtime_result {
        if let Ok(lock) = manager.lock() {
            lock.clear_plugin_ref(&name);
        }
        return Err(anyhow!("Failed to enable plugin in runtime: {}", e));
    }

    Ok(added_to_config)
}

static INIT_EVENT_SENDER: Once = Once::new();
pub static mut EVENT_SENDER: Option<Sender<LuaCommand>> = None;

pub fn send_event_command(command: LuaCommand) -> Result<()> {
    unsafe {
        #[allow(static_mut_refs)]
        if let Some(sender) = &EVENT_SENDER {
            sender
                .send(command)
                .map_err(|_| anyhow!("Failed to send command to Lua worker"))?;
            Ok(())
        } else {
            Err(anyhow!("Event sender not initialized"))
        }
    }
}

fn init_event_sender(tx: Sender<LuaCommand>) {
    INIT_EVENT_SENDER.call_once(|| unsafe {
        EVENT_SENDER = Some(tx);
    });
}

fn handle_event(manager: &Mutex<LuaManager>, event_type: &str, event_data: &str) {
    match manager.lock() {
        Ok(lock) => {
            if !lock.initialized {
                return;
            }

            match event_type {
                "player_join" => {
                    if let Err(e) =
                        events::player_join::trigger_event(&lock.runtime.lua, event_data)
                    {
                        log::error!("Error triggering player_join event: {}", e);
                    }
                }
                "player_leave" => {
                    if let Err(e) =
                        events::player_leave::trigger_event(&lock.runtime.lua, event_data)
                    {
                        log::error!("Error triggering player_leave event: {}", e);
                    }
                }
                "player_chat" => {
                    if let Err(e) =
                        events::player_chat::trigger_event(&lock.runtime.lua, event_data)
                    {
                        log::error!("Error triggering player_chat event: {}", e);
                    }
                }
                "block_place" => {
                    if let Err(e) =
                        events::block_place::trigger_event(&lock.runtime.lua, event_data)
                    {
                        log::error!("Error triggering block_place event: {}", e);
                    }
                }
                "block_break" => {
                    if let Err(e) =
                        events::block_break::trigger_event(&lock.runtime.lua, event_data)
                    {
                        log::error!("Error triggering block_break event: {}", e);
                    }
                }
                _ => {
                    log::warn!("Unknown event type: {}", event_type);
                }
            }
        }
        Err(e) => {
            log::error!("Failed to acquire lock for event handling: {:?}", e);
        }
    }
}

fn disable_plugin(manager: &Mutex<LuaManager>, name: String) -> Result<bool> {
    let is_initialized = {
        match manager.lock() {
            Ok(lock) => lock.initialized,
            Err(_) => return Err(anyhow!("Failed to acquire lock")),
        }
    };

    if !is_initialized {
        return Err(anyhow!(
            "Cannot disable plugin: Lua runtime not initialized"
        ));
    }

    let runtime_result = {
        match manager.lock() {
            Ok(mut lock) => {
                lock.clear_plugin_ref(&name);

                let result = lock.runtime.disable_plugin(&name);

                if result.is_err() {
                    println!("Failed to disable plugin {}: {:?}", &name, result);
                }

                result
            }
            Err(_) => return Err(anyhow!("Failed to acquire lock")),
        }
    };

    if let Err(e) = runtime_result {
        return Err(anyhow!("Failed to disable plugin in runtime: {}", e));
    }

    let config_result = {
        match manager.lock() {
            Ok(mut lock) => lock.config_manager.disable_plugin(&name),
            Err(_) => return Err(anyhow!("Failed to acquire lock")),
        }
    };

    match config_result {
        Ok(removed) => Ok(removed),
        Err(e) => Err(anyhow!("Failed to disable plugin in config: {}", e)),
    }
}
