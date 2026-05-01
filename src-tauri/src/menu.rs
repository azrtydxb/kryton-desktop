use tauri::{
    menu::{
        Menu, MenuItem, PredefinedMenuItem, Submenu,
    },
    AppHandle, Emitter, Manager, Runtime,
};

/// Build and return the application menu for Tauri 2.x.
pub fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    // -------------------------------------------------------------------------
    // Kryton (app) menu
    // -------------------------------------------------------------------------
    let about = MenuItem::with_id(app, "about", "About Kryton", true, None::<&str>)?;
    let preferences = MenuItem::with_id(app, "preferences", "Preferences…", true, Some("CmdOrCtrl+,"))?;
    let services = PredefinedMenuItem::services(app, Some("Services"))?;
    let hide = PredefinedMenuItem::hide(app, Some("Hide Kryton"))?;
    let hide_others = PredefinedMenuItem::hide_others(app, Some("Hide Others"))?;
    let show_all = PredefinedMenuItem::show_all(app, None)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let separator3 = PredefinedMenuItem::separator(app)?;
    let quit = PredefinedMenuItem::quit(app, Some("Quit Kryton"))?;

    let app_menu = Submenu::with_items(
        app,
        "Kryton",
        true,
        &[
            &about,
            &separator,
            &preferences,
            &separator2,
            &services,
            &separator3,
            &hide,
            &hide_others,
            &show_all,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    // -------------------------------------------------------------------------
    // File menu
    // -------------------------------------------------------------------------
    let new_note = MenuItem::with_id(app, "new-note", "New Note", true, Some("CmdOrCtrl+N"))?;
    let new_window = MenuItem::with_id(app, "new-window", "New Window", true, Some("CmdOrCtrl+Shift+N"))?;
    let open = MenuItem::with_id(app, "open", "Open…", true, Some("CmdOrCtrl+O"))?;
    let switch_account = MenuItem::with_id(app, "switch-account", "Switch Account", true, None::<&str>)?;
    let close_window = PredefinedMenuItem::close_window(app, Some("Close Window"))?;

    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &new_note,
            &new_window,
            &PredefinedMenuItem::separator(app)?,
            &open,
            &switch_account,
            &PredefinedMenuItem::separator(app)?,
            &close_window,
        ],
    )?;

    // -------------------------------------------------------------------------
    // Edit menu
    // -------------------------------------------------------------------------
    let undo = PredefinedMenuItem::undo(app, Some("Undo"))?;
    let redo = PredefinedMenuItem::redo(app, Some("Redo"))?;
    let cut = PredefinedMenuItem::cut(app, Some("Cut"))?;
    let copy = PredefinedMenuItem::copy(app, Some("Copy"))?;
    let paste = PredefinedMenuItem::paste(app, Some("Paste"))?;
    let select_all = PredefinedMenuItem::select_all(app, Some("Select All"))?;
    let find = MenuItem::with_id(app, "find", "Find…", true, Some("CmdOrCtrl+F"))?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &undo,
            &redo,
            &PredefinedMenuItem::separator(app)?,
            &cut,
            &copy,
            &paste,
            &PredefinedMenuItem::separator(app)?,
            &select_all,
            &PredefinedMenuItem::separator(app)?,
            &find,
        ],
    )?;

    // -------------------------------------------------------------------------
    // View menu
    // -------------------------------------------------------------------------
    let toggle_sidebar = MenuItem::with_id(app, "toggle-sidebar", "Toggle Sidebar", true, Some("CmdOrCtrl+\\"))?;
    let toggle_graph = MenuItem::with_id(app, "toggle-graph", "Toggle Graph", true, None::<&str>)?;
    let toggle_edit_preview = MenuItem::with_id(app, "toggle-edit-preview", "Toggle Edit / Preview", true, Some("CmdOrCtrl+Shift+P"))?;
    let show_daily = MenuItem::with_id(app, "show-daily", "Show Daily Note", true, Some("CmdOrCtrl+Shift+D"))?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &toggle_sidebar,
            &toggle_graph,
            &PredefinedMenuItem::separator(app)?,
            &toggle_edit_preview,
            &PredefinedMenuItem::separator(app)?,
            &show_daily,
        ],
    )?;

    // -------------------------------------------------------------------------
    // Window menu
    // -------------------------------------------------------------------------
    let minimize = PredefinedMenuItem::minimize(app, Some("Minimize"))?;
    let zoom = PredefinedMenuItem::maximize(app, Some("Zoom"))?;
    let bring_to_front = MenuItem::with_id(app, "bring-to-front", "Bring All to Front", true, None::<&str>)?;

    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[
            &minimize,
            &zoom,
            &PredefinedMenuItem::separator(app)?,
            &bring_to_front,
        ],
    )?;

    // -------------------------------------------------------------------------
    // Help menu
    // -------------------------------------------------------------------------
    let docs = MenuItem::with_id(app, "docs", "Kryton Documentation", true, None::<&str>)?;
    let show_logs = MenuItem::with_id(app, "show-logs", "Show Logs", true, None::<&str>)?;
    let report_issue = MenuItem::with_id(app, "report-issue", "Report an Issue", true, None::<&str>)?;

    let help_menu = Submenu::with_items(
        app,
        "Help",
        true,
        &[
            &docs,
            &PredefinedMenuItem::separator(app)?,
            &show_logs,
            &report_issue,
        ],
    )?;

    // -------------------------------------------------------------------------
    // Assemble the full menu bar
    // -------------------------------------------------------------------------
    Menu::with_items(
        app,
        &[
            &app_menu,
            &file_menu,
            &edit_menu,
            &view_menu,
            &window_menu,
            &help_menu,
        ],
    )
}

/// Emit a `menu-action` event to the focused window (falls back to all windows).
pub fn emit_menu_action<R: Runtime>(app: &AppHandle<R>, action_id: &str) {
    // Try to emit to the focused window first; fall back to broadcasting.
    let emitted = app.webview_windows().into_iter().any(|(_, win)| {
        if win.is_focused().unwrap_or(false) {
            win.emit("menu-action", action_id).is_ok()
        } else {
            false
        }
    });

    if !emitted {
        // No focused window found — broadcast to all.
        let _ = app.emit("menu-action", action_id);
    }
}
