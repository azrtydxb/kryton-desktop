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
        ])
        .setup(|app| {
            let state = ipc::AppState::init(app.handle())?;
            app.manage(state);
            WebviewWindowBuilder::new(
                app,
                "login",
                WebviewUrl::App("src/login/index.html".into()),
            )
            .title("Kryton")
            .inner_size(420.0, 540.0)
            .resizable(false)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Kryton");
}
