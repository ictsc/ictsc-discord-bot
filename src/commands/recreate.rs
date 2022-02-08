use crate::commands::{ApplicationCommandContext, ReactionContext};
use crate::*;
use anyhow::Result;

use std::collections::HashMap;
use std::sync::Arc;
use serenity::model::channel::ReactionType;
use serenity::model::id::ChannelId;
use tokio::sync::Mutex;

const OK_REACTION: &str = "🙆\u{200d}♂\u{fe0f}";
const NG_REACTION: &str = "🙅\u{200d}♂\u{fe0f}";

pub struct RecreateCommand<Repository>
where
    Repository: RoleFinder + Send,
{
    repository: Repository,
    teams: HashMap<String, TeamConfiguration>,
    problems: HashMap<String, ProblemConfiguration>,
    ok_reaction: ReactionType,
    ng_reaction: ReactionType,
    pending_requests: Arc<Mutex<HashMap<u64, RecreateRequest>>>,
}

struct RecreateRequest {
    pub channel_id: ChannelId,
    pub team_id: String,
    pub problem_id: String,
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
            ok_reaction: ReactionType::Unicode(OK_REACTION.into()),
            ng_reaction: ReactionType::Unicode(NG_REACTION.into()),
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
        let channel_id = message.channel_id;

        {
            let mut table = self.pending_requests.lock().await;
            table.insert(message_id, RecreateRequest {
                channel_id, team_id, problem_id,
            });
        }

        Ok(())
    }

    pub async fn add_reaction(&self, ctx: &ReactionContext) -> Result<()> {
        let reaction = ctx.reaction.emoji.to_string();

        let message_id = ctx.reaction.message_id;

        if reaction != OK_REACTION && reaction != NG_REACTION {
            return Ok(());
        }

        let request = {
            let mut table = self.pending_requests.lock().await;
            match table.remove(message_id.as_u64()) {
                Some(v) => v,
                None => return Ok(()),
            }
        };

        match reaction.as_str() {
            OK_REACTION => {
                request.channel_id.send_message(&ctx.context.http, |message| {
                    message.content("初期化を開始します。")
                }).await?;
            },
            NG_REACTION => {
                request.channel_id.send_message(&ctx.context.http, |message| {
                    message.content("初期化を中断します。")
                }).await?;
            },
            _ => {},
        };

        Ok(())
    }
}
