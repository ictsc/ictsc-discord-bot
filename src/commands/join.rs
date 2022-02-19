use crate::commands::ApplicationCommandContext;
use crate::*;

use serenity::model::prelude::*;
use std::collections::HashMap;

pub struct JoinCommand<Repository>
where
    Repository: RoleFinder + RoleGranter + RoleRevoker + Send + Sync + 'static,
{
    repository: Repository,
    guild_id: GuildId,
    mapping: HashMap<String, String>,
}

impl<Repository> JoinCommand<Repository>
where
    Repository: RoleFinder + RoleGranter + RoleRevoker + Send + Sync,
{
    pub fn new(
        repository: Repository,
        guild_id: GuildId,
        mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            repository,
            guild_id,
            mapping,
        }
    }

    #[tracing::instrument(skip_all, fields(
        role_name = role_name
    ))]
    pub async fn run_defer(
        &self,
        ctx: &ApplicationCommandContext,
        user_id: UserId,
        role_name: &str,
    ) -> Result<()> {
        let http = &ctx.context.http;
        let guild_id = self.guild_id;

        // 入るべきRoleの取得
        // TODO: ロール名から毎回検索をかけずに、初回にRoleIdを解決する
        let target_roles = self
            .repository
            .find_by_name(http, guild_id, role_name.clone())
            .await?;
        let target_role = target_roles
            .first()
            .ok_or(SystemError::NoSuchRole(role_name.into()))?;

        self.repository
            .grant(http, guild_id, user_id, target_role.id)
            .await?;
        let granted_roles = self
            .repository
            .find_by_user(http, guild_id, user_id)
            .await?;
        for role in &granted_roles {
            if role.id != target_role.id {
                self.repository
                    .revoke(http, guild_id, user_id, role.id)
                    .await?;
            }
        }

        tracing::info!("granted");

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx))]
    pub async fn run(
        &self,
        ctx: &ApplicationCommandContext,
        invitation_code: String,
    ) -> Result<()> {
        let http = &ctx.context.http;
        let command = &ctx.command;

        // `invitation_code`の検証
        let role_name = self
            .mapping
            .get(&invitation_code)
            .ok_or(UserError::InvalidInvitationCode(invitation_code))?;

        let _guild_id = self.guild_id;
        let user_id = ctx.command.user.id;

        InteractionHelper::defer(http, command).await;

        match self.run_defer(ctx, user_id, role_name).await {
            Ok(_) => {
                InteractionHelper::defer_respond(http, command, "チームに参加しました。").await
            }
            Err(err) => {
                tracing::warn!("failed to run join: {:?}", err);
                InteractionHelper::defer_respond(http, command, format!("{} (id: {})", err, ctx.command.id)).await
            }
        };

        Ok(())
    }
}
