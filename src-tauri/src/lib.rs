use tauri::{WebviewUrl, WebviewWindowBuilder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            WebviewWindowBuilder::new(
                app,
                "login",
                WebviewUrl::App("src/login/index.html".into()),
            )
            .title("Kryton")
            .inner_size(420.0, 540.0)
            .resizable(false)
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Kryton");
}
