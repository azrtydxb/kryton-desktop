use crate::auth::AuthClient;
use crate::error::{AppError, AppResult};
use serde::Deserialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_notification::NotificationExt;
use uuid::Uuid;

const STREAM_PATH: &str = "/api/notifications/stream";

#[derive(Debug, Deserialize)]
pub struct NotifyArgs {
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamEvent {
    #[serde(default)]
    title: String,
    #[serde(default)]
    body: Option<String>,
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

/// Subscribe to a server's notifications SSE stream.
///
/// Contract documented in [`docs/superpowers/specs/2026-05-20-desktop-app-design.md`].
///
/// Behavior:
/// - On 200, parse `data:` SSE lines as `StreamEvent` and call [`notify`].
/// - On 404 (endpoint not yet shipped), log once and exit. Never retried.
/// - On other errors, log and exit. The next launch will retry.
///
/// The subscriber runs forever (until app exit or 404) — `tauri::async_runtime::spawn`
/// the call from the caller.
pub async fn subscribe<R: Runtime>(
    app: AppHandle<R>,
    account_id: Uuid,
    server_url: String,
    client: AuthClient,
) {
    let url = format!("{}{}", server_url.trim_end_matches('/'), STREAM_PATH);
    let resp = match client
        .http()
        .get(&url)
        .header("accept", "text/event-stream")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("notifier[{account_id}]: stream connect failed: {e}");
            return;
        }
    };
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        tracing::info!("notifier[{account_id}]: /api/notifications/stream not present; skipping");
        return;
    }
    if !resp.status().is_success() {
        tracing::warn!(
            "notifier[{account_id}]: stream returned status {}",
            resp.status()
        );
        return;
    }

    use futures_util::StreamExt;
    let mut buf = String::new();
    let byte_stream = resp.bytes_stream();
    tokio::pin!(byte_stream);
    while let Some(chunk) = byte_stream.next().await {
        let chunk = match chunk {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("notifier[{account_id}]: stream read failed: {e}");
                return;
            }
        };
        buf.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(idx) = buf.find("\n\n") {
            let frame = buf[..idx].to_string();
            buf = buf[idx + 2..].to_string();
            for line in frame.lines() {
                if let Some(json) = line.strip_prefix("data:") {
                    let json = json.trim();
                    if json.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<StreamEvent>(json) {
                        Ok(ev) => {
                            let _ = notify(
                                &app,
                                NotifyArgs {
                                    title: ev.title,
                                    body: ev.body,
                                },
                            );
                        }
                        Err(e) => {
                            tracing::debug!("notifier[{account_id}]: bad event json: {e}");
                        }
                    }
                }
            }
        }
    }
}
