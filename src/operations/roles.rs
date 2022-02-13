use crate::Result;
use async_trait::async_trait;
use serenity::http::Http;
use serenity::model::prelude::*;
use serenity::utils::Color;

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
pub trait RoleSyncer: RoleCreator + RoleFinder + RoleDeleter {
    async fn sync_cached(&self, http: &Http, guild_id: GuildId, roles: &[Role], input: CreateRoleInput) -> Result<()> {
        let roles: Vec<_> = roles.iter()
            .filter(|role| role.name == input.name)
            .collect();

        match roles.len() {
            1 => {
                let role = roles[0];

                tracing::debug!("{:?} vs {:?}", role, input);

                let mut diff = role.colour.0 as u64 != input.color;
                diff = diff || role.mentionable != input.mentionable;
                diff = diff || role.hoist != input.hoist;
                diff = diff || role.permissions != input.permissions;

                if diff {
                    tracing::debug!("diff is found, updating");
                    guild_id
                        .edit_role(http, role.id, |role| {
                            role.colour(input.color)
                                .mentionable(input.mentionable)
                                .hoist(input.hoist)
                                .permissions(input.permissions)
                        })
                        .await?;
                }
            }
            _ => {
                for role in roles {
                    self.delete(http, guild_id, role.id).await?;
                }
                self.create(http, guild_id, input).await?;
            }
        };

        Ok(())
    }

    async fn sync(&self, http: &Http, guild_id: GuildId, input: CreateRoleInput) -> Result<()> {
        let roles = self.find_all(http, guild_id).await?;

        self.sync_cached(http, guild_id, &roles, input).await?;

        Ok(())
    }

    async fn sync_bulk(&self, http: &Http, guild_id: GuildId, inputs: Vec<CreateRoleInput>) -> Result<()> {
        let roles = self.find_all(http, guild_id).await?;

        for input in inputs {
            self.sync_cached(http, guild_id, &roles, input).await?;
        }

        Ok(())
    }
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
        let roles = self.find_all(http, guild_id).await?;

        let user = user_id.to_user(http).await?;

        let mut filtered = Vec::new();
        for role in roles.iter() {
            if user.has_role(http, guild_id, role).await? {
                filtered.push(role.clone());
            }
        }

        return Ok(filtered);
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
    async fn delete_all(&self, http: &Http, guild_id: GuildId) -> Result<()> {
        tracing::debug!("RoleManager#delete_all");

        let roles = self.find_all(http, guild_id).await?;

        for role in roles {
            match self.delete(http, guild_id, role.id).await {
                Ok(_) =>
                    tracing::debug!("deleted role (id: {}, name: {})", role.id, role.name),
                Err(err) =>
                    tracing::warn!("couldn't delete role (id: {}, name: {})", role.id, role.name),
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

impl RoleSyncer for RoleManager {}
