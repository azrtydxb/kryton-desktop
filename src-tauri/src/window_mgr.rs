use crate::accounts::Account;
use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

pub fn label_for(id: &Uuid) -> String {
    format!("server-{id}")
}

pub fn data_dir<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<PathBuf> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Invalid(e.to_string()))?;
    let dir = base.join("webview-data").join(id.to_string());
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn open_or_focus<R: Runtime>(app: &AppHandle<R>, acct: &Account) -> AppResult<()> {
    let label = label_for(&acct.id);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    let dir = data_dir(app, &acct.id)?;
    let url: url::Url = acct.server_url.parse()?;
    WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .title(format!("Kryton — {}", acct.label))
        .inner_size(1200.0, 800.0)
        .data_directory(dir)
        .build()
        .map_err(|e| AppError::Invalid(e.to_string()))?;
    Ok(())
}

pub fn hide_all_except<R: Runtime>(app: &AppHandle<R>, id: &Uuid) {
    let keep = label_for(id);
    for (label, win) in app.webview_windows() {
        if label.starts_with("server-") && label != keep {
            let _ = win.hide();
        }
    }
}

pub fn close<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<()> {
    let label = label_for(id);
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| AppError::Invalid(e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_for_is_stable_per_id() {
        assert_eq!(
            label_for(&Uuid::nil()),
            "server-00000000-0000-0000-0000-000000000000"
        );
    }
}
