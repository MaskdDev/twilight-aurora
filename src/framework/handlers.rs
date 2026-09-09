use crate::context::CommandContext;
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use std::fmt::Debug;

/// The default pre-command handler for a command framework.
pub(crate) fn default_pre_command<T, E>(_: CommandContext<T, E>) -> BoxFuture<'static, ()>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    async {}.boxed()
}

/// The default post-command handler for a command framework.
pub(crate) fn default_post_command<T, E>(_: CommandContext<T, E>) -> BoxFuture<'static, ()>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    async {}.boxed()
}

/// The default command check handler for a command framework.
pub(crate) fn default_command_check<T, E>(
    _: CommandContext<T, E>,
) -> BoxFuture<'static, Result<bool, E>>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    async { Ok(true) }.boxed()
}
