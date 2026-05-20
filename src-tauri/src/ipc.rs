use crate::accounts::{self, Account, AccountsFile};
use crate::auth::AuthClient;
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use uuid::Uuid;

pub struct AppState {
    pub file_path: PathBuf,
    pub accounts: Mutex<AccountsFile>,
    pub auth_clients: Mutex<HashMap<Uuid, AuthClient>>,
}

impl AppState {
    pub fn init<R: Runtime>(app: &AppHandle<R>) -> AppResult<Self> {
        let dir = app
            .path()
            .app_config_dir()
            .map_err(|e| AppError::Invalid(e.to_string()))?;
        let file_path = dir.join("accounts.json");
        let file = accounts::load(&file_path)?;
        Ok(Self {
            file_path,
            accounts: Mutex::new(file),
            auth_clients: Mutex::new(HashMap::new()),
        })
    }

    pub fn save(&self) -> AppResult<()> {
        let f = self.accounts.lock().unwrap();
        accounts::save(&self.file_path, &f)
    }

    pub fn auth_for(&self, id: Uuid) -> AppResult<AuthClient> {
        let mut clients = self.auth_clients.lock().unwrap();
        if let Some(c) = clients.get(&id) {
            return Ok(c.clone());
        }
        let c = AuthClient::new()?;
        clients.insert(id, c.clone());
        Ok(c)
    }
}

#[tauri::command]
pub fn list_accounts(state: State<'_, AppState>) -> Vec<Account> {
    state.accounts.lock().unwrap().accounts.clone()
}

#[tauri::command]
pub async fn login_and_add<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    label: String,
    server_url: String,
    username: String,
    password: String,
) -> AppResult<Account> {
    let acct = {
        let mut f = state.accounts.lock().unwrap();
        accounts::add(&mut f, label, server_url.clone(), username.clone())
    };
    let client = state.auth_for(acct.id)?;
    if let Err(e) = client.login(&server_url, &username, &password).await {
        // Rollback the just-added account.
        let mut f = state.accounts.lock().unwrap();
        let _ = accounts::remove(&mut f, acct.id);
        return Err(e);
    }
    state.save()?;
    store_password(&app, &acct.id, &password)?;
    Ok(acct)
}

#[tauri::command]
pub async fn silent_relogin<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    account_id: Uuid,
) -> AppResult<()> {
    let acct = {
        let f = state.accounts.lock().unwrap();
        f.accounts
            .iter()
            .find(|a| a.id == account_id)
            .cloned()
            .ok_or_else(|| AppError::AccountNotFound(account_id.to_string()))?
    };
    let password = read_password(&app, &acct.id)?;
    state
        .auth_for(account_id)?
        .login(&acct.server_url, &acct.username, &password)
        .await?;
    Ok(())
}

#[tauri::command]
pub fn remove_account<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    account_id: Uuid,
) -> AppResult<()> {
    // Close the per-server window first so the OS releases data-dir file handles.
    let _ = crate::window_mgr::close(&app, &account_id);
    {
        let mut f = state.accounts.lock().unwrap();
        accounts::remove(&mut f, account_id)?;
    }
    {
        let mut clients = state.auth_clients.lock().unwrap();
        clients.remove(&account_id);
    }
    state.save()?;
    let _ = crate::auth::keychain::delete(&app, &account_id);
    if let Ok(dir) = crate::window_mgr::data_dir(&app, &account_id) {
        let _ = std::fs::remove_dir_all(&dir);
    }
    Ok(())
}

#[tauri::command]
pub fn get_theme(state: State<'_, AppState>) -> String {
    state.accounts.lock().unwrap().settings.theme.clone()
}

#[tauri::command]
pub fn set_theme<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    theme: String,
) -> AppResult<()> {
    {
        let mut f = state.accounts.lock().unwrap();
        f.settings.theme = theme.clone();
    }
    state.save()?;
    let _ = app.emit("theme:changed", &theme);
    Ok(())
}

#[tauri::command]
pub fn refresh_menu<R: Runtime>(app: AppHandle<R>, state: State<'_, AppState>) -> AppResult<()> {
    let accounts = state.accounts.lock().unwrap().accounts.clone();
    let menu = crate::menu::build(&app, &accounts)
        .map_err(|e| AppError::Invalid(e.to_string()))?;
    app.set_menu(menu).map_err(|e| AppError::Invalid(e.to_string()))?;
    let _ = crate::tray::refresh(&app, &accounts);
    Ok(())
}

