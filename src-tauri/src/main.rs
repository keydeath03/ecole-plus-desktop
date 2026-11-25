// No console window next to the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicUsize, Ordering};

use tauri::webview::DownloadEvent;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

/// The École+ site. Change it here and in src/index.html when the domain changes.
const SITE_HOST: &str = "www.gerermonecole.com";

/// Runs in every page of the site: links meant for a new tab open in the app (a document of the
/// site, such as an invoice) or in the browser (anything else: WhatsApp, e-mail, phone).
const LINKS_SCRIPT: &str = r#"
(() => {
  const open = (href) => {
    const url = new URL(href, location.href);
    if (url.origin === location.origin && window.__TAURI_INTERNALS__) {
      window.__TAURI_INTERNALS__.invoke('open_document', { url: url.href }).catch(() => location.assign(url.href));
    } else {
      location.assign(url.href);
    }
  };
  document.addEventListener('click', (e) => {
    const a = e.target instanceof Element ? e.target.closest('a[href]') : null;
    if (!a || a.hasAttribute('download') || e.defaultPrevented) return;
    const url = new URL(a.href, location.href);
    const outside = url.origin !== location.origin;
    if (a.target === '_blank' || (outside && /^https?:$/.test(url.protocol))) {
      e.preventDefault();
      open(url.href);
    }
  }, true);
  window.open = (href) => { if (href) open(String(href)); return null; };
})();
"#;

/// Our own pages (the start screen) and the site stay in the app; every other address opens in the browser.
fn stays_inside(url: &Url) -> bool {
    match url.scheme() {
        "tauri" | "about" | "data" | "blob" => true,
        "http" | "https" => matches!(url.host_str(), Some(h) if h == SITE_HOST || h == "tauri.localhost"),
        _ => false,
    }
}

fn open_outside(app: &AppHandle, url: &Url) {
    let _ = app.opener().open_url(url.as_str(), None::<&str>);
}

/// A file of the site (an invoice, a report card) lands in Downloads and opens.
fn on_download(app: AppHandle) -> impl Fn(tauri::Webview, DownloadEvent<'_>) -> bool + Send + Sync + 'static {
    move |_webview, event| {
        match event {
            DownloadEvent::Requested { url, destination } => {
                let name = destination
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .filter(|n| !n.is_empty())
                    .or_else(|| url.path_segments().and_then(|s| s.last().map(str::to_owned)).filter(|n| !n.is_empty()))
                    .unwrap_or_else(|| "document.pdf".into());
                if let Ok(dir) = app.path().download_dir() {
                    *destination = dir.join(name);
                }
            }
            DownloadEvent::Finished { path: Some(path), success: true, .. } => {
                let _ = app.opener().open_path(path.to_string_lossy(), None::<&str>);
            }
            _ => {}
        }
        true
    }
}

static DOCUMENTS: AtomicUsize = AtomicUsize::new(0);

/// A document of the site in its own window, still signed in.
#[tauri::command]
fn open_document(app: AppHandle, url: String) -> Result<(), String> {
    let url: Url = url.parse().map_err(|_| "adresse invalide".to_string())?;
    if !stays_inside(&url) {
        open_outside(&app, &url);
        return Ok(());
    }
    let label = format!("doc-{}", DOCUMENTS.fetch_add(1, Ordering::Relaxed));
    let handle = app.clone();
    WebviewWindowBuilder::new(&app, label, WebviewUrl::External(url))
        .title("École+")
        .inner_size(900.0, 1000.0)
        .min_inner_size(480.0, 400.0)
        .initialization_script(LINKS_SCRIPT)
        .on_navigation(move |u| stays_inside(u) || {
            open_outside(&handle, u);
            false
        })
        .on_download(on_download(app.clone()))
        .build()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        // A second launch brings the open window forward instead of opening another app.
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![open_document])
        .setup(|app| {
            let handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("École+")
                .inner_size(1280.0, 820.0)
                .min_inner_size(900.0, 600.0)
                .center()
                .initialization_script(LINKS_SCRIPT)
                .on_navigation(move |u| stays_inside(u) || {
                    open_outside(&handle, u);
                    false
                })
                .on_download(on_download(app.handle().clone()))
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("École+ n'a pas pu démarrer");
}
