use crate::context::CommandContext;
use futures_util::future::BoxFuture;
use std::fmt::Debug;
use std::sync::Arc;

/// A re-export of the twilight `Command` type.
pub type TwilightCommand = twilight_model::application::command::Command;

/// The type for a command handler for the bot.
pub type CommandHandler<T, E> = fn(CommandContext<T, E>) -> BoxFuture<'static, Result<(), E>>;

/// A struct representing a single discord command and its associated handler.
pub struct Command<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// The underlying command information for this command.
    pub command: Arc<TwilightCommand>,

    /// The command handler for this command.
    pub handler: CommandHandler<T, E>,
}
