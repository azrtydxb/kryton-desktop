use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[tauri::command]
pub async fn read_db(account_id: String, app_handle: tauri::AppHandle) -> Result<Vec<u8>, String> {
    let path = account_db_path(&app_handle, &account_id)?;
    if !path.exists() {
        return Ok(vec![]);
    }
    fs::read(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_db(
    account_id: String,
    bytes: Vec<u8>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let path = account_db_path(&app_handle, &account_id)?;
    let parent = path.parent().ok_or("no parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("db.tmp");
    fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

fn account_db_path(app: &tauri::AppHandle, account_id: &str) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(dir.join("accounts").join(account_id).join("kryton.db"))
}
