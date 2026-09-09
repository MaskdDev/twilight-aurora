use twilight_model::id::Id;
use twilight_model::id::marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker};

/// An alias for the discord user ID type.
pub type UserId = Id<UserMarker>;

/// An alias for the discord channel ID type.
pub type ChannelId = Id<ChannelMarker>;

/// An alias for the discord guild ID type.
pub type GuildId = Id<GuildMarker>;

/// An alias for the discord role ID type.
pub type RoleId = Id<RoleMarker>;
