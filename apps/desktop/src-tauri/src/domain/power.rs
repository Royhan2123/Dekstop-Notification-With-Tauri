/// Perintah daya yang bisa dijalankan dari pulau.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerAction {
    Shutdown,
    Restart,
    /// Layar mati sampai ada klik atau tombol; komputer tetap menyala.
    Sleep,
    Lock,
}

impl PowerAction {
    /// Hanya aksi yang menutup sesi pengguna perlu konfirmasi; kunci dan layar mati tidak.
    pub fn needs_confirm(self) -> bool {
        matches!(self, PowerAction::Shutdown | PowerAction::Restart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanya_matikan_dan_mulai_ulang_perlu_konfirmasi() {
        assert!(PowerAction::Shutdown.needs_confirm());
        assert!(PowerAction::Restart.needs_confirm());
        assert!(!PowerAction::Sleep.needs_confirm());
        assert!(!PowerAction::Lock.needs_confirm());
    }
}
