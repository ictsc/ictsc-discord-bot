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
    InvalidInvitationCode,
    #[error("そのような問題はありません。")]
    NoSuchProblem,
    #[error("許可されていない処理です。")]
    Forbidden,
}

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("no such role: {0}")]
    NoSuchRole(String),
}
