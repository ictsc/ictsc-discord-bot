use crate::commands::ApplicationCommandContext;
use crate::*;

use std::collections::HashMap;

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

    #[tracing::instrument(skip(self, ctx))]
    pub async fn run(&self, ctx: &ApplicationCommandContext) -> Result<()> {
        let user = &ctx.command.user;

        let mut table: HashMap<&str, String> = HashMap::new();

        table.insert("ID", user.id.to_string());
        table.insert("名前", user.name.clone());

        InteractionHelper::send_table(&ctx.context.http, &ctx.command, table).await;

        Ok(())
    }
}
