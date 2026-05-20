pub mod accounts;
pub mod auth;
pub mod error;
pub mod ipc;
pub mod menu;
pub mod tray;
pub mod window_mgr;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

pub(crate) fn open_settings_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("src/settings/index.html".into()),
    )
    .title("Kryton — Settings")
    .inner_size(640.0, 480.0)
    .build()?;
    Ok(())
}

pub(crate) fn open_add_server_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("login") {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        "login",
        WebviewUrl::App("src/login/index.html".into()),
    )
    .title("Kryton — Add Server")
    .inner_size(420.0, 540.0)
    .resizable(false)
    .build()?;
    Ok(())
}

pub(crate) fn open_quick_capture_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window("quick-capture") {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        "quick-capture",
        WebviewUrl::App("src/quick-capture/index.html".into()),
    )
    .title("Quick Capture")
    .inner_size(480.0, 320.0)
    .always_on_top(true)
    .build()?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            ipc::list_accounts,
            ipc::login_and_add,
            ipc::silent_relogin,
            ipc::remove_account,
            ipc::open_server,
            ipc::switch_to,
            ipc::close_server,
            ipc::refresh_menu,
            ipc::open_add_server,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let state = ipc::AppState::init(&handle)?;
            let (accounts, default_active) = {
                let f = state.accounts.lock().unwrap();
                (f.accounts.clone(), f.default_active)
            };
            app.manage(state);

            if accounts.is_empty() {
                WebviewWindowBuilder::new(
                    app,
                    "login",
                    WebviewUrl::App("src/login/index.html".into()),
                )
                .title("Kryton")
                .inner_size(420.0, 540.0)
                .resizable(false)
                .build()?;
            } else {
                for a in &accounts {
                    let h = handle.clone();
                    let aid = a.id;
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = ipc::do_silent_relogin(h, aid).await {
                            tracing::warn!("silent relogin failed for {aid}: {e}");
                        }
                    });
                }
                for a in &accounts {
                    window_mgr::open_or_focus(&handle, a)?;
                }
                let active_id = default_active.unwrap_or(accounts[0].id);
                window_mgr::hide_all_except(&handle, &active_id);
            }

            // Build and attach the native menu bar
            let accounts_snapshot = {
                let st: tauri::State<'_, ipc::AppState> = handle.state();
                let guard = st.accounts.lock().unwrap();
                guard.accounts.clone()
            };
            let menu = menu::build(&handle, &accounts_snapshot)?;
            app.set_menu(menu)?;

            // Install system tray
            tray::install(&handle, &accounts_snapshot)?;

            let menu_handle = handle.clone();
            app.on_menu_event(move |_app, ev| {
                let id = ev.id().0.clone();
                let h = menu_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if id == "settings" {
                        let _ = open_settings_window(&h);
                    } else if id == "new_capture" {
                        let _ = open_quick_capture_window(&h);
                    } else if id == "add_server" {
                        let _ = open_add_server_window(&h);
                    } else if id == "reload" {
                        // Find a focused window and reload it
                        for (_label, win) in h.webview_windows() {
                            if win.is_focused().unwrap_or(false) {
                                let _ = win.eval("window.location.reload()");
                                break;
                            }
                        }
                    } else if let Some(rest) = id.strip_prefix("switch:") {
                        if let Ok(uuid) = rest.parse::<uuid::Uuid>() {
                            let st: tauri::State<'_, ipc::AppState> = h.state();
                            let _ = ipc::switch_to(h.clone(), st, uuid).await;
                        }
                    }
                });
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Kryton");
}
