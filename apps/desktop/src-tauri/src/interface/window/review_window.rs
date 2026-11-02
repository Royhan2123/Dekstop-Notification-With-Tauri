use tauri::{AppHandle, Manager};

/// Jendela rincian review dibuat tersembunyi di konfigurasi; menutupnya hanya menyembunyikan.
pub const LABEL: &str = "review";

pub fn show(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.hide();
    }
}
