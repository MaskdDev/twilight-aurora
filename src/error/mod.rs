use crate::context::CommandContext;
use std::fmt::Debug;
use thiserror::Error;

/// A result from the aurora crate.
pub type AuroraResult<T> = Result<T, AuroraError>;

/// An error from the aurora crate.
#[derive(Error, Debug)]
pub enum AuroraError {
    #[error("Could not mention command '{0}' ({1})")]
    CommandMentionError(String, String),

    #[error("Could not find command handler for command with name '{0}'")]
    CommandHandlerNotFound(String),

    #[error("Could not determine the shard configuration to start up with: {0}")]
    ShardRecommendationError(#[from] twilight_gateway::error::StartRecommendedError),

    #[error("A twilight HTTP error occurred: {0}")]
    TwilightHttpError(#[from] twilight_http::Error),

    #[error("Could not deserialize twilight body: {0}")]
    TwilightHttpDeserializeError(#[from] twilight_http::response::DeserializeBodyError),
}

/// A result from the aurora crate, as a result of handling an event.
pub type AuroraRuntimeResult<T, E> = Result<T, AuroraRuntimeError<T, E>>;

/// An error from the aurora crate, as a result of handling an event.
#[derive(Error)]
pub enum AuroraRuntimeError<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    #[error("An error occurred during the handling of a command.")]
    Command(CommandContext<T, E>, E),

    #[error("An error occurred during the handling of {0}: {1}")]
    EventHandler(&'static str, E),
}
