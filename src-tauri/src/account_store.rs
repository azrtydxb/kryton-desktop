use serde::{Deserialize, Serialize};
use std::fs;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    pub id: String,
    pub label: String,
    pub server_url: String,
    pub last_logged_in_at: i64,
}

#[derive(Serialize, Deserialize, Default)]
struct AccountStore {
    accounts: Vec<Account>,
}

fn store_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("accounts.json"))
}

fn load(app: &tauri::AppHandle) -> Result<AccountStore, String> {
    let path = store_path(app)?;
    if !path.exists() {
        return Ok(AccountStore::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn save(app: &tauri::AppHandle, store: &AccountStore) -> Result<(), String> {
    let path = store_path(app)?;
    let text = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_accounts(app: tauri::AppHandle) -> Result<Vec<Account>, String> {
    Ok(load(&app)?.accounts)
}

#[tauri::command]
pub async fn add_account(
    label: String,
    server_url: String,
    app: tauri::AppHandle,
) -> Result<Account, String> {
    let mut store = load(&app)?;
    let id = format!("acc_{}", uuid::Uuid::new_v4().simple());
    let acc = Account {
        id: id.clone(),
        label,
        server_url,
        last_logged_in_at: chrono::Utc::now().timestamp(),
    };
    store.accounts.push(acc.clone());
    save(&app, &store)?;
    Ok(acc)
}

#[tauri::command]
pub async fn remove_account(account_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let mut store = load(&app)?;
    store.accounts.retain(|a| a.id != account_id);
    save(&app, &store)?;
    Ok(())
}

#[tauri::command]
pub async fn rename_account(
    account_id: String,
    new_label: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut store = load(&app)?;
    for a in &mut store.accounts {
        if a.id == account_id {
            a.label = new_label.clone();
        }
    }
    save(&app, &store)?;
    Ok(())
}

#[tauri::command]
pub async fn touch_account(account_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let mut store = load(&app)?;
    for a in &mut store.accounts {
        if a.id == account_id {
            a.last_logged_in_at = chrono::Utc::now().timestamp();
        }
    }
    save(&app, &store)?;
    Ok(())
}
