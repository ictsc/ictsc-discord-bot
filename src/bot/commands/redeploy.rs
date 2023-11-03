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

    // TODO: いろいろガバガバなので修正する
    pub async fn handle_redeploy_command(
        &self,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let problem_code = self.get_option_as_str(interaction, "problem_code").unwrap();

        let sender = &interaction.user;

        let problem = self
            .problems
            .iter()
            .find(|problem| problem.code == problem_code);

        let problem = match problem {
            Some(problem) => problem,
            None => {
                self.respond(interaction, |data| {
                    data.ephemeral(true).content(format!(
                        "問題コード `{}` に対応する問題はありません。",
                        problem_code
                    ))
                })
                .await?;
                return Ok(());
            }
        };

        self.respond(interaction, |data| {
            data.content(format!("問題 `{}` を再展開しますか？", problem.name))
        })
        .await?;

        let ok_reaction = ReactionType::Unicode(OK_REACTION.to_string());
        let ng_reaction = ReactionType::Unicode(NG_REACTION.to_string());

        let message = interaction
            .get_interaction_response(&self.discord_client)
            .await?;

        message.react(&self.discord_client, ok_reaction).await?;
        message.react(&self.discord_client, ng_reaction).await?;

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
                    .await?;
                return Ok(());
            }
        };

        let should_be_recreated = match &reaction.as_inner_ref().emoji {
            ReactionType::Unicode(emoji) => emoji == OK_REACTION,
            _ => {
                message
                    .reply(&self.discord_client, "予期しない状態です")
                    .await?;
                return Ok(());
            }
        };

        if should_be_recreated {
            let target = RedeployTarget {
                // Team IDを引っ張ってきて
                team_id: String::from("dummy"),
                problem_id: problem.code.clone(),
            };

            let result = self.redeploy_service.redeploy(&target).await;

            match result {
                Ok(_) => {
                    message
                        .reply(&self.discord_client, "再展開を開始しました。")
                        .await?;

                    for notifier in &self.redeploy_notifiers {
                        notifier.notify(&target).await;
                    }
                }
                Err(_) => {
                    message
                        .reply(&self.discord_client, "再展開に失敗しました。")
                        .await?;
                }
            }
        } else {
            message
                .reply(&self.discord_client, "再展開を中断します。")
                .await?;
        }

        Ok(())
    }
}
