use crate::error::{AppError, AppResult};
use serde::Deserialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;

#[derive(Debug, Deserialize)]
pub struct NotifyArgs {
    pub title: String,
    pub body: Option<String>,
}

pub fn notify<R: Runtime>(app: &AppHandle<R>, args: NotifyArgs) -> AppResult<()> {
    let mut builder = app.notification().builder().title(&args.title);
    if let Some(b) = args.body {
        builder = builder.body(b);
    }
    builder
        .show()
        .map_err(|e| AppError::Invalid(e.to_string()))?;
    Ok(())
}
