use std::sync::Arc;

use tauri::State;

use crate::application::net_service::NetService;
use crate::interface::dto::NetRateDto;

/// Kecepatan jaringan saat ini (dipanggil panel tiap detik selama terbuka).
#[tauri::command]
#[specta::specta]
pub fn net_rate(net: State<Arc<NetService>>) -> Result<NetRateDto, String> {
    net.rate()
        .map(|r| NetRateDto {
            down_bps: r.down,
            up_bps: r.up,
        })
        .ok_or_else(|| "pembacaan jaringan tidak didukung di sistem ini".to_string())
}
