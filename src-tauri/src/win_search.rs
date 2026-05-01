#![cfg(target_os = "windows")]

use std::fs;
use std::path::PathBuf;

/// Returns the directory where `.url` shortcut files are written for a given account.
/// Path: %LocalAppData%\Microsoft\Windows\Start Menu\Programs\Kryton\<account_id>\
fn shortcuts_dir(account_id: &str) -> Result<PathBuf, String> {
    let local = std::env::var("LOCALAPPDATA").map_err(|e| e.to_string())?;
    let dir = PathBuf::from(local)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Kryton")
        .join(account_id);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

/// Sanitize a note title to a safe filename component.
fn safe_filename(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect()
}

/// Create or update a `.url` shortcut so Windows Search can find the note.
pub fn upsert_note(account_id: &str, note_path: &str, title: &str) -> Result<(), String> {
    let dir = shortcuts_dir(account_id)?;
    let filename = safe_filename(title);
    let path = dir.join(format!("{}.url", filename));
    let content = format!(
        "[InternetShortcut]\nURL=kryton://note/{}?account={}\n",
        urlencoding::encode(note_path),
        account_id
    );
    fs::write(&path, content).map_err(|e| e.to_string())
}

/// Remove the `.url` shortcut for a note.
pub fn remove_note(account_id: &str, title: &str) -> Result<(), String> {
    let dir = shortcuts_dir(account_id)?;
    let filename = safe_filename(title);
    let path = dir.join(format!("{}.url", filename));
    // Ignore errors from missing files.
    let _ = fs::remove_file(path);
    Ok(())
}
