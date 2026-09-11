mod handlers;
mod runner;

use crate::client::runner::shard_runner;
use crate::commands::TwilightCommand;
use crate::context::AppContext;
use crate::error::{AuroraError, AuroraResult, AuroraRuntimeError};
use crate::framework::CommandFramework;
use derive_builder::Builder;
use futures_util::future::BoxFuture;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::task::JoinSet;
use twilight_gateway::{ConfigBuilder, EventTypeFlags, MessageSender, Shard, create_recommended};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_model::gateway::event::Event;
use twilight_model::oauth::Application;

// Types for client callbacks.
type ErrorHandler<T, E> = fn(AuroraRuntimeError<T, E>) -> BoxFuture<'static, ()>;
type EventHandler<T, E> = fn(Event, AppContext<T, E>) -> BoxFuture<'static, Result<(), E>>;

/// The client struct representing a discord application.
#[derive(Builder)]
#[builder(pattern = "owned", vis = "pub")]
pub struct Client<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// The application's token,
    #[builder(setter(into))]
    token: Arc<String>,

    /// The application's intents.
    intents: Intents,

    /// The events the application should listen for.
    #[builder(default = "EventTypeFlags::all()")]
    event_filter: EventTypeFlags,

    /// The application's data struct.
    #[builder(setter(into))]
    pub(crate) data: Arc<T>,

    /// The application's HTTP client.
    #[builder(setter(into))]
    pub(crate) http: Arc<HttpClient>,

    /// The application's command framework.
    #[builder(setter(into))]
    pub(crate) framework: Arc<CommandFramework<T, E>>,

    /// The error handler for this client.
    #[builder(default = "handlers::default_on_error::<T, E>")]
    on_error: ErrorHandler<T, E>,

    /// The application's event handler. This runs for all events apart from InteractionCreate,
    /// which is handled by the framework.
    event_handler: EventHandler<T, E>,

    /// The application's shard messengers.
    #[builder(setter(skip))]
    shard_messengers: Arc<Vec<MessageSender>>,

    /// The application's current user application.
    #[builder(setter(into))]
    #[builder(vis = "pub(crate)")]
    pub(crate) application: Arc<Application>,
}

impl<T, E> Clone for Client<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    fn clone(&self) -> Self {
        Self {
            token: Arc::clone(&self.token),
            intents: self.intents,
            event_filter: self.event_filter,
            data: Arc::clone(&self.data),
            http: Arc::clone(&self.http),
            framework: Arc::clone(&self.framework),
            on_error: self.on_error,
            event_handler: self.event_handler,
            shard_messengers: Arc::clone(&self.shard_messengers),
            application: Arc::clone(&self.application),
        }
    }
}

impl<T, E> Client<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// Establish gateway connection(s) with discord and start listening for events.
    ///
    /// This will start receiving events in a loop and start dispatching the events to your registered handlers.
    ///
    /// This will retrieve an automatically determined number of shards to use from the API - determined by Discord - and then open a number of shards equivalent to that amount.
    pub async fn start(&mut self) -> AuroraResult<()> {
        // Get shards to spawn
        let shards = self.initialise_shards().await?;

        // Start shard runners
        let mut shard_runners = JoinSet::new();
        for shard in shards {
            tracing::info!("Spawning shard {}", shard.id());
            shard_runners.spawn(shard_runner(shard, self.clone()));
        }

        // Wait for all shard runners to exit
        while shard_runners.join_next().await.is_some() {}

        // Return Ok
        Ok(())
    }

    /// Initialise the shards to create for this client, storing their message senders in this client's shard vector.
    ///
    /// Returns the shards to be created.
    pub async fn initialise_shards(&mut self) -> AuroraResult<Vec<Shard>> {
        // Create per-shard config
        let config = ConfigBuilder::new(self.token.to_string(), self.intents).build();

        // Create recommended shards
        let shards = create_recommended(&self.http, config, |_, builder| builder.build())
            .await?
            .collect::<Vec<Shard>>();

        // Wrap shard message senders in an arc and store them internally.
        self.shard_messengers = Arc::new(shards.iter().map(Shard::sender).collect());

        // Return shards
        Ok(shards)
    }
}

impl<T, E> ClientBuilder<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// Create a new client builder using the provided bot token, intents and framework.
    ///
    /// Registers all commands passed in the framework.
    pub async fn new(
        token: String,
        intents: Intents,
        mut framework: CommandFramework<T, E>,
    ) -> Result<Self, AuroraError> {
        // Create HTTP client
        let client = HttpClient::new(token.clone());

        // Get information about the current user application.
        let application = client.current_user_application().await?.model().await?;

        // Register all commands
        let commands = client
            .interaction(application.id)
            .set_global_commands(
                &framework
                    .commands
                    .iter()
                    .map(|command| TwilightCommand::clone(&command.command))
                    .collect::<Vec<_>>(),
            )
            .await?
            .model()
            .await?;

        // Store command IDs in framework
        for command in commands {
            if let Some(local_command) = framework
                .commands
                .iter_mut()
                .find(|c| c.command.name == command.name)
            {
                let inner_command = Arc::make_mut(&mut local_command.command);
                inner_command.id = command.id
            }
        }

        // Return client builder
        Ok(Self::create_empty()
            .token(token)
            .intents(intents)
            .http(client)
            .framework(framework)
            .application(application))
    }
}
