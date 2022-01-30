use crate::*;
use anyhow::Result;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::HashMap;
use serenity::http::Http;

pub struct WhoAmICommand<F>
where
    F: UserFinder + Send + Sync,
{
    finder: F,
}

impl<F> WhoAmICommand<F>
where
    F: UserFinder + Send + Sync,
{
    pub fn new(finder: F) -> Self {
        Self { finder }
    }

    pub async fn run(&self, http: &Http, id: UserId) -> Result<HashMap<&'static str, String>> {
        let info = self.finder.find_by_id(http, id).await.map(|user| {
            let mut table: HashMap<&str, String> = HashMap::new();
            table.insert("ID", user.id.to_string());
            table.insert("名前", user.name);
            table
        })?;
        Ok(info)
    }
}
