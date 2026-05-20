use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

pub fn register<R: Runtime>(app: &AppHandle<R>, accel: &str) -> tauri::Result<()> {
    let shortcut: Shortcut = accel
        .parse::<Shortcut>()
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("{}", e)))?;
    let h = app.clone();
    app.global_shortcut()
        .on_shortcut(
            shortcut,
            move |_app: &AppHandle<R>, _sc: &Shortcut, ev: ShortcutEvent| {
                if ev.state() == ShortcutState::Pressed {
                    let _ = crate::open_quick_capture_window(&h);
                }
            },
        )
        .map_err(|e| tauri::Error::Anyhow(anyhow::anyhow!("{}", e)))?;
    Ok(())
}
