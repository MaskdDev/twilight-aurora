use crate::commands::{Command, TwilightCommand};
use crate::context::AppContext;
use crate::model::reply::CreateReply;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use twilight_http::client::InteractionClient;
use twilight_http::{Error, Response};
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::application_command::CommandData;
use twilight_model::channel::Message;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseType};

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

    /// Whether an initial response has been made for this command.
    has_responded: Arc<AtomicBool>,

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
            has_responded: Arc::clone(&self.has_responded),
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
            has_responded: Arc::new(AtomicBool::from(false)),
            command_data: command_data.into(),
            locale,
        }
    }
    /// Get the interaction client from this context's HTTP client and application ID.
    pub fn interaction_client(&self) -> InteractionClient<'_> {
        self.app_ctx.interaction_client()
    }

    /// Send a text-only reply in response to this command.
    ///
    /// Sends an initial response if no response to this command has been sent, or
    /// edits an existing response if one has been made.
    pub async fn say(&self, content: impl Into<String>) -> Result<(), Error> {
        self.send(CreateReply::new().content(content)).await
    }

    /// Send a text-only reply in response to this command, ephemerally.
    ///
    /// Sends an initial response if no response to this command has been sent, or
    /// edits an existing response if one has been made (will not be ephemeral unless the previous
    /// response was already ephemeral).
    pub async fn say_ephemeral(&self, content: impl Into<String>) -> Result<(), Error> {
        self.send(CreateReply::new().content(content).ephemeral(true))
            .await
    }

    /// Send a reply in response to this command.
    ///
    /// Sends an initial response if no response to this command has been sent, or
    /// edits an existing response if one has been made.
    pub async fn send(&self, reply: CreateReply) -> Result<(), Error> {
        // Check if a response has been made
        if self
            .has_responded
            .compare_exchange(false, true, Ordering::Release, Ordering::Acquire)
            .is_ok()
        {
            self.send_response(reply).await?;
        } else {
            self.edit_response(reply).await?;
        }

        // Return Ok
        Ok(())
    }

    /// Send a reply in response to this command.
    pub async fn send_response(&self, reply: CreateReply) -> Result<(), Error> {
        // Create interaction response
        let response = InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(reply.build()),
        };

        // Respond to interaction
        self.interaction_client()
            .create_response(self.interaction.id, &self.interaction.token, &response)
            .await?;

        // Return Ok
        Ok(())
    }

    /// Update an initial interaction response.
    pub async fn edit_response(&self, reply: CreateReply) -> Result<Response<Message>, Error> {
        // Get interaction client
        let client = self.interaction_client();

        // Respond to interaction
        let message = reply
            .apply(client.update_response(&self.interaction.token))
            .await?;

        // Return message response
        Ok(message)
    }

    /// Defer a response to this command.
    ///
    /// Does nothing if this command has already been responded to.
    pub async fn defer(&self, ephemeral: bool) -> Result<(), Error> {
        // Set the has responded flag to true, if not done already.
        if self
            .has_responded
            .compare_exchange(false, true, Ordering::Release, Ordering::Acquire)
            .is_err()
        {
            // Already responded to interaction, exit early.
            return Ok(());
        }

        // Create interaction response
        let response = InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(CreateReply::new().ephemeral(ephemeral).build()),
        };

        // Respond to interaction
        self.interaction_client()
            .create_response(self.interaction.id, &self.interaction.token, &response)
            .await?;

        // Return Ok
        Ok(())
    }
}
