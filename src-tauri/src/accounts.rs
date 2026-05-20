use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub id: Uuid,
    pub label: String,
    pub server_url: String,
    pub username: String,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default = "default_shortcut")]
    pub shortcut_quick_capture: String,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default = "default_channel")]
    pub auto_update_channel: String,
}

fn default_shortcut() -> String {
    "CmdOrCtrl+Shift+N".into()
}
fn default_channel() -> String {
    "stable".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountsFile {
    pub accounts: Vec<Account>,
    pub default_active: Option<Uuid>,
    #[serde(default)]
    pub settings: Settings,
}

pub fn load(path: &Path) -> AppResult<AccountsFile> {
    if !path.exists() {
        return Ok(AccountsFile::default());
    }
    let s = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&s)?)
}

pub fn save(path: &Path, file: &AccountsFile) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(file)?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn add(
    file: &mut AccountsFile,
    label: String,
    server_url: String,
    username: String,
) -> Account {
    let acct = Account {
        id: Uuid::new_v4(),
        label,
        server_url,
        username,
        last_active: Utc::now(),
    };
    file.accounts.push(acct.clone());
    if file.default_active.is_none() {
        file.default_active = Some(acct.id);
    }
    acct
}

pub fn remove(file: &mut AccountsFile, id: Uuid) -> AppResult<Account> {
    let idx = file
        .accounts
        .iter()
        .position(|a| a.id == id)
        .ok_or_else(|| AppError::AccountNotFound(id.to_string()))?;
    let acct = file.accounts.remove(idx);
    if file.default_active == Some(id) {
        file.default_active = file.accounts.first().map(|a| a.id);
    }
    Ok(acct)
}

pub fn touch(file: &mut AccountsFile, id: Uuid) {
    if let Some(a) = file.accounts.iter_mut().find(|a| a.id == id) {
        a.last_active = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip_persistence() {
        let dir = tempdir().unwrap();
        let p: PathBuf = dir.path().join("accounts.json");
        let mut f = AccountsFile::default();
        add(&mut f, "Home".into(), "https://k.example".into(), "pascal".into());
        save(&p, &f).unwrap();
        let f2 = load(&p).unwrap();
        assert_eq!(f.accounts, f2.accounts);
        assert_eq!(f.default_active, f2.default_active);
    }

    #[test]
    fn add_sets_default_only_when_empty() {
        let mut f = AccountsFile::default();
        let a = add(&mut f, "A".into(), "https://a".into(), "u".into());
        assert_eq!(f.default_active, Some(a.id));
        let b = add(&mut f, "B".into(), "https://b".into(), "u".into());
        // Still A
        assert_eq!(f.default_active, Some(a.id));
        let _ = b;
    }

    #[test]
    fn remove_clears_or_reassigns_default() {
        let mut f = AccountsFile::default();
        let a = add(&mut f, "A".into(), "https://a".into(), "u".into());
        let b = add(&mut f, "B".into(), "https://b".into(), "u".into());
        remove(&mut f, a.id).unwrap();
        assert_eq!(f.default_active, Some(b.id));
        remove(&mut f, b.id).unwrap();
        assert_eq!(f.default_active, None);
    }

    #[test]
    fn missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let f = load(&dir.path().join("nope.json")).unwrap();
        assert!(f.accounts.is_empty());
    }
}
