use crate::error::{AppError, AppResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const LOGIN_PATH: &str = "/api/auth/sign-in/email";

#[derive(Debug, Serialize)]
struct LoginBody<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct LoginResponse {
    pub user: Option<LoginUser>,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
}

pub struct AuthClient {
    http: Client,
}

impl AuthClient {
    pub fn new() -> AppResult<Self> {
        let http = Client::builder().cookie_store(true).build()?;
        Ok(Self { http })
    }

    pub async fn login(
        &self,
        server_url: &str,
        username: &str,
        password: &str,
    ) -> AppResult<LoginResponse> {
        let url = format!("{}{}", server_url.trim_end_matches('/'), LOGIN_PATH);
        let resp = self
            .http
            .post(&url)
            .json(&LoginBody {
                email: username,
                password,
            })
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(AppError::AuthFailed(resp.status().as_u16()));
        }
        Ok(resp
            .json::<LoginResponse>()
            .await
            .unwrap_or(LoginResponse { user: None }))
    }
}

pub mod keychain {
    use crate::error::{AppError, AppResult};
    use tauri::{AppHandle, Runtime};
    use uuid::Uuid;

    const SERVICE: &str = "app.kryton.desktop";

    fn account(id: &Uuid) -> String {
        id.to_string()
    }

    pub fn store<R: Runtime>(_app: &AppHandle<R>, id: &Uuid, password: &str) -> AppResult<()> {
        let entry = keyring::Entry::new(SERVICE, &account(id))
            .map_err(|e| AppError::Keychain(e.to_string()))?;
        entry
            .set_password(password)
            .map_err(|e| AppError::Keychain(e.to_string()))
    }

    pub fn read<R: Runtime>(_app: &AppHandle<R>, id: &Uuid) -> AppResult<String> {
        let entry = keyring::Entry::new(SERVICE, &account(id))
            .map_err(|e| AppError::Keychain(e.to_string()))?;
        entry
            .get_password()
            .map_err(|e| AppError::Keychain(e.to_string()))
    }

    pub fn delete<R: Runtime>(_app: &AppHandle<R>, id: &Uuid) -> AppResult<()> {
        let entry = keyring::Entry::new(SERVICE, &account(id))
            .map_err(|e| AppError::Keychain(e.to_string()))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            // Treat "no entry" as success — idempotent delete.
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::Keychain(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn login_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(LOGIN_PATH))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"user":{"email":"u@example.com"}})),
            )
            .mount(&server)
            .await;
        let c = AuthClient::new().unwrap();
        let r = c.login(&server.uri(), "u@example.com", "p").await.unwrap();
        assert_eq!(
            r.user.as_ref().and_then(|u| u.email.as_deref()),
            Some("u@example.com")
        );
    }

    #[tokio::test]
    async fn login_401_maps_to_auth_failed() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(LOGIN_PATH))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let c = AuthClient::new().unwrap();
        let err = c.login(&server.uri(), "u@example.com", "p").await.unwrap_err();
        assert!(matches!(err, AppError::AuthFailed(401)));
    }
}
