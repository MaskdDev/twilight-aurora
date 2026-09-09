use crate::commands::{Command, TwilightCommand};
use crate::context::AppContext;
use std::fmt::Debug;
use std::sync::Arc;
use twilight_http::client::InteractionClient;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::CommandData;

pub struct CommandContext<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// The application context for this application.
    pub app_ctx: AppContext<T, E>,

    /// The twilight command object for this command.
    pub command: Arc<TwilightCommand>,

    /// The interaction that triggered this command, with its command data stripped out.
    pub interaction: Arc<Interaction>,

    /// The locale of this interaction.
    pub locale: Option<String>,

    /// The command data of this command.
    pub command_data: Arc<CommandData>,
}

impl<T, E> Clone for CommandContext<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    fn clone(&self) -> Self {
        Self {
            app_ctx: self.app_ctx.clone(),
            command: Arc::clone(&self.command),
            interaction: Arc::clone(&self.interaction),
            command_data: Arc::clone(&self.command_data),
            locale: self.locale.clone(),
        }
    }
}

impl<T, E> CommandContext<T, E>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    /// Initialise a command context struct.
    pub fn new(
        ctx: AppContext<T, E>,
        command: &Command<T, E>,
        interaction: Interaction,
        command_data: CommandData,
    ) -> Self {
        let locale = interaction.locale.clone();

        Self {
            app_ctx: ctx,
            command: Arc::clone(&command.command),
            interaction: interaction.into(),
            command_data: command_data.into(),
            locale,
        }
    }
    /// Get the interaction client from this context's HTTP client and application ID.
    pub fn interaction_client(&self) -> InteractionClient<'_> {
        self.app_ctx.interaction_client()
    }
}
