mod account_store;
mod auth_storage;
mod db_io;
mod menu;
mod window_manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Build and set the native menu bar.
            let app_menu = menu::build_menu(app.handle())?;
            app.set_menu(app_menu)?;

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = window_manager::open_launcher_window(handle).await;
            });
            Ok(())
        })
        .on_menu_event(|app, event| {
            menu::emit_menu_action(app, event.id().as_ref());
        })
        .invoke_handler(tauri::generate_handler![
            db_io::read_db,
            db_io::write_db,
            account_store::list_accounts,
            account_store::add_account,
            account_store::remove_account,
            account_store::rename_account,
            account_store::touch_account,
            window_manager::open_account_window,
            window_manager::open_launcher_window,
            auth_storage::get_auth_token,
            auth_storage::set_auth_token,
            auth_storage::clear_auth_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
