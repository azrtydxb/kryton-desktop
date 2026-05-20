use crate::accounts::Account;
use tauri::{
    menu::{Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    AppHandle, Runtime,
};

pub fn build<R: Runtime>(app: &AppHandle<R>, accounts: &[Account]) -> tauri::Result<Menu<R>> {
    // App submenu ("Kryton" on macOS)
    let app_submenu = SubmenuBuilder::new(app, "Kryton")
        .item(&PredefinedMenuItem::about(app, None, None)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("settings", "Settings…")
                .accelerator("CmdOrCtrl+,")
                .build(app)?,
        )
        .separator()
        .item(&PredefinedMenuItem::quit(app, None)?)
        .build()?;

    // File submenu
    let file_submenu = SubmenuBuilder::new(app, "File")
        .item(
            &MenuItemBuilder::with_id("new_capture", "Quick Capture…")
                .accelerator("CmdOrCtrl+Shift+N")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("add_server", "Add Server…")
                .accelerator("CmdOrCtrl+Shift+A")
                .build(app)?,
        )
        .build()?;

    // Edit submenu
    let edit_submenu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    // View submenu
    let view_submenu = SubmenuBuilder::new(app, "View")
        .item(
            &MenuItemBuilder::with_id("reload", "Reload")
                .accelerator("CmdOrCtrl+R")
                .build(app)?,
        )
        .build()?;

    // Servers submenu — one item per account, Cmd+1..9
    let mut servers_builder = SubmenuBuilder::new(app, "Servers");
    for (i, acct) in accounts.iter().enumerate() {
        let id = format!("switch:{}", acct.id);
        let label = acct.label.clone();
        let mut item_builder = MenuItemBuilder::with_id(id, label);
        if i < 9 {
            item_builder = item_builder.accelerator(format!("CmdOrCtrl+{}", i + 1));
        }
        servers_builder = servers_builder.item(&item_builder.build(app)?);
    }
    let servers_submenu = servers_builder.build()?;

    MenuBuilder::new(app)
        .item(&app_submenu)
        .item(&file_submenu)
        .item(&edit_submenu)
        .item(&view_submenu)
        .item(&servers_submenu)
        .build()
}
