use super::HelperResult;

use crate::bot::Bot;

use serenity::builder::{CreateInteractionResponseData, EditInteractionResponse};
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::*;

// Interactionに対する操作するためのヘルパー関数
impl Bot {
    // ユーザからのinteractionに即時応答するメソッド
    pub async fn respond<F>(
        &self,
        interaction: &ApplicationCommandInteraction,
        f: F,
    ) -> HelperResult<()>
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

    // ユーザからのinteractionの応答を保留するメソッド
    pub async fn defer_response(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> HelperResult<()> {
        interaction.defer(&self.discord_client).await?;
        Ok(())
    }

    // ユーザからのinteractionの応答を編集するメソッド
    pub async fn edit_response<F>(
        &self,
        interaction: &ApplicationCommandInteraction,
        f: F,
    ) -> HelperResult<()>
    where
        F: FnOnce(&mut EditInteractionResponse) -> &mut EditInteractionResponse,
    {
        interaction
            .edit_original_interaction_response(&self.discord_client, f)
            .await?;
        Ok(())
    }

    // ユーザからのinteractionの応答をMessageとして取得するメソッド
    pub async fn get_response(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> HelperResult<Message> {
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
