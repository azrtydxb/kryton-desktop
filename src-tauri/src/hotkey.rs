use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub fn register_default(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut()
        .on_shortcut("CmdOrCtrl+Shift+K", |app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    // Collect all windows first, then find focused one.
                    let windows: Vec<_> = app_clone.webview_windows().into_values().collect();
                    let focused = windows.iter().find(|w| w.is_focused().unwrap_or(false)).cloned();
                    if let Some(w) = focused.or_else(|| windows.into_iter().next()) {
                        let _ = w.set_focus();
                        let _ = w.emit("open-quick-switcher", ());
                    }
                });
            }
        })?;
    Ok(())
}
