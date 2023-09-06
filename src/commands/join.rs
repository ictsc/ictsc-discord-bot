use crate::commands::ApplicationCommandContext;
use crate::*;

use serenity::futures::lock::Mutex;
use serenity::model::prelude::*;
use std::collections::HashMap;

pub struct JoinCommand<Repository>
where
    Repository: RoleFinder + RoleGranter + RoleRevoker + Send + Sync + 'static,
{
    repository: Repository,
    guild_id: GuildId,
    mapping: HashMap<String, String>,
    cache: Mutex<Option<HashMap<String, Role>>>,
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
            cache: Mutex::new(None),
        }
    }

    #[tracing::instrument(skip(self, http))]
    async fn find_role(&self, http: &serenity::http::Http, role_name: String) -> Result<Role> {
        let guild_id = self.guild_id;

        let mut guard = self.cache.lock().await;

        match *guard {
            Some(ref cache) => {
                tracing::debug!("cache found");
                Ok(cache
                    .get(&role_name)
                    .ok_or(SystemError::NoSuchRole(role_name))?
                    .clone())
            }
            None => {
                tracing::debug!("cache not found, fetching");
                let roles_mapping: HashMap<_, _> = self
                    .repository
                    .find_all(http, guild_id)
                    .await?
                    .into_iter()
                    .map(|r| (r.name.clone(), r))
                    .collect();

                let role = roles_mapping
                    .get(&role_name)
                    .ok_or(SystemError::NoSuchRole(role_name))?
                    .clone();

                *guard = Some(roles_mapping);

                Ok(role)
            }
        }
    }

    #[tracing::instrument(skip(self, ctx))]
    pub async fn run(
        &self,
        ctx: &ApplicationCommandContext,
        invitation_code: String,
    ) -> Result<()> {
        let http = &ctx.context.http;
        let command = &ctx.command;
        let guild_id = self.guild_id;

        // `invitation_code`の検証
        let role_name = self
            .mapping
            .get(&invitation_code)
            .ok_or(UserError::InvalidInvitationCode(invitation_code))?
            .clone();

        let user_id = ctx.command.user.id;

        let target_role = self.find_role(&ctx.context.http, role_name).await?;

        self.repository
            .grant(http, guild_id, user_id, target_role.id)
            .await?;

        InteractionHelper::defer_respond(http, command, "チームに参加しました。").await?;

        tracing::debug!("deleting old roles");
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
}
