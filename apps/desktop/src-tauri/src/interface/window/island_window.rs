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
