use serenity::model::prelude::*;

use super::HelperResult;
use crate::bot::Bot;

// Guildのメンバーを操作するためのヘルパー関数
impl Bot {
    #[tracing::instrument(skip_all)]
    pub async fn get_member(&self, user: &User) -> HelperResult<Member> {
        tracing::trace!("Get member");
        Ok(self.guild_id.member(&self.discord_client, user).await?)
    }

    #[tracing::instrument(skip_all)]
    pub async fn grant_roles<T>(
        &self,
        member: &mut Member,
        role_ids: T,
    ) -> HelperResult<Vec<RoleId>>
    where
        T: AsRef<[RoleId]>,
    {
        tracing::trace!("Grant roles");

        Ok(member
            .add_roles(&self.discord_client, role_ids.as_ref())
            .await?)
    }

    #[tracing::instrument(skip_all)]
    pub async fn revoke_roles<T>(
        &self,
        member: &mut Member,
        role_ids: T,
    ) -> HelperResult<Vec<RoleId>>
    where
        T: AsRef<[RoleId]>,
    {
        tracing::trace!("Revoke roles");

        Ok(member
            .remove_roles(&self.discord_client, role_ids.as_ref())
            .await?)
    }
}
