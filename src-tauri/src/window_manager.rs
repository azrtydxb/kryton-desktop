use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

#[tauri::command]
pub async fn open_account_window(
    account_id: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let label = format!("account-{}", account_id);
    if let Some(existing) = app.get_webview_window(&label) {
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    let url = WebviewUrl::App(format!("index.html?account={}", account_id).into());
    WebviewWindowBuilder::new(&app, &label, url)
        .title(format!("Kryton — {}", account_id))
        .inner_size(1200.0, 800.0)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_launcher_window(app: tauri::AppHandle) -> Result<(), String> {
    let label = "launcher";
    if let Some(existing) = app.get_webview_window(label) {
        existing.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    let url = WebviewUrl::App("index.html?launcher=1".into());
    WebviewWindowBuilder::new(&app, label, url)
        .title("Kryton")
        .inner_size(700.0, 500.0)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}
