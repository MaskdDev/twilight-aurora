use bitflags::bitflags;
use twilight_http::request::application::interaction::UpdateResponse;
use twilight_model::channel::message::{AllowedMentions, Component, Embed, MessageFlags};
use twilight_model::http::attachment::Attachment;
use twilight_model::http::interaction::InteractionResponseData;
use twilight_model::poll::Poll;

/// A reply to be sent in response to a command interaction.
#[derive(Default)]
pub struct CreateReply {
    data: InteractionResponseData,
    clear_flags: ClearFlags,
}

/// Builder helper methods, similar to [`twilight_util::builder::InteractionResponseDataBuilder`].
impl CreateReply {
    /// Create a new empty reply.
    pub const fn new() -> Self {
        Self {
            data: InteractionResponseData {
                allowed_mentions: None,
                attachments: None,
                choices: None,
                components: None,
                content: None,
                custom_id: None,
                embeds: None,
                flags: None,
                title: None,
                tts: None,
                poll: None,
            },
            clear_flags: ClearFlags::empty(),
        }
    }

    /// Build the reply, returning an [`InteractionResponseData`] struct.
    pub(crate) fn build(self) -> InteractionResponseData {
        self.data
    }

    /// Apply this reply as an update onto an [`UpdateResponse`].
    pub(crate) fn apply<'a>(
        &'a self,
        mut update_response: UpdateResponse<'a>,
    ) -> UpdateResponse<'a> {
        // Add content to update, if present.
        update_response = match &self.data.content {
            Some(content) => update_response.content(Some(content)),
            None if self.clear_flags.contains(ClearFlags::CONTENT) => update_response.content(None),
            None => update_response,
        };

        // Add embeds to update, if present.
        update_response = match &self.data.embeds {
            Some(embeds) => update_response.embeds(Some(embeds)),
            None if self.clear_flags.contains(ClearFlags::EMBEDS) => update_response.embeds(None),
            None => update_response,
        };

        // Add components to update, if present.
        update_response = match &self.data.components {
            Some(components) => update_response.components(Some(components)),
            None if self.clear_flags.contains(ClearFlags::COMPONENTS) => {
                update_response.components(None)
            }
            None => update_response,
        };

        // Add allowed mentions to update, if present.
        update_response = match &self.data.allowed_mentions {
            Some(allowed_mentions) => update_response.allowed_mentions(Some(allowed_mentions)),
            None if self.clear_flags.contains(ClearFlags::MENTIONS) => {
                update_response.allowed_mentions(None)
            }
            None => update_response,
        };

        // Add flags and attachments to update, if present.
        if let Some(flags) = self.data.flags {
            update_response = update_response.flags(flags)
        }
        if let Some(attachments) = &self.data.attachments {
            update_response = update_response.attachments(attachments)
        }

        // Return update response
        update_response
    }

    /// Set the [`AllowedMentions`] of the reply.
    ///
    /// Defaults to [`None`].
    pub fn allowed_mentions(mut self, allowed_mentions: AllowedMentions) -> Self {
        self.data.allowed_mentions = Some(allowed_mentions);

        self
    }

    /// Set this reply to clear the response's [`AllowedMentions`]. Only takes effect if this
    /// reply is being used to modify an existing response.
    ///
    /// This is undone if allowed mentions are set following this call.
    pub fn clear_allowed_mentions(mut self) -> Self {
        self.data.allowed_mentions = None;
        self.clear_flags.insert(ClearFlags::MENTIONS);
        self
    }

    /// Set the attachments of the message.
    ///
    /// Defaults to [`None`].
    pub fn attachments(mut self, attachments: impl IntoIterator<Item = Attachment>) -> Self {
        self.data.attachments = Some(attachments.into_iter().collect());
        self
    }

    /// Add a [`Component`] to the reply.
    ///
    /// Existing components are kept.
    pub fn component(mut self, component: impl Into<Component>) -> Self {
        if let Some(embeds) = &mut self.data.components {
            embeds.push(component.into())
        } else {
            self.data.components = Some(vec![component.into()]);
        }
        self
    }

    /// Set the message [`Component`]s of the reply.
    ///
    /// Defaults to [`None`].
    pub fn components(mut self, components: impl IntoIterator<Item = Component>) -> Self {
        self.data.components = Some(components.into_iter().collect());
        self
    }

    /// Set this reply to clear the response's [`Component`]s. Only takes effect if this
    /// reply is being used to modify an existing response.
    ///
    /// This is undone if components are set following this call.
    pub fn clear_components(mut self) -> Self {
        self.data.components = None;
        self.clear_flags.insert(ClearFlags::COMPONENTS);
        self
    }

    /// Set the message content of the reply.
    ///
    /// Defaults to [`None`].
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.data.content = Some(content.into());
        self
    }

    /// Set this reply to clear the response's message content. Only takes effect if this
    /// reply is being used to modify an existing response.
    ///
    /// This is undone if message content is set following this call.
    pub fn clear_content(mut self) -> Self {
        self.data.content = None;
        self.clear_flags.insert(ClearFlags::CONTENT);
        self
    }

    /// Add an [`Embed`] to the reply.
    ///
    /// Existing embeds are kept.
    pub fn embed(mut self, embed: Embed) -> Self {
        if let Some(embeds) = &mut self.data.embeds {
            embeds.push(embed)
        } else {
            self.data.embeds = Some(vec![embed]);
        }
        self
    }

    /// Set the [`Embed`]s of the reply. Overwrites any embeds that may already be present.
    ///
    /// Defaults to an empty list.
    pub fn embeds(mut self, embeds: impl IntoIterator<Item = Embed>) -> Self {
        self.data.embeds = Some(embeds.into_iter().collect());
        self
    }

    /// Set this reply to clear the response's [`Embed`]s. Only takes effect if this
    /// reply is being used to modify an existing response.
    pub fn clear_embeds(mut self) -> Self {
        self.data.embeds = None;
        self.clear_flags.insert(ClearFlags::EMBEDS);
        self
    }

    /// Set the [`MessageFlags`].
    ///
    /// The only supported flags are [`EPHEMERAL`], [`SUPPRESS_EMBEDS`] and [`IS_COMPONENTS_V2`].
    ///
    /// Defaults to [`None`].
    ///
    /// [`EPHEMERAL`]: MessageFlags::EPHEMERAL
    /// [`SUPPRESS_EMBEDS`]: MessageFlags::SUPPRESS_EMBEDS
    /// [`IS_COMPONENTS_V2`]: MessageFlags::IS_COMPONENTS_V2
    pub const fn flags(mut self, flags: MessageFlags) -> Self {
        self.data.flags = Some(flags);
        self
    }

    /// Set the reply's ephemeral status.
    pub fn ephemeral(self, ephemeral: bool) -> Self {
        if ephemeral {
            self.add_flags(MessageFlags::EPHEMERAL)
        } else {
            self.remove_flags(MessageFlags::EPHEMERAL)
        }
    }

    /// Set whether the response has text-to-speech enabled.
    ///
    /// Defaults to [`None`].
    pub const fn tts(mut self, value: bool) -> Self {
        self.data.tts = Some(value);
        self
    }

    /// Set the poll of the reply.
    pub fn poll(mut self, poll: Poll) -> Self {
        self.data.poll = Some(poll);
        self
    }

    /// Add a specific combination of flags to the reply's flags.
    pub fn add_flags(self, flags: MessageFlags) -> Self {
        let new_flags = match self.data.flags {
            Some(old_flags) => old_flags | flags,
            None => flags,
        };
        self.flags(new_flags)
    }

    /// Remove a specific combination of flags from the reply's flags.
    pub fn remove_flags(mut self, flags: MessageFlags) -> Self {
        self.data.flags = self.data.flags.map(|f| f.difference(flags));
        self
    }

    /// Set the reply's suppress embeds flag.
    pub fn suppress_embeds(self, suppress_embeds: bool) -> Self {
        if suppress_embeds {
            self.add_flags(MessageFlags::SUPPRESS_EMBEDS)
        } else {
            self.remove_flags(MessageFlags::SUPPRESS_EMBEDS)
        }
    }

    /// Set the reply's components v2 flag.
    pub fn components_v2(self, components_v2: bool) -> Self {
        if components_v2 {
            self.add_flags(MessageFlags::IS_COMPONENTS_V2)
        } else {
            self.remove_flags(MessageFlags::IS_COMPONENTS_V2)
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct ClearFlags: u8 {
        const MENTIONS   = 1 << 0;
        const COMPONENTS = 1 << 1;
        const CONTENT    = 1 << 2;
        const EMBEDS     = 1 << 3;
    }
}
