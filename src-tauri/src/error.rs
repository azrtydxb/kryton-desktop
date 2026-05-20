use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url: {0}")]
    Url(#[from] url::ParseError),
    #[error("auth failed (status {0})")]
    AuthFailed(u16),
    #[error("account not found: {0}")]
    AccountNotFound(String),
    #[error("keychain: {0}")]
    Keychain(String),
    #[error("invalid input: {0}")]
    Invalid(String),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_failed_serializes_as_string() {
        let e = AppError::AuthFailed(401);
        let s = serde_json::to_string(&e).unwrap();
        assert_eq!(s, "\"auth failed (status 401)\"");
    }
}
