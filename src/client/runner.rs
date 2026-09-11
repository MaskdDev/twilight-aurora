use crate::Client;
use crate::context::{AppContext, CommandContext};
use crate::error::{AuroraError, AuroraRuntimeError};
use std::fmt::Debug;
use std::mem;
use twilight_gateway::{Shard, StreamExt};
use twilight_model::application::interaction::InteractionData;
use twilight_model::gateway::event::Event;

/// The shard runner for a shard, iterating over events and spawning worker threads for them.
pub(crate) async fn shard_runner<T, E>(mut shard: Shard, client: Client<T, E>)
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    while let Some(item) = shard.next_event(client.event_filter).await {
        // Receive event
        let Ok(event) = item else {
            tracing::warn!(source = ?item.unwrap_err(), "error receiving event");
            continue;
        };

        // Create application context
        let ctx = AppContext::from(&client, &shard);

        // Spawn a new task to handle the event
        tokio::spawn(handle_event(event, ctx, client.clone()));
    }
}

/// The handler that handles a specific event received by a shard.
async fn handle_event<T, E>(
    event: Event,
    ctx: AppContext<T, E>,
    client: Client<T, E>,
) -> Result<(), AuroraError>
where
    T: Send + Sync + 'static,
    E: Send + Sync + Debug + 'static,
{
    // Check event type
    let leftover_event = match event {
        Event::InteractionCreate(mut interaction) => {
            // Check if the command is a registered command
            let is_registered_command = match &interaction.data {
                Some(InteractionData::ApplicationCommand(data)) => {
                    ctx.framework.get_command(&data.name).is_some()
                }
                _ => false,
            };

            // If command is a registered command, pass on to framework handler
            if is_registered_command {
                // Extract the command data from the interaction.
                let data = match mem::take(&mut interaction.data) {
                    Some(InteractionData::ApplicationCommand(data)) => *data,
                    _ => panic!("Data was just verified to belong to an application command."),
                };

                // Get command
                let command = ctx
                    .framework
                    .get_command(&data.name)
                    .expect("This was just verified to belong to an existing command.");

                // Create command context
                let cmd_ctx = CommandContext::new(ctx.clone(), command, interaction.0, data);

                // Run command check
                if let Ok(check) = (ctx.framework.command_check)(cmd_ctx.clone()).await
                    && check
                {
                    // Run pre-command hook
                    (ctx.framework.pre_command)(cmd_ctx.clone()).await;

                    // Run command handler
                    match (command.handler)(cmd_ctx.clone()).await {
                        Ok(_) => {}
                        Err(error) => {
                            (client.on_error)(AuroraRuntimeError::Command(cmd_ctx.clone(), error))
                                .await
                        }
                    }

                    // Run post-command hook
                    (ctx.framework.post_command)(cmd_ctx.clone()).await;
                }

                // Return no further event
                None
            } else {
                Some(Event::InteractionCreate(interaction))
            }
        }
        _ => Some(event),
    };

    // If there is a leftover event, process it using the user's handler.
    if let Some(event) = leftover_event {
        // Get event name
        let event_name = event.kind().name().unwrap_or("unknown event");

        // Run provided event handler, handling any potential error.
        match (client.event_handler)(event, ctx).await {
            Ok(_) => {}
            Err(error) => {
                (client.on_error)(AuroraRuntimeError::EventHandler(event_name, error)).await
            }
        }
    }

    Ok(())
}
