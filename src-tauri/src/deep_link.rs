use crate::accounts::AccountsFile;
use crate::ipc::AppState;
use crate::window_mgr;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use url::Url;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub struct Parsed {
    pub host: String,
    pub path: String,
}

pub fn parse(input: &str) -> Option<Parsed> {
    let u = Url::parse(input).ok()?;
    if u.scheme() != "kryton" {
        return None;
    }
    let host = u.host_str()?.to_string();
    let path = if u.path().is_empty() {
        "/".to_string()
    } else {
        u.path().to_string()
    };
    Some(Parsed { host, path })
}

pub fn find_account_id(file: &AccountsFile, host: &str) -> Option<Uuid> {
    file.accounts.iter().find_map(|a| {
        let server = Url::parse(&a.server_url).ok()?;
        if server.host_str() == Some(host) {
            Some(a.id)
        } else {
            None
        }
    })
}

pub fn handle_argv<R: Runtime>(app: &AppHandle<R>, argv: &[String]) {
    for arg in argv {
        if let Some(parsed) = parse(arg) {
            handle(app, parsed);
            return;
        }
    }
}

pub fn handle<R: Runtime>(app: &AppHandle<R>, parsed: Parsed) {
    let acct_id = {
        let st: tauri::State<'_, AppState> = app.state();
        let f = st.accounts.lock().unwrap();
        find_account_id(&f, &parsed.host)
    };
    let h = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(id) = acct_id {
            let st: tauri::State<'_, AppState> = h.state();
            let _ = crate::ipc::switch_to(h.clone(), st, id).await;
            let target_url = {
                let st: tauri::State<'_, AppState> = h.state();
                let f = st.accounts.lock().unwrap();
                f.accounts
                    .iter()
                    .find(|a| a.id == id)
                    .cloned()
                    .map(|a| format!("{}{}", a.server_url.trim_end_matches('/'), parsed.path))
            };
            if let Some(u) = target_url {
                if let Some(w) = h.get_webview_window(&window_mgr::label_for(&id)) {
                    let js = format!(
                        "window.location.href = {}",
                        serde_json::to_string(&u).unwrap()
                    );
                    let _ = w.eval(&js);
                }
            }
        } else {
            let _ = h.emit("deep-link:unknown", &parsed.host);
            let _ = crate::open_add_server_window(&h);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{add, AccountsFile};

    #[test]
    fn parses_kryton_url() {
        let p = parse("kryton://k.example.com/notes/foo.md").unwrap();
        assert_eq!(p.host, "k.example.com");
        assert_eq!(p.path, "/notes/foo.md");
    }

    #[test]
    fn rejects_other_schemes() {
        assert!(parse("https://k.example.com/").is_none());
    }

    #[test]
    fn matches_account_by_host() {
        let mut f = AccountsFile::default();
        let a = add(&mut f, "A".into(), "https://k.example.com:8080".into(), "u".into());
        assert_eq!(find_account_id(&f, "k.example.com"), Some(a.id));
        assert_eq!(find_account_id(&f, "other.example"), None);
    }
}
