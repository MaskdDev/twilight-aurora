use crate::Client;
use crate::error::AuroraResult;
use crate::framework::CommandFramework;
use crate::gateway::ShardHandle;
use std::fmt::Debug;
use std::sync::Arc;
use twilight_gateway::Shard;
use twilight_http::Client as HttpClient;
use twilight_http::client::InteractionClient;
use twilight_model::oauth::Application;

pub struct AppContext<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// The data for this application.
    pub data: Arc<T>,

    /// The application's HTTP client.
    pub http: Arc<HttpClient>,

    /// The bot's command framework.
    pub framework: Arc<CommandFramework<T, E>>,

    /// The application's current user application.
    application: Arc<Application>,

    /// A handle to the shard servicing this task.
    pub shard_info: ShardHandle,
}

impl<T, E> Clone for AppContext<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            http: Arc::clone(&self.http),
            framework: Arc::clone(&self.framework),
            application: Arc::clone(&self.application),
            shard_info: self.shard_info.clone(),
        }
    }
}

impl<T, E> AppContext<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// Initialise the application context given the client and shard executing an event.
    pub fn from(client: &Client<T, E>, shard: &Shard) -> Self {
        Self {
            data: Arc::clone(&client.data),
            http: Arc::clone(&client.http),
            framework: Arc::clone(&client.framework),
            application: Arc::clone(&client.application),
            shard_info: ShardHandle::from(shard),
        }
    }

    /// Mention a command, given its name.
    ///
    /// Only the parent command name is verified to exist.
    ///
    /// If the command name isn't found or couldn't be mentioned, an error is returned.
    pub fn mention_command(&self, command_name: &str) -> AuroraResult<String> {
        self.framework.mention_command(command_name)
    }

    /// Get the interaction client from this context's HTTP client and application ID.
    pub fn interaction_client(&self) -> InteractionClient<'_> {
        self.http.interaction(self.application.id)
    }
}
