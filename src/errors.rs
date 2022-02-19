use std::fmt::Display;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    UserError(#[from] UserError),
    #[error("不明なエラーが発生しました。")]
    SystemError(#[from] SystemError),
    #[error("不明なエラーが発生しました。")]
    SerenityError(#[from] serenity::Error),
    #[error("不明なエラーが発生しました。")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("招待コードが不正です。")]
    InvalidInvitationCode(String),
    #[error("そのような問題はありません。")]
    NoSuchProblem(String),
    #[error("チームに所属していないため、処理を実行することができません。")]
    UserNotInTeam,
    #[error("再作成リクエストが実行中です。")]
    RequestInQueue,
    #[error("競技時間外です。")]
    OutOfCompetitionTime,
    #[error("許可されていない処理です。")]
    Forbidden,
}

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("no such role: {0}")]
    NoSuchRole(String),
    #[error("unexpected command: {0}")]
    UnhandledCommand(String),
    #[error("unexpected error: {0}")]
    UnexpectedError(String),
}
