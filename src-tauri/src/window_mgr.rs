use crate::accounts::Account;
use crate::error::{AppError, AppResult};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

pub fn label_for(id: &Uuid) -> String {
    format!("server-{id}")
}

pub fn data_dir<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<PathBuf> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Invalid(e.to_string()))?;
    let dir = base.join("webview-data").join(id.to_string());
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn open_or_focus<R: Runtime>(app: &AppHandle<R>, acct: &Account) -> AppResult<()> {
    let label = label_for(&acct.id);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    let dir = data_dir(app, &acct.id)?;
    let url: url::Url = acct.server_url.parse()?;
    let creds_json = match crate::auth::keychain::read(app, &acct.id) {
        Ok(password) => serde_json::to_string(&serde_json::json!({
            "email": acct.username,
            "password": password,
        }))
        .unwrap_or_else(|_| "null".into()),
        Err(_) => "null".into(),
    };
    let init_script = format!(
        r#"
        window.__kryton_desktop = {{
          accountId: "{account_id}",
          notify: (title, body) => {{
            if (window.__TAURI__ && window.__TAURI__.core) {{
              return window.__TAURI__.core.invoke('notify_from_web', {{ title, body }});
            }}
          }}
        }};
        (() => {{
          function showBanner(text) {{
            const ensure = () => {{
              if (!document.body) {{
                document.addEventListener('DOMContentLoaded', () => ensure(), {{ once: true }});
                return;
              }}
              if (document.getElementById('kryton-desktop-banner')) return;
              const b = document.createElement('div');
              b.id = 'kryton-desktop-banner';
              b.setAttribute('style', [
                'position:fixed','top:0','left:0','right:0',
                'z-index:2147483647',
                'padding:10px 14px',
                'background:oklch(0.7 0.2 25 / 0.92)',
                'color:#fff',
                'font-family:"JetBrains Mono",ui-monospace,monospace',
                'font-size:12px',
                'letter-spacing:0.02em',
                'box-shadow:0 4px 12px rgba(0,0,0,0.4)',
                'display:flex','align-items:center','gap:10px','justify-content:center',
              ].join(';'));
              b.innerHTML =
                '<span>⚠ ' + text + '</span>' +
                '<button id="kryton-desktop-banner-dismiss" style="margin-left:auto;background:transparent;color:#fff;border:1px solid rgba(255,255,255,0.5);padding:3px 8px;border-radius:4px;cursor:pointer;font-family:inherit;font-size:11px;">Dismiss</button>';
              document.body.appendChild(b);
              const dismiss = document.getElementById('kryton-desktop-banner-dismiss');
              if (dismiss) dismiss.addEventListener('click', () => b.remove());
            }};
            ensure();
          }}
          window.__kryton_desktop.showBanner = showBanner;
          (async () => {{
            if (window.__kryton_relogin_attempted) return;
            window.__kryton_relogin_attempted = true;
            const creds = {creds_json};
            if (!creds) return;
            try {{
              const sess = await fetch('/api/auth/get-session', {{ credentials: 'include' }});
              if (sess.ok) {{
                const data = await sess.json();
                if (data && data.user) return;
              }}
            }} catch (e) {{ console.warn('kryton-desktop: get-session failed', e); }}
            try {{
              const r = await fetch('/api/auth/sign-in/email', {{
                method: 'POST',
                headers: {{ 'content-type': 'application/json' }},
                credentials: 'include',
                body: JSON.stringify(creds),
              }});
              if (r.ok) {{
                window.location.replace('/');
              }} else if (r.status === 401 || r.status === 403) {{
                showBanner('Saved credentials are invalid. Remove this server in Settings and add it again.');
              }} else {{
                showBanner('Auto-login failed (HTTP ' + r.status + '). Sign in manually below.');
              }}
            }} catch (e) {{
              showBanner('Auto-login failed: ' + (e && e.message ? e.message : 'network error') + '. Sign in manually below.');
            }}
          }})();
        }})();
    "#,
        account_id = acct.id,
        creds_json = creds_json,
    );
    WebviewWindowBuilder::new(app, &label, WebviewUrl::External(url))
        .title(format!("Kryton — {}", acct.label))
        .inner_size(1200.0, 800.0)
        .data_directory(dir)
        .initialization_script(&init_script)
        .build()
        .map_err(|e| AppError::Invalid(e.to_string()))?;
    Ok(())
}

pub fn hide_all_except<R: Runtime>(app: &AppHandle<R>, id: &Uuid) {
    let keep = label_for(id);
    for (label, win) in app.webview_windows() {
        if label.starts_with("server-") && label != keep {
            let _ = win.hide();
        }
    }
}

pub fn close<R: Runtime>(app: &AppHandle<R>, id: &Uuid) -> AppResult<()> {
    let label = label_for(id);
    if let Some(win) = app.get_webview_window(&label) {
        win.close().map_err(|e| AppError::Invalid(e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_for_is_stable_per_id() {
        assert_eq!(
            label_for(&Uuid::nil()),
            "server-00000000-0000-0000-0000-000000000000"
        );
    }
}
