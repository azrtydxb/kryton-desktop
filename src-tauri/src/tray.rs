use crate::accounts::Account;
use tauri::{
    menu::{Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager, Runtime,
};

const TRAY_ID: &str = "main";

/// Build a new tray menu reflecting the current account list.
fn build_menu<R: Runtime>(app: &AppHandle<R>, accounts: &[Account]) -> tauri::Result<Menu<R>> {
    let mut builder = MenuBuilder::new(app);

    if accounts.is_empty() {
        builder = builder.item(
            &MenuItemBuilder::with_id("tray_no_servers", "No servers")
                .enabled(false)
                .build(app)?,
        );
    } else {
        for acct in accounts {
            let id = format!("tray_switch:{}", acct.id);
            builder = builder.item(&MenuItemBuilder::with_id(id, &acct.label).build(app)?);
        }
    }

    builder = builder
        .separator()
        .item(
            &MenuItemBuilder::with_id("tray_quick_capture", "Quick Capture…").build(app)?,
        )
        .item(&MenuItemBuilder::with_id("tray_settings", "Settings…").build(app)?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, None)?);

    builder.build()
}

/// Create and register the tray icon. Call once during setup.
pub fn install<R: Runtime>(
    app: &AppHandle<R>,
    accounts: &[Account],
) -> tauri::Result<TrayIcon<R>> {
    let menu = build_menu(app, accounts)?;
    let h = app.clone();

    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app: &AppHandle<R>, ev: tauri::menu::MenuEvent| {
            let id = ev.id().0.clone();
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                handle_tray_event(&app, &id).await;
            });
        })
        .build(app)?;

    let _ = h; // h captured in closure above
    Ok(tray)
}

/// Rebuild the tray menu in-place. Call after any account change.
pub fn refresh<R: Runtime>(app: &AppHandle<R>, accounts: &[Account]) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let menu = build_menu(app, accounts)?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

async fn handle_tray_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    if id == "tray_quick_capture" {
        let _ = crate::open_quick_capture_window(app);
    } else if id == "tray_settings" {
        let _ = crate::open_settings_window(app);
    } else if let Some(rest) = id.strip_prefix("tray_switch:") {
        if let Ok(uuid) = rest.parse::<uuid::Uuid>() {
            let st: tauri::State<'_, crate::ipc::AppState> = app.state();
            let _ = crate::ipc::switch_to(app.clone(), st, uuid).await;
        }
    }
}
