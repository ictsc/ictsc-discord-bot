use crate::*;
use serenity::http::Http;
use serenity::model::prelude::*;
use std::collections::HashMap;

pub struct JoinCommand<Repository>
where
    Repository: RoleFinder + RoleGranter + RoleRevoker + Send + Sync + 'static,
{
    repository: Repository,
    definitions: HashMap<String, String>,
}

impl<Repository> JoinCommand<Repository>
where
    Repository: RoleFinder + RoleGranter + RoleRevoker + Send + Sync,
{
    pub fn new(repository: Repository, definitions: HashMap<String, String>) -> Self {
        Self {
            repository, definitions,
        }
    }

    pub async fn run(&self, http: &Http, guild_id: GuildId, user_id: UserId, invitation_code: String) -> Result<Role> {
        // `invitation_code`の検証
        let target_role_name = self.definitions.get(&invitation_code)
            .ok_or(UserError::InvalidInvitationCode)?;

        // 入るべきRoleの取得
        // TODO: ロール名から毎回検索をかけずに、初回にRoleIdを解決する
        let target_roles = self.repository.find_by_name(http, guild_id, target_role_name).await?;
        let target_role = target_roles.first()
            .ok_or(SystemError::NoSuchRole(target_role_name.clone()))?;

        self.repository.grant(http, guild_id, user_id, target_role.id).await?;
        let granted_roles = self.repository.find_by_user(http, guild_id, user_id).await?;
        for role in &granted_roles {
            if role.id != target_role.id {
                self.repository.revoke(http, guild_id, user_id, role.id).await?;
            }
        }

        Ok(target_role.clone())
    }
}
