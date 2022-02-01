use anyhow::Result;
use async_trait::async_trait;
use serenity::http::Http;
use serenity::model::prelude::*;


#[derive(Default)]
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
}

#[async_trait]
pub trait RoleDeleter {
    async fn delete(&self, http: &Http, guild_id: GuildId, role_id: RoleId) -> Result<()>;
}

#[async_trait]
pub trait RoleSyncer: RoleCreator + RoleFinder + RoleDeleter {
    async fn sync(&self, http: &Http, guild_id: GuildId, input: CreateRoleInput) -> Result<()> {
        let roles = self.find_by_name(http, guild_id, &input.name).await?;

        match roles.len() {
            1 => {
                guild_id
                    .edit_role(http, roles[0].id, |role| {
                        role.colour(input.color)
                            .mentionable(input.mentionable)
                            .hoist(input.hoist)
                            .permissions(input.permissions)
                    })
                    .await?;
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
}

#[async_trait]
impl RoleDeleter for RoleManager {
    async fn delete(&self, http: &Http, guild_id: GuildId, role_id: RoleId) -> Result<()> {
        Ok(guild_id.delete_role(http, role_id).await?)
    }
}

impl RoleSyncer for RoleManager {}
