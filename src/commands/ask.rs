use crate::commands::ApplicationCommandContext;
use crate::*;
use anyhow::Result;
use serenity::http::{CacheHttp, Http};
use serenity::model::prelude::*;

pub struct AskCommand<F, C>
where
    F: UserFinder + Send,
    C: ThreadCreator + Send,
{
    finder: F,
    creator: C,
}

impl<F, C> AskCommand<F, C>
where
    F: UserFinder + Send,
    C: ThreadCreator + Send + Sync,
{
    pub fn new(finder: F, creator: C) -> Self {
        Self { finder, creator }
    }

    pub async fn run(&self, ctx: &ApplicationCommandContext, summary: String) -> Result<()> {
        let channel_id = ctx.command.channel_id;
        let user = &ctx.command.user;
        let content = format!("{} 質問内容を入力してください。", user.mention());

        self.creator
            .create(&ctx.context.http, channel_id, summary, content)
            .await?;

        InteractionHelper::send_ephemeral(
            &ctx.context.http,
            &ctx.command,
            "質問スレッドが開始されました",
        )
        .await;

        Ok(())
    }
}
