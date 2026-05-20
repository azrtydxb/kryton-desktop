pub mod accounts;
pub mod auth;
pub mod error;
pub mod ipc;
pub mod window_mgr;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Kryton");
}
