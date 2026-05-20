use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

/// Check for an update; if one is available, show a native confirm dialog and
/// (on accept) download + install + restart. The settings window also receives
/// an `update:available` event so it can render an inline notice if open.
pub async fn check_and_prompt<R: Runtime>(app: AppHandle<R>) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!("updater unavailable: {e}");
            return;
        }
    };
    let update = match updater.check().await {
        Ok(Some(u)) => u,
        Ok(None) => {
            tracing::info!("no update available");
            return;
        }
        Err(e) => {
            tracing::warn!("update check failed: {e}");
            return;
        }
    };
    let version = update.version.clone();
    tracing::info!("update available: {version}");
    let _ = app.emit("update:available", &version);

    // Native confirm dialog so the prompt works regardless of which (or no)
    // shell window happens to be open.
    let (tx, rx) = std::sync::mpsc::channel::<bool>();
    app.dialog()
        .message(format!(
            "Kryton {version} is available. Install and restart now?"
        ))
        .title("Update available")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Install".into(),
            "Later".into(),
        ))
        .show(move |accepted| {
            let _ = tx.send(accepted);
        });
    let accepted = rx.recv().unwrap_or(false);
    if !accepted {
        tracing::info!("user declined update {version}");
        return;
    }

    let downloaded = Arc::new(AtomicUsize::new(0));
    let downloaded_log = downloaded.clone();
    if let Err(e) = update
        .download_and_install(
            move |chunk, _total| {
                downloaded.fetch_add(chunk, Ordering::Relaxed);
            },
            move || {
                tracing::info!(
                    "downloaded {} bytes",
                    downloaded_log.load(Ordering::Relaxed)
                );
            },
        )
        .await
    {
        tracing::warn!("download_and_install failed: {e}");
        return;
    }
    app.restart();
}

/// Manual "Check for updates now" path exposed to the settings UI.
#[tauri::command]
pub async fn apply_update<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = match updater.check().await.map_err(|e| e.to_string())? {
        Some(u) => u,
        None => return Ok(()),
    };
    let downloaded = Arc::new(AtomicUsize::new(0));
    let downloaded_log = downloaded.clone();
    update
        .download_and_install(
            move |chunk, _total| {
                downloaded.fetch_add(chunk, Ordering::Relaxed);
            },
            move || {
                tracing::info!(
                    "downloaded {} bytes",
                    downloaded_log.load(Ordering::Relaxed)
                );
            },
        )
        .await
        .map_err(|e| e.to_string())?;
    app.restart();
}
