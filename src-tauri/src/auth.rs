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

#[derive(Clone)]
pub struct AuthClient {
    http: Client,
}

impl AuthClient {
    pub fn new() -> AppResult<Self> {
        let http = Client::builder().cookie_store(true).build()?;
        Ok(Self { http })
    }

    pub fn http(&self) -> &Client {
        &self.http
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
    //! Credential store.
    //!
    //! Release builds use the OS keychain via the `keyring` crate (macOS
    //! Keychain, Windows Credential Manager, Linux Secret Service).
    //!
    //! Debug builds use an AES-256-GCM-encrypted file at
    //! `<app_config_dir>/secrets.enc`. The key is a random 32-byte file at
    //! `<app_config_dir>/secrets.key` with permissions `0600`. This is a
    //! deliberate dev-only tradeoff because unsigned dev binaries get a
    //! fresh keychain identity on every `cargo build`, breaking keyring
    //! reads across rebuilds.
    use crate::error::{AppError, AppResult};
    use tauri::{AppHandle, Runtime};
    use uuid::Uuid;

    #[cfg(not(debug_assertions))]
    mod backend {
        use super::*;

        const SERVICE: &str = "app.kryton.desktop";

        fn account(id: &Uuid) -> String {
            id.to_string()
        }

        pub fn store<R: Runtime>(_app: &AppHandle<R>, id: &Uuid, password: &str) -> AppResult<()> {
            tracing::info!("keychain::store service={SERVICE} account={id} len={}", password.len());
            let entry = keyring::Entry::new(SERVICE, &account(id)).map_err(|e| {
                tracing::warn!("keychain::store new failed: {e}");
                AppError::Keychain(e.to_string())
            })?;
            entry.set_password(password).map_err(|e| {
                tracing::warn!("keychain::store set_password failed: {e}");
                AppError::Keychain(e.to_string())
            })?;
            // Immediately read it back to verify it actually landed.
            match entry.get_password() {
                Ok(p) if p == password => tracing::info!("keychain::store verified ({} bytes)", p.len()),
                Ok(p) => tracing::warn!("keychain::store readback mismatch ({} bytes vs {} expected)", p.len(), password.len()),
                Err(e) => tracing::warn!("keychain::store readback failed: {e}"),
            }
            Ok(())
        }

        pub fn read<R: Runtime>(_app: &AppHandle<R>, id: &Uuid) -> AppResult<String> {
            tracing::debug!("keychain::read service={SERVICE} account={id}");
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
                Err(keyring::Error::NoEntry) => Ok(()),
                Err(e) => Err(AppError::Keychain(e.to_string())),
            }
        }
    }

    #[cfg(debug_assertions)]
    mod backend {
        use super::*;
        use aes_gcm::aead::{Aead, KeyInit};
        use aes_gcm::{Aes256Gcm, Nonce};
        use rand::RngCore;
        use std::collections::HashMap;
        use std::path::PathBuf;
        use std::sync::Mutex;
        use tauri::Manager;

        static LOCK: Mutex<()> = Mutex::new(());

        fn dir<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
            let d = app
                .path()
                .app_config_dir()
                .map_err(|e| AppError::Keychain(e.to_string()))?;
            std::fs::create_dir_all(&d)?;
            Ok(d)
        }

        fn key_path<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
            Ok(dir(app)?.join("secrets.key"))
        }

        fn data_path<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
            Ok(dir(app)?.join("secrets.enc"))
        }

        fn read_or_create_key<R: Runtime>(app: &AppHandle<R>) -> AppResult<[u8; 32]> {
            let path = key_path(app)?;
            if path.exists() {
                let bytes = std::fs::read(&path)?;
                if bytes.len() != 32 {
                    return Err(AppError::Keychain("invalid key file length".into()));
                }
                let mut out = [0u8; 32];
                out.copy_from_slice(&bytes);
                return Ok(out);
            }
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            std::fs::write(&path, key)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(
                    &path,
                    std::fs::Permissions::from_mode(0o600),
                );
            }
            Ok(key)
        }

        fn load<R: Runtime>(app: &AppHandle<R>) -> AppResult<HashMap<String, String>> {
            let path = data_path(app)?;
            if !path.exists() {
                return Ok(HashMap::new());
            }
            let blob = std::fs::read(&path)?;
            if blob.len() < 12 {
                return Err(AppError::Keychain("secrets file too short".into()));
            }
            let (nonce_bytes, ciphertext) = blob.split_at(12);
            let key = read_or_create_key(app)?;
            let cipher = Aes256Gcm::new_from_slice(&key)
                .map_err(|e| AppError::Keychain(e.to_string()))?;
            let plaintext = cipher
                .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
                .map_err(|e| AppError::Keychain(format!("decrypt: {e}")))?;
            Ok(serde_json::from_slice(&plaintext).unwrap_or_default())
        }

        fn save<R: Runtime>(
            app: &AppHandle<R>,
            map: &HashMap<String, String>,
        ) -> AppResult<()> {
            let key = read_or_create_key(app)?;
            let cipher = Aes256Gcm::new_from_slice(&key)
                .map_err(|e| AppError::Keychain(e.to_string()))?;
            let mut nonce_bytes = [0u8; 12];
            rand::thread_rng().fill_bytes(&mut nonce_bytes);
            let plaintext = serde_json::to_vec(map)?;
            let ciphertext = cipher
                .encrypt(Nonce::from_slice(&nonce_bytes), plaintext.as_ref())
                .map_err(|e| AppError::Keychain(format!("encrypt: {e}")))?;
            let mut blob = nonce_bytes.to_vec();
            blob.extend_from_slice(&ciphertext);
            let path = data_path(app)?;
            let tmp = path.with_extension("enc.tmp");
            std::fs::write(&tmp, &blob)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ =
                    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
            }
            std::fs::rename(&tmp, &path)?;
            Ok(())
        }

        pub fn store<R: Runtime>(app: &AppHandle<R>, id: &Uuid, password: &str) -> AppResult<()> {
            let _g = LOCK.lock().unwrap();
            let mut map = load(app)?;
            map.insert(id.to_string(), password.to_string());
            save(app, &map)
        }

        pub fn read<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<String> {
            let _g = LOCK.lock().unwrap();
            let map = load(app)?;
            map.get(&id.to_string())
                .cloned()
                .ok_or_else(|| AppError::Keychain("no password for account".into()))
        }

        pub fn delete<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<()> {
            let _g = LOCK.lock().unwrap();
            let mut map = load(app)?;
            map.remove(&id.to_string());
            save(app, &map)
        }
    }

    pub fn store<R: Runtime>(app: &AppHandle<R>, id: &Uuid, password: &str) -> AppResult<()> {
        backend::store(app, id, password)
    }

    pub fn read<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<String> {
        backend::read(app, id)
    }

    pub fn delete<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<()> {
        backend::delete(app, id)
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
