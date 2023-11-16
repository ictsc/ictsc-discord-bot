use serenity::builder::CreateInteractionResponseData;
use serenity::builder::EditInteractionResponse;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::message_component::MessageComponentInteraction;
use serenity::model::prelude::*;

use super::HelperResult;
use crate::bot::Bot;

pub enum Interaction<'a> {
    ApplicationCommandInteraction(&'a ApplicationCommandInteraction),
    MessageComponentInteraction(&'a MessageComponentInteraction),
}

impl<'a> From<&'a ApplicationCommandInteraction> for Interaction<'a> {
    fn from(interaction: &'a ApplicationCommandInteraction) -> Self {
        Interaction::ApplicationCommandInteraction(interaction)
    }
}

impl<'a> From<&'a MessageComponentInteraction> for Interaction<'a> {
    fn from(interaction: &'a MessageComponentInteraction) -> Self {
        Interaction::MessageComponentInteraction(interaction)
    }
}

// Interactionに対する操作するためのヘルパー関数
impl Bot {
    // ユーザからのinteractionに即時応答するメソッド
    #[tracing::instrument(skip_all)]
    pub async fn respond<'a, I, F>(&self, interaction: I, f: F) -> HelperResult<()>
    where
        I: Into<Interaction<'a>>,
        for<'b, 'c> F: FnOnce(
            &'b mut CreateInteractionResponseData<'c>,
        ) -> &'b mut CreateInteractionResponseData<'c>,
    {
        tracing::trace!("Respond");
        Ok(match interaction.into() {
            Interaction::ApplicationCommandInteraction(interaction) => {
                interaction
                    .create_interaction_response(&self.discord_client, |response| {
                        response.kind(InteractionResponseType::ChannelMessageWithSource);
                        response.interaction_response_data(f)
                    })
                    .await?
            },
            Interaction::MessageComponentInteraction(interaction) => {
                interaction
                    .create_interaction_response(&self.discord_client, |response| {
                        response.kind(InteractionResponseType::UpdateMessage);
                        response.interaction_response_data(f)
                    })
                    .await?
            },
        })
    }

    // ユーザからのinteractionの応答を保留するメソッド
    #[tracing::instrument(skip_all)]
    pub async fn defer_response<'a, I>(&self, interaction: I) -> HelperResult<()>
    where
        I: Into<Interaction<'a>>,
    {
        tracing::trace!("Defer response");
        Ok(match interaction.into() {
            Interaction::ApplicationCommandInteraction(interaction) => {
                interaction
                    .create_interaction_response(&self.discord_client, |f| {
                        f.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    })
                    .await?
            },
            Interaction::MessageComponentInteraction(interaction) => {
                interaction
                    .create_interaction_response(&self.discord_client, |f| {
                        f.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    })
                    .await?
            },
        })
    }

    // ユーザからのinteractionの応答を編集するメソッド
    #[tracing::instrument(skip_all)]
    pub async fn edit_response<'a, I, F>(&self, interaction: I, f: F) -> HelperResult<Message>
    where
        I: Into<Interaction<'a>>,
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse,
    {
        tracing::trace!("Edit response");
        Ok(match interaction.into() {
            Interaction::ApplicationCommandInteraction(interaction) => {
                interaction
                    .edit_original_interaction_response(&self.discord_client, f)
                    .await?
            },
            Interaction::MessageComponentInteraction(interaction) => {
                interaction
                    .edit_original_interaction_response(&self.discord_client, f)
                    .await?
            },
        })
    }

    // ユーザからのinteractionの応答をMessageとして取得するメソッド
    #[tracing::instrument(skip_all)]
    pub async fn get_response<'a, I>(&self, interaction: I) -> HelperResult<Message>
    where
        I: Into<Interaction<'a>>,
    {
        tracing::trace!("Get response");
        Ok(match interaction.into() {
            Interaction::ApplicationCommandInteraction(interaction) => {
                interaction
                    .get_interaction_response(&self.discord_client)
                    .await?
            },
            Interaction::MessageComponentInteraction(interaction) => {
                interaction
                    .get_interaction_response(&self.discord_client)
                    .await?
            },
        })
    }

    pub fn get_option_as_str<'t>(
        &self,
        options: &'t [CommandDataOption],
        name: &str,
    ) -> Option<&'t str> {
        for option in options {
            if option.name == name {
                return option.value.as_ref().and_then(|v| v.as_str());
            }
        }
        None
    }
}
