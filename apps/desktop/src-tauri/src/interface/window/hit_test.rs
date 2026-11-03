//! Klik tembus: jendela mengabaikan kursor kecuali kursor berada di dalam hitbox pulau.
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Deserialize;
use specta::Type;
use tauri::{Emitter, WebviewWindow};

use super::island_window::{center_top, monitor_at};

/// Kursor dipantau cepat hanya bila berada sedekat ini dari tepi atas monitor (piksel logis).
const NEAR_TOP: f64 = 160.0;
const FAST: Duration = Duration::from_millis(33);
const SLOW: Duration = Duration::from_millis(150);

#[derive(Clone, Copy, Debug, Default, Deserialize, Type)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

#[derive(Default)]
pub struct Hitbox(Mutex<Rect>);

impl Hitbox {
    pub fn set(&self, rect: Rect) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = rect;
    }

    fn get(&self) -> Rect {
        *self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub fn spawn(window: WebviewWindow, hitbox: Arc<Hitbox>) {
    tauri::async_runtime::spawn(async move {
        let mut ignoring = true;
        let mut origin = None;
        let mut last_cursor = (0, 0);
        let _ = window.set_ignore_cursor_events(true);
        loop {
            let delay = tick(
                &window,
                &hitbox,
                &mut ignoring,
                &mut origin,
                &mut last_cursor,
            );
            tokio::time::sleep(delay).await;
        }
    });
}

/// Satu putaran: pindahkan jendela bila monitor kursor berubah, lalu atur klik tembus.
fn tick(
    window: &WebviewWindow,
    hitbox: &Hitbox,
    ignoring: &mut bool,
    origin: &mut Option<(i32, i32)>,
    last_cursor: &mut (i32, i32),
) -> Duration {
    let Ok(cursor) = window.cursor_position() else {
        return SLOW;
    };
    let Some(monitor) = monitor_at(window, cursor) else {
        return SLOW;
    };
    let top_left = (monitor.position().x, monitor.position().y);
    if *origin != Some(top_left) {
        *origin = Some(top_left);
        if let Err(e) = center_top(window, &monitor) {
            tracing::warn!("gagal memosisikan pulau: {e}");
        }
    }

    let scale = window.scale_factor().unwrap_or(1.0);
    // Mata Lumi mengikuti kursor di mana pun; jendela klik-tembus tidak menerima gerak mouse, jadi dikirim dari sini.
    let here = (cursor.x as i32, cursor.y as i32);
    if (here.0 - last_cursor.0).abs() + (here.1 - last_cursor.1).abs() >= 3
        && let Ok(pos) = window.outer_position()
    {
        *last_cursor = here;
        let _ = window.emit(
            "cursor",
            (
                (cursor.x - pos.x as f64) / scale,
                (cursor.y - pos.y as f64) / scale,
            ),
        );
    }
    let near = cursor.y - (top_left.1 as f64) < NEAR_TOP * scale;
    let inside = near
        && window.outer_position().is_ok_and(|pos| {
            let x = (cursor.x - pos.x as f64) / scale;
            let y = (cursor.y - pos.y as f64) / scale;
            hitbox.get().contains(x, y)
        });

    if inside == *ignoring {
        *ignoring = !inside;
        let _ = window.set_ignore_cursor_events(*ignoring);
    }
    if near || !*ignoring { FAST } else { SLOW }
}

#[cfg(test)]
mod tests {
    use super::Rect;

    #[test]
    fn rect_contains_menghitung_tepi_dan_luar() {
        let r = Rect {
            x: 10.0,
            y: 0.0,
            width: 100.0,
            height: 30.0,
        };
        assert!(r.contains(10.0, 0.0));
        assert!(r.contains(110.0, 30.0));
        assert!(!r.contains(9.9, 10.0));
        assert!(!r.contains(50.0, 30.1));
        assert!(!Rect::default().contains(5.0, 5.0));
    }
}
