use tauri::{Monitor, PhysicalPosition, WebviewWindow};

/// Monitor yang berisi titik `p` (piksel fisik); jatuh ke monitor utama bila tidak ada.
pub fn monitor_at(window: &WebviewWindow, p: PhysicalPosition<f64>) -> Option<Monitor> {
    let found = window.available_monitors().ok()?.into_iter().find(|m| {
        let (pos, size) = (m.position(), m.size());
        p.x >= pos.x as f64
            && p.x < pos.x as f64 + size.width as f64
            && p.y >= pos.y as f64
            && p.y < pos.y as f64 + size.height as f64
    });
    found.or_else(|| window.primary_monitor().ok().flatten())
}

/// Letakkan jendela di tengah atas `monitor`.
pub fn center_top(window: &WebviewWindow, monitor: &Monitor) -> tauri::Result<()> {
    let win_w = window.outer_size()?.width as i32;
    let x = monitor.position().x + (monitor.size().width as i32 - win_w) / 2;
    window.set_position(PhysicalPosition::new(x, monitor.position().y))
}

/// macOS: ubah jendela pulau jadi NSPanel agar tampil di semua Space dan di atas aplikasi layar
/// penuh. `set_visible_on_all_workspaces` dan level jendela saja tidak cukup untuk Space layar penuh;
/// macOS hanya mengizinkan NSPanel non-aktivasi dengan perilaku `fullScreenAuxiliary` di sana.
/// Hanya boleh dipanggil sekali per jendela.
// API `cocoa` milik tauri-nspanel ditandai usang, tapi itu satu-satunya jalur yang didokumentasikannya.
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn float_over_fullscreen(window: &WebviewWindow) {
    use tauri_nspanel::WebviewWindowExt;
    use tauri_nspanel::cocoa::appkit::NSWindowCollectionBehavior as Behavior;

    // NSStatusWindowLevel (25): di atas menu bar (24) dan jendela layar penuh.
    const LEVEL: i32 = 25;
    // NSWindowStyleMaskNonactivatingPanel: mengklik pulau tidak merebut fokus dari aplikasi lain.
    const NON_ACTIVATING: i32 = 1 << 7;

    let panel = match window.to_panel() {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("gagal mengubah pulau jadi NSPanel: {e}");
            return;
        }
    };
    panel.set_level(LEVEL);
    panel.set_style_mask(NON_ACTIVATING);
    panel.set_collection_behaviour(
        Behavior::NSWindowCollectionBehaviorFullScreenAuxiliary
            | Behavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | Behavior::NSWindowCollectionBehaviorStationary,
    );
}
