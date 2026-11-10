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

/// macOS: pulau tampil di semua Space dan di atas aplikasi layar penuh.
/// `set_visible_on_all_workspaces` saja tidak cukup untuk Space layar penuh; jendela perlu perilaku
/// `fullScreenAuxiliary` dan level di atas menu bar.
#[cfg(target_os = "macos")]
pub fn float_over_fullscreen(window: &WebviewWindow) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    // NSWindowCollectionBehavior: canJoinAllSpaces (1<<0) | stationary (1<<4) | fullScreenAuxiliary (1<<8)
    const BEHAVIOR: usize = 1 | (1 << 4) | (1 << 8);
    // NSStatusWindowLevel (25): di atas menu bar (24) dan jendela layar penuh.
    const LEVEL: isize = 25;

    let Ok(ptr) = window.ns_window() else {
        tracing::warn!("NSWindow pulau tidak tersedia");
        return;
    };
    let ns_window = ptr.cast::<AnyObject>();
    // SAFETY: `ptr` adalah NSWindow hidup milik jendela Tauri; kedua selector ada di NSWindow dan
    // dipanggil dari thread utama (setup).
    unsafe {
        let _: () = msg_send![ns_window, setCollectionBehavior: BEHAVIOR];
        let _: () = msg_send![ns_window, setLevel: LEVEL];
    }
}
