use anyhow::Result;
use async_trait::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[async_trait]
pub trait UserFinder {
    async fn find_by_id(&self, ctx: &Context, id: UserId) -> Result<User>;
}

pub struct UserManager;

#[async_trait]
impl UserFinder for UserManager {
    async fn find_by_id(&self, ctx: &Context, id: UserId) -> Result<User> {
        Ok(id.to_user(&ctx.http).await?)
    }
}
