use crate::Result;
use async_trait::async_trait;
use serenity::http::Http;
use serenity::model::prelude::*;

#[derive(Default, Debug)]
pub struct CreateRoleInput {
    pub name: String,
    pub color: u64,
    pub mentionable: bool,
    pub hoist: bool,
    pub permissions: Permissions,
}

#[async_trait]
pub trait RoleCreator {
    async fn create(&self, http: &Http, guild_id: GuildId, input: CreateRoleInput) -> Result<Role>;
}

#[async_trait]
pub trait RoleFinder {
    async fn find_by_id(
        &self,
        http: &Http,
        guild_id: GuildId,
        role_id: RoleId,
    ) -> Result<Option<Role>>;
    async fn find_by_name<S: AsRef<str> + Send>(
        &self,
        http: &Http,
        guild_id: GuildId,
        name: S,
    ) -> Result<Vec<Role>>;
    async fn find_all(&self, http: &Http, guild_id: GuildId) -> Result<Vec<Role>>;
    async fn find_by_user(
        &self,
        http: &Http,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<Vec<Role>>;
}

#[async_trait]
pub trait RoleDeleter {
    async fn delete(&self, http: &Http, guild_id: GuildId, role_id: RoleId) -> Result<()>;
}

#[async_trait]
pub trait RoleBulkDeleter {
    async fn delete_all(&self, http: &Http, guild_id: GuildId) -> Result<()>;
}

#[async_trait]
pub trait RoleGranter {
    async fn grant(
        &self,
        http: &Http,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> Result<()>;
}

#[async_trait]
pub trait RoleRevoker {
    async fn revoke(
        &self,
        http: &Http,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> Result<()>;
}

#[async_trait]
pub trait RoleSyncer {
    async fn sync(
        &self,
        http: &Http,
        guild_id: GuildId,
        inputs: Vec<CreateRoleInput>,
    ) -> Result<()>;
}

pub struct RoleManager;

#[async_trait]
impl RoleCreator for RoleManager {
    async fn create(&self, http: &Http, guild_id: GuildId, input: CreateRoleInput) -> Result<Role> {
        Ok(guild_id
            .create_role(http, |role| {
                role.name(input.name)
                    .colour(input.color)
                    .mentionable(input.mentionable)
                    .hoist(input.hoist)
                    .permissions(input.permissions)
            })
            .await?)
    }
}

#[async_trait]
impl RoleFinder for RoleManager {
    async fn find_by_id(
        &self,
        http: &Http,
        guild_id: GuildId,
        role_id: RoleId,
    ) -> Result<Option<Role>> {
        for (id, role) in guild_id.roles(http).await? {
            if id == role_id {
                return Ok(Some(role));
            }
        }
        Ok(None)
    }

    async fn find_by_name<S: AsRef<str> + Send>(
        &self,
        http: &Http,
        guild_id: GuildId,
        name: S,
    ) -> Result<Vec<Role>> {
        let mut result: Vec<Role> = vec![];
        for (_, role) in guild_id.roles(http).await? {
            if name.as_ref() == role.name {
                result.push(role);
            }
        }
        Ok(result)
    }

    async fn find_all(&self, http: &Http, guild_id: GuildId) -> Result<Vec<Role>> {
        Ok(guild_id
            .roles(http)
            .await?
            .into_iter()
            .map(|(_k, v)| v)
            .collect())
    }

    async fn find_by_user(
        &self,
        http: &Http,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<Vec<Role>> {
        let member = guild_id.member(http, user_id).await?;

        let mut roles = Vec::new();
        for role_id in member.roles {
            roles.push(self.find_by_id(http, guild_id, role_id).await?.unwrap());
        }

        Ok(roles)
    }
}

#[async_trait]
impl RoleDeleter for RoleManager {
    async fn delete(&self, http: &Http, guild_id: GuildId, role_id: RoleId) -> Result<()> {
        Ok(guild_id.delete_role(http, role_id).await?)
    }
}

#[async_trait]
impl RoleBulkDeleter for RoleManager {
    #[tracing::instrument(skip(self, http))]
    async fn delete_all(&self, http: &Http, guild_id: GuildId) -> Result<()> {
        let roles = self.find_all(http, guild_id).await?;

        for role in roles {
            tracing::debug!(role_id = ?role.id, role_name = ?role.name, "deleting role");
            let result = self.delete(http, guild_id, role.id).await;
            if let Err(err) = result {
                tracing::warn!(?err, role_id = ?role.id, role_name = ?role.name, "failed to delete role");
            }
        }

        Ok(())
    }
}

#[async_trait]
impl RoleGranter for RoleManager {
    async fn grant(
        &self,
        http: &Http,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> Result<()> {
        let mut member = guild_id.member(http, user_id).await?;
        member.add_role(http, role_id).await;
        Ok(())
    }
}

#[async_trait]
impl RoleRevoker for RoleManager {
    async fn revoke(
        &self,
        http: &Http,
        guild_id: GuildId,
        user_id: UserId,
        role_id: RoleId,
    ) -> Result<()> {
        let mut member = guild_id.member(http, user_id).await?;
        member.remove_role(http, role_id).await;
        Ok(())
    }
}

#[async_trait]
impl<T> RoleSyncer for T
where
    T: RoleCreator + RoleDeleter + RoleFinder + Sync
{
    #[tracing::instrument(skip_all)]
    async fn sync(
        &self,
        http: &Http,
        guild_id: GuildId,
        inputs: Vec<CreateRoleInput>,
    ) -> Result<()> {
        let roles = self.find_all(http, guild_id).await?;

        let mut results = Vec::new();

        for input in inputs {
            tracing::debug!(?input, "syncing role");

            let filtered: Vec<_> = roles
                .iter()
                .filter(|role| role.name == input.name)
                .collect();

            match filtered.len() {
                1 => {
                    let role = filtered[0].clone();

                    tracing::debug!(?role, ?input, "role found, syncing");

                    let mut diff = role.colour.0 as u64 != input.color;
                    diff = diff || role.mentionable != input.mentionable;
                    diff = diff || role.hoist != input.hoist;
                    diff = diff || role.permissions != input.permissions;

                    if !diff {
                        results.push(role);
                        continue;
                    }

                    tracing::debug!("diff found, updating");

                    results.push(guild_id
                        .edit_role(http, role.id, |role| {
                            role.colour(input.color)
                                .mentionable(input.mentionable)
                                .hoist(input.hoist)
                                .permissions(input.permissions)
                        })
                        .await?);
                }
                _ => {
                    tracing::debug!(role_name = ?input.name, "role not found or several roles found, updating");
                    for role in filtered {
                        self.delete(http, guild_id, role.id).await?;
                    }
                    results.push(self.create(http, guild_id, input).await?);
                }
            }
        }

        Ok(())
    }
}
