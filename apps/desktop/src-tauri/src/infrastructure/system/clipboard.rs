use crate::domain::ports::ClipboardAccess;

/// Clipboard sistem (hanya teks). Gagal membaca (mis. isinya gambar) dianggap tidak ada teks.
pub struct SystemClipboard;

impl ClipboardAccess for SystemClipboard {
    fn read_text(&self) -> Option<String> {
        arboard::Clipboard::new().ok()?.get_text().ok()
    }

    fn write_text(&self, text: &str) -> bool {
        arboard::Clipboard::new()
            .and_then(|mut c| c.set_text(text))
            .is_ok()
    }

    fn write_files(&self, paths: &[String]) -> bool {
        arboard::Clipboard::new()
            .and_then(|mut c| c.set().file_list(paths))
            .is_ok()
    }
}
