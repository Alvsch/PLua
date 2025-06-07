use async_trait::async_trait;
use pumpkin::{
    command::{
        CommandExecutor, CommandSender,
        args::{Arg, ConsumedArgs, message::MsgArgConsumer},
        dispatcher::CommandError,
        tree::{
            CommandTree,
            builder::{argument, literal},
        },
    },
    server::Server,
};
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::lua;

const NAMES: [&str; 1] = ["plua"];
const DESCRIPTION: &str = "Manage Lua plugins for the Pumpkin server";

const ARG_PLUGIN_NAME: &str = "plugin_name";

pub const PERMISSION_NODE: &str = "plua:command.plua";

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("list").execute(ListPluginsExecutor {}))
        .then(
            literal("enable")
                .then(argument(ARG_PLUGIN_NAME, MsgArgConsumer).execute(EnablePluginExecutor {})),
        )
        .then(
            literal("disable")
                .then(argument(ARG_PLUGIN_NAME, MsgArgConsumer).execute(DisablePluginExecutor {})),
        )
        .then(
            literal("reload")
                .execute(ReloadAllExecutor {})
                .then(argument(ARG_PLUGIN_NAME, MsgArgConsumer).execute(ReloadPluginExecutor {})),
        )
        .then(
            literal("info")
                .then(argument(ARG_PLUGIN_NAME, MsgArgConsumer).execute(PluginInfoExecutor {})),
        )
}

struct ListPluginsExecutor {}

#[async_trait]
impl CommandExecutor for ListPluginsExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _: &Server,
        _: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let plugins = lua::get_plugin_list();

        if plugins.is_empty() {
            sender
                .send_message(
                    TextComponent::text("No Lua plugins found.").color_named(NamedColor::Yellow),
                )
                .await;
            return Ok(());
        }

        sender
            .send_message(TextComponent::text("=== Lua Plugins ===").color_named(NamedColor::Gold))
            .await;

        for (name, enabled) in plugins {
            let status_color = if enabled {
                NamedColor::Green
            } else {
                NamedColor::Red
            };
            let status_text = if enabled { "Enabled" } else { "Disabled" };

            sender
                .send_message(
                    TextComponent::text(format!("- {} [", name))
                        .add_text(status_text)
                        .color_named(status_color)
                        .add_child(TextComponent::text("]").color_named(status_color)),
                )
                .await;
        }

        Ok(())
    }
}

struct EnablePluginExecutor {}

#[async_trait]
impl CommandExecutor for EnablePluginExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Msg(plugin_name)) = args.get(ARG_PLUGIN_NAME) else {
            return Err(CommandError::InvalidConsumption(Some(
                ARG_PLUGIN_NAME.into(),
            )));
        };

        match lua::enable_plugin(plugin_name) {
            Ok(true) => {
                sender
                    .send_message(
                        TextComponent::text(format!("Plugin '{}' has been enabled.", plugin_name))
                            .color_named(NamedColor::Green),
                    )
                    .await;
            }
            Ok(false) => {
                sender
                    .send_message(
                        TextComponent::text(format!(
                            "Plugin '{}' is already enabled.",
                            plugin_name
                        ))
                        .color_named(NamedColor::Yellow),
                    )
                    .await;
            }
            Err(e) => {
                sender
                    .send_message(
                        TextComponent::text(format!(
                            "Failed to enable plugin '{}': {}",
                            plugin_name, e
                        ))
                        .color_named(NamedColor::Red),
                    )
                    .await;
            }
        }

        Ok(())
    }
}

struct DisablePluginExecutor {}

#[async_trait]
impl CommandExecutor for DisablePluginExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Msg(plugin_name)) = args.get(ARG_PLUGIN_NAME) else {
            return Err(CommandError::InvalidConsumption(Some(
                ARG_PLUGIN_NAME.into(),
            )));
        };

        match lua::disable_plugin(plugin_name) {
            Ok(true) => {
                sender
                    .send_message(
                        TextComponent::text(format!("Plugin '{}' has been disabled.", plugin_name))
                            .color_named(NamedColor::Green),
                    )
                    .await;
            }
            Ok(false) => {
                sender
                    .send_message(
                        TextComponent::text(format!(
                            "Plugin '{}' is already disabled.",
                            plugin_name
                        ))
                        .color_named(NamedColor::Yellow),
                    )
                    .await;
            }
            Err(e) => {
                sender
                    .send_message(
                        TextComponent::text(format!(
                            "Failed to disable plugin '{}': {}",
                            plugin_name, e
                        ))
                        .color_named(NamedColor::Red),
                    )
                    .await;
            }
        }

        Ok(())
    }
}

