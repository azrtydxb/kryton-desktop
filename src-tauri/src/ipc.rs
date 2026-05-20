use crate::accounts::{self, Account, AccountsFile};
use crate::auth::AuthClient;
use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime, State};
use uuid::Uuid;

pub struct AppState {
    pub file_path: PathBuf,
    pub accounts: Mutex<AccountsFile>,
    pub auth: AuthClient,
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
            auth: AuthClient::new()?,
        })
    }

    pub fn save(&self) -> AppResult<()> {
        let f = self.accounts.lock().unwrap();
        accounts::save(&self.file_path, &f)
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
    state.auth.login(&server_url, &username, &password).await?;
    let acct = {
        let mut f = state.accounts.lock().unwrap();
        accounts::add(&mut f, label, server_url, username)
    };
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
        .auth
        .login(&acct.server_url, &acct.username, &password)
        .await?;
    Ok(())
}

#[tauri::command]
pub fn remove_account(state: State<'_, AppState>, account_id: Uuid) -> AppResult<()> {
    {
        let mut f = state.accounts.lock().unwrap();
        accounts::remove(&mut f, account_id)?;
    }
    state.save()?;
    Ok(())
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
        .auth
        .login(&acct.server_url, &acct.username, &password)
        .await?;
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
