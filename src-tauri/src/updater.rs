use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_updater::UpdaterExt;

pub async fn check_and_prompt<R: Runtime>(app: AppHandle<R>) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!("updater unavailable: {e}");
            return;
        }
    };
    match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!("update available: {}", update.version);
            let _ = app.emit("update:available", &update.version);
        }
        Ok(None) => tracing::info!("no update available"),
        Err(e) => tracing::warn!("update check failed: {e}"),
    }
}

#[tauri::command]
pub async fn apply_update<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        let downloaded = Arc::new(AtomicUsize::new(0));
        let downloaded_fin = downloaded.clone();
        update
            .download_and_install(
                |chunk, _total| {
                    downloaded.fetch_add(chunk, Ordering::Relaxed);
                },
                move || {
                    tracing::info!(
                        "downloaded {} bytes",
                        downloaded_fin.load(Ordering::Relaxed)
                    );
                },
            )
            .await
            .map_err(|e| e.to_string())?;
        app.restart();
    }
    Ok(())
}
