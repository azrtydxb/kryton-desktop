mod account_store;
mod auth_storage;
mod db_io;
mod hotkey;
mod menu;
mod tray;
mod window_manager;

// Phase B: Tier 3 native integrations
#[cfg(target_os = "macos")]
mod spotlight;
#[cfg(target_os = "windows")]
mod win_search;
#[cfg(target_os = "windows")]
mod jumplist;

use tauri::{tray::TrayIcon, Manager};

#[tauri::command]
async fn refresh_tray(app: tauri::AppHandle) -> Result<(), String> {
    let tray = app.state::<TrayIcon>();
    tray::rebuild_tray(&app, &tray).map_err(|e| e.to_string())
}

// --- Spotlight commands (macOS) ---

#[tauri::command]
#[cfg(target_os = "macos")]
async fn spotlight_index(
    account_id: String,
    note_path: String,
    title: String,
    snippet: String,
) -> Result<(), String> {
    spotlight::index_note(&account_id, &note_path, &title, &snippet)
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
async fn spotlight_index(
    _account_id: String,
    _note_path: String,
    _title: String,
    _snippet: String,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
#[cfg(target_os = "macos")]
async fn spotlight_remove(account_id: String, note_path: String) -> Result<(), String> {
    spotlight::remove_note(&account_id, &note_path)
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
async fn spotlight_remove(_account_id: String, _note_path: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
#[cfg(target_os = "macos")]
async fn spotlight_clear_account(account_id: String) -> Result<(), String> {
    spotlight::clear_account(&account_id)
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
async fn spotlight_clear_account(_account_id: String) -> Result<(), String> {
    Ok(())
}

// --- Windows Search commands ---

#[tauri::command]
#[cfg(target_os = "windows")]
async fn win_search_upsert_note(
    account_id: String,
    note_path: String,
    title: String,
) -> Result<(), String> {
    win_search::upsert_note(&account_id, &note_path, &title)
}

#[tauri::command]
#[cfg(not(target_os = "windows"))]
async fn win_search_upsert_note(
    _account_id: String,
    _note_path: String,
    _title: String,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
#[cfg(target_os = "windows")]
async fn win_search_remove_note(
    account_id: String,
    title: String,
) -> Result<(), String> {
    win_search::remove_note(&account_id, &title)
}

#[tauri::command]
#[cfg(not(target_os = "windows"))]
async fn win_search_remove_note(_account_id: String, _title: String) -> Result<(), String> {
    Ok(())
}

// --- Jumplist commands (Windows) ---

#[derive(serde::Deserialize)]
pub struct JumplistNote {
    pub title: String,
    pub account_id: String,
    pub note_path: String,
}

#[tauri::command]
#[cfg(target_os = "windows")]
async fn update_jumplist(
    account_id: String,
    recent_notes: Vec<JumplistNote>,
) -> Result<(), String> {
    let notes: Vec<jumplist::RecentNote> = recent_notes
        .into_iter()
        .map(|n| jumplist::RecentNote {
            title: n.title,
            account_id: n.account_id,
            note_path: n.note_path,
        })
        .collect();
    jumplist::update_jumplist(&account_id, &notes)
}

#[tauri::command]
#[cfg(not(target_os = "windows"))]
async fn update_jumplist(
    _account_id: String,
    _recent_notes: Vec<JumplistNote>,
) -> Result<(), String> {
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
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

            // Create system tray and store it in app state so refresh_tray can access it.
            let tray_icon = tray::create_tray(app.handle())?;
            app.manage(tray_icon);

            // Register global hotkey ⌘⇧K / Ctrl+Shift+K.
            if let Err(e) = hotkey::register_default(app.handle()) {
                log::warn!("Failed to register global hotkey: {}", e);
            }

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
            refresh_tray,
            // Phase B: Tier 3 native
            spotlight_index,
            spotlight_remove,
            spotlight_clear_account,
            win_search_upsert_note,
            win_search_remove_note,
            update_jumplist,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
