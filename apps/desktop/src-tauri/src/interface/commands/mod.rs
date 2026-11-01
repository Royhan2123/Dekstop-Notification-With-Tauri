/// Command contoh untuk memverifikasi pembuatan `bindings.ts`; hapus saat command asli ada.
#[tauri::command]
#[specta::specta]
pub fn ping() -> String {
    "pong".into()
}