struct ReloadAllExecutor {}

#[async_trait]
impl CommandExecutor for ReloadAllExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _: &Server,
        _: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        match lua::reload() {
            Ok(_) => {
                sender
                    .send_message(
                        TextComponent::text("All Lua plugins have been reloaded.")
                            .color_named(NamedColor::Green),
                    )
                    .await;
            }
            Err(e) => {
                sender
                    .send_message(
                        TextComponent::text(format!("Failed to reload plugins: {}", e))
                            .color_named(NamedColor::Red),
                    )
                    .await;
            }
        }

        Ok(())
    }
}

struct ReloadPluginExecutor {}

#[async_trait]
impl CommandExecutor for ReloadPluginExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Msg(plugin_name)) = args.get(ARG_PLUGIN_NAME) else {
            return Err(CommandError::InvalidConsumption(Some(
                ARG_PLUGIN_NAME.into(),
            )));
        };

        match lua::reload_plugin(plugin_name) {
            Ok(true) => {
                sender
                    .send_message(
                        TextComponent::text(format!("Plugin '{}' has been reloaded.", plugin_name))
                            .color_named(NamedColor::Green),
                    )
                    .await;
            }
            Ok(false) => {
                sender
                    .send_message(
                        TextComponent::text(format!("Plugin '{}' not found.", plugin_name))
                            .color_named(NamedColor::Yellow),
                    )
                    .await;
            }
            Err(e) => {
                sender
                    .send_message(
                        TextComponent::text(format!(
                            "Failed to reload plugin '{}': {}",
                            plugin_name, e
                        ))
                        .color_named(NamedColor::Red),
                    )
                    .await;
            }
        }

        Ok(())
    }
}

struct PluginInfoExecutor {}

#[async_trait]
impl CommandExecutor for PluginInfoExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let Some(Arg::Msg(plugin_name)) = args.get(ARG_PLUGIN_NAME) else {
            return Err(CommandError::InvalidConsumption(Some(
                ARG_PLUGIN_NAME.into(),
            )));
        };

        if let Some((name, description, version, author, enabled, file_path)) =
            lua::get_plugin_info(plugin_name)
        {
            sender
                .send_message(
                    TextComponent::text(format!("=== {} ===", name)).color_named(NamedColor::Gold),
                )
                .await;

            sender
                .send_message(
                    TextComponent::text("Description: ")
                        .color_named(NamedColor::Yellow)
                        .add_text(description),
                )
                .await;

            sender
                .send_message(
                    TextComponent::text("Version: ")
                        .color_named(NamedColor::Yellow)
                        .add_text(version),
                )
                .await;

            sender
                .send_message(
                    TextComponent::text("Author: ")
                        .color_named(NamedColor::Yellow)
                        .add_text(author),
                )
                .await;

            let status_color = if enabled {
                NamedColor::Green
            } else {
                NamedColor::Red
            };
            let status_text = if enabled { "Enabled" } else { "Disabled" };

            sender
                .send_message(
                    TextComponent::text("Status: ")
                        .color_named(NamedColor::Yellow)
                        .add_text(status_text)
                        .color_named(status_color),
                )
                .await;

            let path_str = file_path.to_string_lossy().into_owned();
            sender
                .send_message(
                    TextComponent::text("File: ")
                        .color_named(NamedColor::Yellow)
                        .add_text(path_str),
                )
                .await;
        } else {
            sender
                .send_message(
                    TextComponent::text(format!("Plugin '{}' not found.", plugin_name))
                        .color_named(NamedColor::Red),
                )
                .await;
        }

        Ok(())
    }
}
