use thiserror::Error;

pub type CommandResult<T> = anyhow::Result<T>;

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
    #[error("不明なエラーが発生しました。")]
    Error(#[from] Box<dyn std::error::Error>),
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("招待コード `{0}` に対応するチームはありません。招待コードを再度お確かめください。")]
    InvalidInvitationCode(String),
    #[error("問題コード `{0}` に対応する問題がありません。問題コードを再度お確かめください。")]
    NoSuchProblem(String),
    #[error("チームに所属していないため、処理を実行することができません。/joinを用いてチームに参加してください。")]
    UserNotInTeam,
    #[error("質問の概要が長すぎます。")]
    SummaryTooLong,
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
    #[error("no such category: {0}")]
    NoSuchCategory(String),
    #[error("unexpected command: {0}")]
    UnhandledCommand(String),
    #[error("unexpected error: {0}")]
    UnexpectedError(String),
}
