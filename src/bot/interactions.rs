use crate::bot::Bot;
use crate::*;

use serenity::builder::{CreateInteractionResponseData, EditInteractionResponse};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{InteractionResponseType, Message};

impl Bot {
    pub async fn reply<F>(&self, interaction: &ApplicationCommandInteraction, f: F) -> Result<()>
    where
        for<'a, 'b> F: FnOnce(
            &'a mut CreateInteractionResponseData<'b>,
        ) -> &'a mut CreateInteractionResponseData<'b>,
    {
        interaction
            .create_interaction_response(&self.discord_client, |response| {
                response.kind(InteractionResponseType::ChannelMessageWithSource);
                response.interaction_response_data(f)
            })
            .await?;
        Ok(())
    }

    pub async fn defer_reply(&self, interaction: &ApplicationCommandInteraction) -> Result<()> {
        interaction.defer(&self.discord_client).await?;
        Ok(())
    }

    pub async fn edit_reply<F>(
        &self,
        interaction: &ApplicationCommandInteraction,
        f: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse,
    {
        interaction
            .edit_original_interaction_response(&self.discord_client, f)
            .await?;
        Ok(())
    }

    pub async fn get_response_message(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Message> {
        Ok(interaction
            .get_interaction_response(&self.discord_client)
            .await?)
    }

    pub fn get_option_as_str<'t>(
        &self,
        interaction: &'t ApplicationCommandInteraction,
        name: &str,
    ) -> Option<&'t str> {
        for option in &interaction.data.options {
            if option.name == name {
                return option.value.as_ref().and_then(|v| v.as_str());
            }
        }
        None
    }
}
