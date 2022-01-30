use anyhow::Result;
use async_trait::async_trait;
use serenity::model::prelude::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[async_trait]
pub trait InteractionResponder {
    async fn send<D>(ctx: Context, command: ApplicationCommandInteraction, msg: D) -> Result<()>
    where
        D: ToString + Send;
}

#[async_trait]
pub trait InteractionFollowUpper {
    async fn send_followup<D>(
        ctx: Context,
        command: ApplicationCommandInteraction,
        msg: D,
    ) -> Result<()>
    where
        D: ToString + Send;
}

pub struct InteractionHelper;

#[async_trait]
impl InteractionResponder for InteractionHelper {
    async fn send<D>(ctx: Context, command: ApplicationCommandInteraction, msg: D) -> Result<()>
    where
        D: ToString + Send,
    {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data.content(msg))
            })
            .await?;
        Ok(())
    }
}

#[async_trait]
impl InteractionFollowUpper for InteractionHelper {
    async fn send_followup<D>(
        ctx: Context,
        command: ApplicationCommandInteraction,
        msg: D,
    ) -> Result<()>
    where
        D: ToString + Send,
    {
        command
            .create_followup_message(&ctx.http, |response| response.content(msg))
            .await?;
        Ok(())
    }
}
