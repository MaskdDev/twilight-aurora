mod handlers;

use crate::commands::Command;
use crate::context::CommandContext;
use crate::error::{AuroraError, AuroraResult};
use derive_builder::Builder;
use futures_util::future::BoxFuture;
use std::convert::Infallible;
use std::fmt::Debug;

#[derive(Builder)]
#[builder(pattern = "owned", vis = "pub")]
#[builder(build_fn(error = "Infallible"))]
pub struct CommandFramework<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// The top-level commands for this framework.
    #[builder(setter(into), default)]
    pub commands: Vec<Command<T, E>>,

    /// The pre-command handler for this framework.
    #[builder(default = "handlers::default_pre_command")]
    pub(crate) pre_command: fn(CommandContext<T, E>) -> BoxFuture<'static, ()>,

    /// The post-command handler for this framework.
    #[builder(default = "handlers::default_post_command")]
    pub(crate) post_command: fn(CommandContext<T, E>) -> BoxFuture<'static, ()>,

    /// A check run before every command. The command is only executed if this returns true.
    #[builder(default = "handlers::default_command_check::<T, E>")]
    pub(crate) command_check: fn(CommandContext<T, E>) -> BoxFuture<'static, Result<bool, E>>,
}

impl<T, E> CommandFramework<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// Mention a command, given its name.
    ///
    /// Only the parent command name is verified to exist.
    ///
    /// If the command name isn't found or couldn't be mentioned, an error is returned.
    pub fn mention_command(&self, command_name: &str) -> AuroraResult<String> {
        // Get command name segments
        let mut segments = command_name.split_whitespace();

        // Get command parent name
        let parent_name = segments.next().ok_or_else(|| {
            AuroraError::CommandMentionError(
                command_name.to_string(),
                String::from("command name empty"),
            )
        })?;

        // Find command
        let command = self
            .commands
            .iter()
            .find(|c| c.command.name == parent_name)
            .ok_or_else(|| {
                AuroraError::CommandMentionError(
                    command_name.to_string(),
                    String::from("command not found"),
                )
            })?;

        // Get command ID
        let command_id = command.command.id.ok_or_else(|| {
            AuroraError::CommandMentionError(
                command_name.to_string(),
                String::from("command ID not found"),
            )
        })?;

        // Get child command name
        let child_name: Vec<&str> = segments.collect();

        // Return command mention
        if child_name.is_empty() {
            Ok(format!("</{}:{}>", parent_name, command_id))
        } else {
            Ok(format!(
                "</{} {}:{}>",
                parent_name,
                child_name.join(" "),
                command_id
            ))
        }
    }

    /// Get a command given its name.
    pub fn get_command(&self, command_name: &str) -> Option<&Command<T, E>> {
        // Get command name segments
        let mut segments = command_name.split_whitespace();

        // Get command parent name
        let parent_name = segments.next();

        // Return command
        parent_name
            .map(|name| self.commands.iter().find(|c| c.command.name == name))
            .flatten()
    }
}
