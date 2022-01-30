use anyhow::Result;
use async_trait::async_trait;

use serenity::model::prelude::application_command::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::HashMap;

#[async_trait]
pub trait ThreadCreator {
    async fn create<St, Sc>(&self, ctx: &Context, channel_id: ChannelId, title: St, content: Sc) -> Result<ChannelId>
        where St: ToString + Send, Sc: ToString + Send;
}

pub struct ThreadManager;

#[async_trait]
impl ThreadCreator for ThreadManager{
    async fn create<St, Sc>(&self, ctx: &Context, channel_id: ChannelId, title: St, content: Sc) -> Result<ChannelId>
        where St: ToString + Send, Sc: ToString + Send {
        let message = channel_id.send_message(&ctx.http, |message| {
            message.content(content)
        }).await?;

        let thread = channel_id.create_public_thread(&ctx.http, message.id, |thread| {
            thread.name(title)
        }).await?;

        Ok(thread.id)
    }
}

