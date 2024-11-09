use serenity::all::CommandDataOption;
use serenity::all::CommandInteraction;
use serenity::all::ComponentInteraction;
use serenity::all::CreateInteractionResponse;
use serenity::all::CreateInteractionResponseMessage;
use serenity::all::Message;
use serenity::builder::EditInteractionResponse;

use super::HelperResult;
use crate::bot::Bot;

pub enum Interaction<'a> {
    CommandInteraction(&'a CommandInteraction),
    ComponentInteraction(&'a ComponentInteraction),
}

impl<'a> From<&'a CommandInteraction> for Interaction<'a> {
    fn from(interaction: &'a CommandInteraction) -> Self {
        Interaction::CommandInteraction(interaction)
    }
}

impl<'a> From<&'a ComponentInteraction> for Interaction<'a> {
    fn from(interaction: &'a ComponentInteraction) -> Self {
        Interaction::ComponentInteraction(interaction)
    }
}

// Interactionに対する操作するためのヘルパー関数
impl Bot {
    // ユーザからのinteractionに即時応答するメソッド
    #[tracing::instrument(skip_all)]
    pub async fn respond<'a, I>(
        &self,
        interaction: I,
        message: CreateInteractionResponseMessage,
    ) -> HelperResult<()>
    where
        I: Into<Interaction<'a>>,
    {
        tracing::trace!("Respond");
        Ok(match interaction.into() {
            Interaction::CommandInteraction(interaction) => {
                interaction
                    .create_response(
                        &self.discord_client,
                        CreateInteractionResponse::Message(message),
                    )
                    .await?
            },
            Interaction::ComponentInteraction(interaction) => {
                interaction
                    .create_response(
                        &self.discord_client,
                        CreateInteractionResponse::Message(message),
                    )
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
            Interaction::CommandInteraction(interaction) => {
                interaction
                    .create_response(
                        &self.discord_client,
                        CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
                    )
                    .await?
            },
            Interaction::ComponentInteraction(interaction) => {
                interaction
                    .create_response(
                        &self.discord_client,
                        CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new()),
                    )
                    .await?
            },
        })
    }

    // ユーザからのinteractionの応答を編集するメソッド
    #[tracing::instrument(skip_all)]
    pub async fn edit_response<'a, I>(
        &self,
        interaction: I,
        message: EditInteractionResponse,
    ) -> HelperResult<Message>
    where
        I: Into<Interaction<'a>>,
    {
        tracing::trace!("Edit response");
        Ok(match interaction.into() {
            Interaction::CommandInteraction(interaction) => {
                interaction
                    .edit_response(&self.discord_client, message)
                    .await?
            },
            Interaction::ComponentInteraction(interaction) => {
                interaction
                    .edit_response(&self.discord_client, message)
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
            Interaction::CommandInteraction(interaction) => {
                interaction.get_response(&self.discord_client).await?
            },
            Interaction::ComponentInteraction(interaction) => {
                interaction.get_response(&self.discord_client).await?
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
                return option.value.as_str();
            }
        }
        None
    }
}
