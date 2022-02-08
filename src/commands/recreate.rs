use crate::commands::ApplicationCommandContext;
use crate::*;
use anyhow::Result;

use std::collections::HashMap;
use std::sync::Arc;
use serenity::model::channel::ReactionType;
use tokio::sync::Mutex;

pub struct RecreateCommand<Repository>
where
    Repository: RoleFinder + Send,
{
    repository: Repository,
    teams: HashMap<String, TeamConfiguration>,
    problems: HashMap<String, ProblemConfiguration>,
    ok_reaction: ReactionType,
    ng_reaction: ReactionType,
    pending_requests: Arc<Mutex<HashMap<u64, (String, String)>>>,
}

impl<Repository> RecreateCommand<Repository>
where
    Repository: RoleFinder + Send,
{
    pub fn new(repository: Repository, teams: &[TeamConfiguration], problems: &[ProblemConfiguration]) -> Self {
        let mut ts = HashMap::new();
        teams.iter().for_each(|team| {
            ts.insert(team.role_name.clone(), team.clone());
        });

        let mut ps = HashMap::new();
        problems.iter().for_each(|problem| {
            ps.insert(problem.code.clone(), problem.clone());
        });

        Self {
            repository,
            teams: ts,
            problems: ps,
            ok_reaction: ReactionType::Unicode("🙆\u{200d}♂\u{fe0f}".into()),
            ng_reaction: ReactionType::Unicode("🙅\u{200d}♂\u{fe0f}".into()),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&self, ctx: &ApplicationCommandContext, code: String) -> Result<()> {
        let guild_id = ctx.command.guild_id.unwrap();
        let user = &ctx.command.user;

        let roles = self.repository.find_by_user(&ctx.context.http, guild_id, user.id).await?;

        let mut team = None;
        for role in &roles {
            match self.teams.get(&role.name) {
                Some(t) => team = Some(t.clone()),
                _ => (),
            };
        }

        let team = team.ok_or(errors::UserError::Forbidden)?;

        let problem = self.problems.get(&code)
            .map(|problem| problem.clone())
            .ok_or(errors::UserError::NoSuchProblem)?;

        let content = format!("問題「{}」を初期化します。よろしいですか？", problem.name);

        let message = InteractionHelper::send(&ctx.context.http, &ctx.command, content).await?;
        InteractionHelper::react(&ctx.context.http, &ctx.command, self.ok_reaction.clone()).await;
        InteractionHelper::react(&ctx.context.http, &ctx.command, self.ng_reaction.clone()).await;

        let message_id = *message.id.as_u64();
        let team_id = team.id;
        let problem_id = problem.id;

        {
            let mut table = self.pending_requests.lock().await;
            table.insert(message_id, (team_id, problem_id));
        }

        Ok(())
    }
}
