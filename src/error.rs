use thiserror::Error;

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