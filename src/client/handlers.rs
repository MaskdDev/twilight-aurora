use crate::error::AuroraRuntimeError;
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use std::fmt::Debug;

/// The default error handler for a client.
pub(crate) fn default_on_error<T, E>(_: AuroraRuntimeError<T, E>) -> BoxFuture<'static, ()>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    async {}.boxed()
}
