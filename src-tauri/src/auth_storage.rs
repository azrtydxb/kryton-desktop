use std::fs;
use tauri::Manager;

fn token_path(
    app: &tauri::AppHandle,
    account_id: &str,
) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let tokens_dir = dir.join("tokens");
    fs::create_dir_all(&tokens_dir).map_err(|e| e.to_string())?;
    Ok(tokens_dir.join(format!("{}.tok", account_id)))
}

#[tauri::command]
pub async fn get_auth_token(
    account_id: String,
    app: tauri::AppHandle,
) -> Result<Option<String>, String> {
    let p = token_path(&app, &account_id)?;
    if !p.exists() {
        return Ok(None);
    }
    fs::read_to_string(&p).map(Some).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_auth_token(
    account_id: String,
    token: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let p = token_path(&app, &account_id)?;
    fs::write(&p, &token).map_err(|e| e.to_string())?;
    // restrict to owner read/write on unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&p, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_auth_token(
    account_id: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let p = token_path(&app, &account_id)?;
    let _ = fs::remove_file(&p);
    Ok(())
}
