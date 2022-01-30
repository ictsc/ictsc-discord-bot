use anyhow::Result;
use async_trait::async_trait;

use serenity::model::prelude::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::HashMap;

#[async_trait]
pub trait InteractionResponder {
    async fn send<D>(ctx: &Context, command: ApplicationCommandInteraction, msg: D) -> Result<()>
    where
        D: ToString + Send;
}

#[async_trait]
pub trait InteractionEphemeralResponder {
    async fn send_ephemeral<D>(
        ctx: &Context,
        command: ApplicationCommandInteraction,
        msg: D,
    ) -> Result<()>
    where
        D: ToString + Send;
}

#[async_trait]
pub trait InteractionTableResponder {
    async fn send_table(
        ctx: &Context,
        command: ApplicationCommandInteraction,
        table: HashMap<&str, String>,
    ) -> Result<()>;
}

pub struct InteractionHelper;

#[async_trait]
impl InteractionResponder for InteractionHelper {
    async fn send<D>(ctx: &Context, command: ApplicationCommandInteraction, msg: D) -> Result<()>
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
impl InteractionEphemeralResponder for InteractionHelper {
    async fn send_ephemeral<D>(
        ctx: &Context,
        command: ApplicationCommandInteraction,
        msg: D,
    ) -> Result<()>
    where
        D: ToString + Send,
    {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.content(msg)
                            .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
            })
            .await?;
        Ok(())
    }
}

#[async_trait]
impl InteractionTableResponder for InteractionHelper {
    async fn send_table(
        ctx: &Context,
        command: ApplicationCommandInteraction,
        table: HashMap<&str, String>,
    ) -> Result<()> {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.create_embed(|embed| {
                            for (key, value) in table {
                                embed.field(key, value, false);
                            }
                            embed
                        })
                    })
            })
            .await?;
        Ok(())
    }
}
