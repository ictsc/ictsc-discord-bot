use crate::bot::*;
use crate::services::redeploy::RedeployTarget;

use std::time::Duration;

use anyhow::Result;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::{command::*, ReactionType};
use serenity::prelude::*;

const OK_REACTION: &str = "🙆\u{200d}♂\u{fe0f}";
const NG_REACTION: &str = "🙅\u{200d}♂\u{fe0f}";

#[derive(Debug, thiserror::Error)]
enum RedeployCommandError<'a> {
    #[error("問題コード `{0}` に対応する問題はありません。問題コードを再度お確かめください。")]
    InvalidProblemCodeError(&'a str),

    // /redeployコマンドの使用者のチームが解決できない時に発生するエラー
    #[error("予期しないエラーが発生しました。運営にお問い合わせください。")]
    UnexpectedSenderTeamsError,

    #[error("予期しないエラーが発生しました。")]
    Error(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

type RedeployCommandResult<'t, T> = std::result::Result<T, RedeployCommandError<'t>>;

impl Bot {
    pub fn create_redeploy_command(
        command: &mut CreateApplicationCommand,
    ) -> &mut CreateApplicationCommand {
        command
            .name("redeploy")
            .description("問題環境を再展開します。")
            .create_option(|option| {
                option
                    .name("problem_code")
                    .description("問題コード")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }

    pub async fn handle_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let problem= match self.validate_redeploy_command(interaction) {
            Ok(problem) => problem,
            Err(err) => {
                self.respond(interaction, |data| {
                    data.ephemeral(true).content(err.to_string())
                })
                .await?;
                return Ok(());
            },
        };

        self.defer_response(interaction).await?;

        if let Err(err) = self.do_redeploy_command(ctx, interaction, problem).await {
            tracing::error!(?err, "failed to do redeploy command");
            self.edit_response(interaction, |data| data.content(err.to_string()))
                .await?;
        }

        Ok(())
    }

    fn validate_redeploy_command<'t>(
        &self,
        interaction: &'t ApplicationCommandInteraction,
    ) -> RedeployCommandResult<'t, &Problem> {
        // TODO: unwrapを治す
        let problem_code = self
            .get_option_as_str(interaction, "problem_code")
            .ok_or(anyhow::anyhow!("problem_code is not found"))
            .unwrap();

        let problem = self
            .problems
            .iter()
            .find(|problem| problem.code == problem_code);

        problem.ok_or(RedeployCommandError::InvalidProblemCodeError(problem_code))
    }

    // TODO: いろいろガバガバなので修正する
    async fn do_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
        problem: &Problem,
    ) -> RedeployCommandResult<()> {
        let sender = &interaction.user;
        let sender_member = self.guild_id.member(&self.discord_client, sender.id).await.unwrap();

        let mut sender_teams = Vec::new();
        for role_id in sender_member.roles {
            let role = self.find_roles_by_id_cached(role_id).await.unwrap();
            match role {
                Some(role) => {
                    for team in &self.teams {
                        if role.name == team.role_name {
                            sender_teams.push(team);
                        }
                    }
                },
                None => (),
            }
        }

        // /joinコマンドの制約上、ユーザは高々1つのチームにしか所属しないはずである。
        // また、/redeployはGuildでのみ使用可能なため、チームに所属していないユーザは使用できない。
        if sender_teams.len() != 1 {
            return Err(RedeployCommandError::UnexpectedSenderTeamsError);
        }

        let sender_team = sender_teams.first().unwrap();

        self.edit_response(interaction, |data| {
            // TODO: チーム名にする
            data.content(format!("チーム `{}` の問題 `{}` を再展開しますか？", sender_team.role_name, problem.name))
        })
        .await.unwrap();

        let ok_reaction = ReactionType::Unicode(OK_REACTION.to_string());
        let ng_reaction = ReactionType::Unicode(NG_REACTION.to_string());

        let message = interaction
            .get_interaction_response(&self.discord_client)
            .await.unwrap();

        message.react(&self.discord_client, ok_reaction).await.unwrap();
        message.react(&self.discord_client, ng_reaction).await.unwrap();

        let reaction = message
            .await_reaction(ctx)
            .author_id(sender.id)
            .added(true)
            .removed(false)
            .filter(|reaction| {
                if let ReactionType::Unicode(emoji) = &reaction.emoji {
                    emoji == OK_REACTION || emoji == NG_REACTION
                } else {
                    false
                }
            })
            .timeout(Duration::from_secs(30))
            .await;

        let reaction = match reaction {
            Some(reaction) => reaction,
            None => {
                message
                    .reply(&self.discord_client, "タイムアウトしました。")
                    .await.unwrap();
                return Ok(());
            }
        };

        let should_be_recreated = match &reaction.as_inner_ref().emoji {
            ReactionType::Unicode(emoji) => emoji == OK_REACTION,
            _ => {
                message
                    .reply(&self.discord_client, "予期しない状態です")
                    .await.unwrap();
                return Ok(());
            }
        };

        if !should_be_recreated {
            message
                .reply(&self.discord_client, "再展開を中断します。")
                .await.unwrap();
            return Ok(());
        }

        let target = RedeployTarget {
            team_id: sender_team.id.clone(),
            problem_id: problem.code.clone(),
        };
        let result = self.redeploy_service.redeploy(&target).await;
        for notifier in &self.redeploy_notifiers {
            notifier.notify(&target, &result).await;
        }

        match result {
            Ok(_) => {
                message
                    .reply(&self.discord_client, "再展開を開始しました。")
                    .await.unwrap();
            }
            Err(_) => {
                message
                    .reply(&self.discord_client, "再展開に失敗しました。")
                    .await.unwrap();
            }
        };

        Ok(())
    }
}
