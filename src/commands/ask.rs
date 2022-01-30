use crate::*;
use anyhow::Result;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::HashMap;

pub struct AskCommand<F, C>
where
    F: UserFinder+ Send,
    C: ThreadCreator + Send,
{
    finder: F,
    creator: C,
}

impl<F, C> AskCommand<F, C>
where
    F: UserFinder+ Send,
    C: ThreadCreator + Send + Sync,
{
    pub fn new(finder: F, creator: C) -> Self {
        Self { finder, creator }
    }

    pub async fn run<S: ToString + Send>(&self, ctx: &Context, channel_id: ChannelId, user_id: UserId, summary: S) -> Result<()> {
        let user = self.finder.find_by_id(ctx, user_id).await?;
        let content = format!("{} 質問内容を入力してください。", user.mention());
        let thread_id = self.creator.create(ctx, channel_id, summary, content).await?;
        Ok(())
    }
}
