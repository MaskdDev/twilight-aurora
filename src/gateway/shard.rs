use std::time::Duration;
use twilight_gateway::{MessageSender, Shard};
use twilight_model::gateway::ShardId;

/// A struct representing the shard performing a given task.
#[derive(Clone)]
pub struct ShardHandle {
    /// The average latency of the current shard.
    pub average_latency: Option<Duration>,

    /// The shard ID of the current shard.
    pub shard_id: ShardId,

    /// A messenger to the current shard.
    pub shard_messenger: MessageSender,
}

impl From<&Shard> for ShardHandle {
    fn from(value: &Shard) -> Self {
        ShardHandle {
            average_latency: value.latency().average(),
            shard_id: value.id(),
            shard_messenger: value.sender(),
        }
    }
}
