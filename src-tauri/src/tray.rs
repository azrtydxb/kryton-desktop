use tauri::{AppHandle, Emitter, Manager};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem, MenuEvent};
use crate::account_store;
use crate::window_manager;

pub fn create_tray(app: &AppHandle) -> tauri::Result<TrayIcon> {
    let menu = build_tray_menu(app)?;
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(handle_menu_event)
        .build(app)?;
    Ok(tray)
}

fn build_tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let accounts = tauri::async_runtime::block_on(account_store::list_accounts(app.clone()))
        .unwrap_or_default();

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = Vec::new();

    for acc in &accounts {
        let label_open = format!("open-{}", acc.id);
        let label_quick = format!("quick-{}", acc.id);
        let label_sync = format!("sync-{}", acc.id);
        let label_logout = format!("logout-{}", acc.id);
        let submenu = Submenu::with_items(
            app,
            &acc.label,
            true,
            &[
                &MenuItem::with_id(app, &label_open, "Open Window", true, None::<&str>)?,
                &MenuItem::with_id(app, &label_quick, "Quick Switcher", true, None::<&str>)?,
                &MenuItem::with_id(app, &label_sync, "Sync Now", true, None::<&str>)?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(app, &label_logout, "Log Out", true, None::<&str>)?,
            ],
        )?;
        items.push(Box::new(submenu));
    }

    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "tray-add-account",
        "Add Account...",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(MenuItem::with_id(
        app,
        "tray-launcher",
        "Show Launcher",
        true,
        None::<&str>,
    )?));
    items.push(Box::new(PredefinedMenuItem::separator(app)?));
    items.push(Box::new(PredefinedMenuItem::quit(app, None)?));

    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        items.iter().map(|b| b.as_ref()).collect();
    Menu::with_items(app, &item_refs)
}

fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    let id = event.id().0.as_str().to_string();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if id == "tray-launcher" || id == "tray-add-account" {
            let _ = window_manager::open_launcher_window(app_clone).await;
        } else if let Some(rest) = id.strip_prefix("open-") {
            let _ = window_manager::open_account_window(rest.to_string(), app_clone).await;
        } else if let Some(rest) = id.strip_prefix("quick-") {
            // Emit open-quick-switcher to the account window
            if let Some(window) = app_clone.get_webview_window(&format!("account-{}", rest)) {
                let _ = window.set_focus();
                let _ = window.emit("open-quick-switcher", ());
            }
        } else if let Some(rest) = id.strip_prefix("sync-") {
            // Emit a sync event to that account's window
            if let Some(window) = app_clone.get_webview_window(&format!("account-{}", rest)) {
                let _ = window.emit("trigger-sync", ());
            }
        } else if let Some(rest) = id.strip_prefix("logout-") {
            // Emit a logout event
            if let Some(window) = app_clone.get_webview_window(&format!("account-{}", rest)) {
                let _ = window.emit("logout", ());
            }
        }
    });
}

pub fn rebuild_tray(app: &AppHandle, tray: &TrayIcon) -> tauri::Result<()> {
    let menu = build_tray_menu(app)?;
    tray.set_menu(Some(menu))?;
    Ok(())
}
