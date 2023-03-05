use crate::commands::ApplicationCommandContext;
use crate::*;

use serenity::http::CacheHttp;
use serenity::model::prelude::*;

static STAFF_ROLE_NAME: &str = "ICTSC2022 Staff";

pub struct AskCommand<F, C>
where
    F: RoleFinder + Send,
    C: ThreadCreator + Send,
{
    guild_id: GuildId,
    finder: F,
    creator: C,
}

impl<F, C> AskCommand<F, C>
where
    F: RoleFinder + Send,
    C: ThreadCreator + Send + Sync,
{
    pub fn new(guild_id: GuildId, finder: F, creator: C) -> Self {
        Self { guild_id, finder, creator }
    }

    #[tracing::instrument(skip(self, ctx))]
    pub async fn run(&self, ctx: &ApplicationCommandContext, summary: String) -> Result<()> {
        let http = &ctx.context.http;

        if summary.chars().count() > 50 {
            return Err(UserError::SummaryTooLong.into());
        }

        let staff_roles = self.finder.find_by_name(http, self.guild_id, STAFF_ROLE_NAME).await?;
        let staff_role = staff_roles.get(0)
            .ok_or(SystemError::NoSuchRole(STAFF_ROLE_NAME.into()))?;

        let channel_id = ctx.command.channel_id;
        let user = &ctx.command.user;
        let content = format!("{} {} 質問内容を入力してください。", user.mention(), staff_role.mention());

        self.creator
            .create(http, channel_id, summary, content)
            .await?;

        InteractionHelper::defer_respond(
            &ctx.context.http,
            &ctx.command,
            "質問スレッドが開始されました",
        )
        .await;

        Ok(())
    }
}
