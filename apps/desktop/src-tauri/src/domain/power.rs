/// Perintah daya yang bisa dijalankan dari pulau.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerAction {
    Shutdown,
    Restart,
    Sleep,
    Lock,
}

impl PowerAction {
    /// Aksi yang mengakhiri sesi pengguna perlu konfirmasi; mengunci layar tidak.
    pub fn needs_confirm(self) -> bool {
        !matches!(self, PowerAction::Lock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanya_kunci_layar_tanpa_konfirmasi() {
        assert!(PowerAction::Shutdown.needs_confirm());
        assert!(PowerAction::Restart.needs_confirm());
        assert!(PowerAction::Sleep.needs_confirm());
        assert!(!PowerAction::Lock.needs_confirm());
    }
}
