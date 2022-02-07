use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    UserError(#[from] UserError),
    #[error("{0}")]
    SystemError(#[from] SystemError),
    #[error("{0}")]
    SerenityError(#[from] serenity::Error),
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("招待コードが不正です。")]
    InvalidInvitationCode,
}

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("no such role: {0}")]
    NoSuchRole(String),
}
