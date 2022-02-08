use crate::Result;
use async_trait::async_trait;
use serenity::http::Http;
use serenity::model::prelude::*;

#[async_trait]
pub trait UserFinder {
    async fn find_by_id(&self, http: &Http, id: UserId) -> Result<User>;
}

pub struct UserManager;

#[async_trait]
impl UserFinder for UserManager {
    async fn find_by_id(&self, http: &Http, id: UserId) -> Result<User> {
        Ok(id.to_user(http).await?)
    }
}