#[tauri::command]
pub fn open_add_server<R: Runtime>(app: AppHandle<R>) -> AppResult<()> {
    crate::open_add_server_window(&app).map_err(|e| AppError::Invalid(e.to_string()))
}

fn store_password<R: Runtime>(app: &AppHandle<R>, id: &Uuid, password: &str) -> AppResult<()> {
    crate::auth::keychain::store(app, id, password)
}

fn read_password<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<String> {
    crate::auth::keychain::read(app, id)
}

/// Free helper that obtains AppState from the AppHandle directly.
/// Used from async spawn tasks where `tauri::State<'_>` lifetimes are unavailable.
pub async fn do_silent_relogin<R: Runtime>(app: AppHandle<R>, id: Uuid) -> AppResult<()> {
    let app_state = app.state::<AppState>();
    let acct = {
        let f = app_state.accounts.lock().unwrap();
        f.accounts
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or_else(|| AppError::AccountNotFound(id.to_string()))?
    };
    let password = read_password(&app, &acct.id)?;
    app_state
        .auth_for(id)?
        .login(&acct.server_url, &acct.username, &password)
        .await?;
    Ok(())
}

const DAILY_PATH: &str = "/api/daily";

#[derive(Deserialize)]
struct DailyNote {
    path: String,
    content: String,
}

#[tauri::command]
pub async fn capture_to_active<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    markdown: String,
) -> AppResult<()> {
    let acct = {
        let f = state.accounts.lock().unwrap();
        let id = f
            .default_active
            .ok_or_else(|| AppError::Invalid("no active server".into()))?;
        f.accounts
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or_else(|| AppError::AccountNotFound(id.to_string()))?
    };
    let base = acct.server_url.trim_end_matches('/');
    let http = state.auth_for(acct.id)?.http().clone();

    let daily: DailyNote = http
        .post(format!("{base}{DAILY_PATH}"))
        .header("content-type", "application/json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let new_content = if daily.content.is_empty() {
        markdown
    } else {
        format!("{}\n\n{}", daily.content.trim_end_matches('\n'), markdown)
    };

    let put_url = format!("{base}/api/notes/{}", daily.path);
    http.put(&put_url)
        .json(&serde_json::json!({"content": new_content}))
        .send()
        .await?
        .error_for_status()?;

    let _ = app.emit("capture:saved", acct.id.to_string());
    Ok(())
}

use crate::window_mgr;

#[tauri::command]
pub async fn open_server<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    account_id: Uuid,
) -> AppResult<()> {
    let acct = {
        let f = state.accounts.lock().unwrap();
        f.accounts
            .iter()
            .find(|a| a.id == account_id)
            .cloned()
            .ok_or_else(|| AppError::AccountNotFound(account_id.to_string()))?
    };
    window_mgr::open_or_focus(&app, &acct)?;
    if let Some(w) = app.get_webview_window("login") {
        let _ = w.close();
    }
    Ok(())
}

#[tauri::command]
pub async fn switch_to<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    account_id: Uuid,
) -> AppResult<()> {
    let acct = {
        let mut f = state.accounts.lock().unwrap();
        crate::accounts::touch(&mut f, account_id);
        f.default_active = Some(account_id);
        f.accounts
            .iter()
            .find(|a| a.id == account_id)
            .cloned()
            .ok_or_else(|| AppError::AccountNotFound(account_id.to_string()))?
    };
    state.save()?;
    window_mgr::open_or_focus(&app, &acct)?;
    window_mgr::hide_all_except(&app, &account_id);
    Ok(())
}

#[tauri::command]
pub async fn close_server<R: Runtime>(
    app: AppHandle<R>,
    account_id: Uuid,
) -> AppResult<()> {
    window_mgr::close(&app, &account_id)
}

use crate::notifier::{notify, NotifyArgs};

#[derive(serde::Serialize)]
pub struct WebviewCreds {
    pub email: String,
    pub password: String,
}

#[tauri::command]
pub fn webview_creds<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    account_id: Uuid,
) -> AppResult<WebviewCreds> {
    let username = {
        let f = state.accounts.lock().unwrap();
        f.accounts.iter().find(|a| a.id == account_id).cloned()
            .ok_or_else(|| AppError::AccountNotFound(account_id.to_string()))?
            .username
    };
    let password = crate::auth::keychain::read(&app, &account_id)?;
    Ok(WebviewCreds { email: username, password })
}

#[tauri::command]
pub fn notify_from_web<R: Runtime>(app: AppHandle<R>, args: NotifyArgs) -> AppResult<()> {
    notify(&app, args)
}
