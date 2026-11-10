//! macOS: sembunyikan HUD volume/kecerahan bawaan sistem supaya hanya HUD pulau yang tampil.
//!
//! HUD itu digambar proses `OSDUIHelper` milik user yang dijalankan macOS saat dibutuhkan. Selama
//! setelan menyala, Isla menjalankannya lebih dulu lalu membekukannya (SIGSTOP): permintaan HUD tidak
//! digambar, volume dan kecerahan tetap berubah normal. Berlaku untuk semua sumber (Touch Bar,
//! keyboard, headset), tanpa izin Accessibility.
//!
//! Keamanan: tidak ada perubahan permanen pada sistem. Saat setelan dimatikan prosesnya dihentikan
//! (macOS menjalankannya lagi bila perlu). Proses penjaga kecil menunggu Isla; bila Isla mati atau
//! crash, penjaga menghentikan proses HUD yang beku supaya HUD sistem kembali normal.
use std::ffi::c_void;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Nama proses HUD sistem; layanan launchd-nya `com.apple.OSDUIHelper`.
const HELPER: &str = "OSDUIHelper";
/// Jeda minimal antar percobaan menjalankan HUD sistem (hindari memberondong launchctl bila gagal).
const KICKSTART_EVERY: Duration = Duration::from_secs(5);

unsafe extern "C" {
    fn proc_listallpids(buffer: *mut c_void, buffer_size: i32) -> i32;
    fn proc_name(pid: i32, buffer: *mut c_void, buffer_size: u32) -> i32;
}

/// PID semua proses bernama persis `name` (libproc, tanpa memanggil program lain).
pub(crate) fn pids_named(name: &str) -> Vec<i32> {
    // SAFETY: panggilan pertama dengan buffer kosong hanya menanyakan ukuran yang dibutuhkan.
    let bytes = unsafe { proc_listallpids(std::ptr::null_mut(), 0) };
    if bytes <= 0 {
        return Vec::new();
    }
    // Ruang cadangan: proses bisa bertambah di antara dua panggilan.
    let capacity = bytes as usize / std::mem::size_of::<i32>() + 64;
    let mut pids = vec![0i32; capacity];
    // SAFETY: `pids` hidup selama panggilan dan ukurannya (byte) diberikan apa adanya.
    let filled = unsafe {
        proc_listallpids(
            pids.as_mut_ptr().cast(),
            (capacity * std::mem::size_of::<i32>()) as i32,
        )
    };
    if filled <= 0 {
        return Vec::new();
    }
    pids.truncate(filled as usize / std::mem::size_of::<i32>());
    let mut found = Vec::new();
    let mut buf = [0u8; 256];
    for pid in pids.into_iter().filter(|p| *p > 0) {
        // SAFETY: `buf` hidup selama panggilan; ukurannya diberikan; hasil berupa string C.
        let n = unsafe { proc_name(pid, buf.as_mut_ptr().cast(), buf.len() as u32) };
        if n > 0 && &buf[..n as usize] == name.as_bytes() {
            found.push(pid);
        }
    }
    found
}

fn signal(pids: &[i32], sig: i32) {
    for &pid in pids {
        // SAFETY: kill() hanya mengirim sinyal; proses milik user yang sama.
        unsafe { libc::kill(pid, sig) };
    }
}

/// Skrip penjaga: menunggu proses Isla (`parent`) berakhir, lalu menghentikan HUD sistem yang beku.
pub(crate) fn guardian_script(parent: u32) -> String {
    format!("while kill -0 {parent} 2>/dev/null; do sleep 1; done; pkill -KILL -x {HELPER}")
}

/// Proses penjaga lepas dari Isla (grup proses sendiri) supaya tetap hidup bila Isla crash.
fn spawn_guardian() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        use std::os::unix::process::CommandExt;
        let _ = Command::new("sh")
            .args(["-c", &guardian_script(std::process::id())])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .spawn();
    });
}

/// Menahan HUD sistem selama setelan menyala. Dipanggil berkala (±2x per detik) dari satu thread.
#[derive(Default)]
pub struct SystemHudSuppressor {
    active: bool,
    last_kickstart: Option<Instant>,
}

impl SystemHudSuppressor {
    pub fn tick(&mut self, enabled: bool) {
        if enabled {
            self.suppress();
        } else if self.active {
            self.restore();
        }
    }

    fn suppress(&mut self) {
        if !self.active {
            spawn_guardian(); // sebelum membekukan apa pun
            self.active = true;
            tracing::info!("HUD volume sistem ditahan; hanya HUD pulau yang tampil");
        }
        let mut pids = pids_named(HELPER);
        if pids.is_empty()
            && self
                .last_kickstart
                .is_none_or(|t| t.elapsed() >= KICKSTART_EVERY)
        {
            // Jalankan lebih dulu supaya yang beku sudah menunggu saat volume berubah (tanpa kedip).
            self.last_kickstart = Some(Instant::now());
            // SAFETY: getuid() tanpa argumen.
            let domain = format!("gui/{}/com.apple.{HELPER}", unsafe { libc::getuid() });
            let _ = Command::new("launchctl")
                .args(["kickstart", &domain])
                .output();
            pids = pids_named(HELPER);
        }
        signal(&pids, libc::SIGSTOP); // berulang tak masalah; proses baru ikut dibekukan
    }

    fn restore(&mut self) {
        self.active = false;
        // KILL, bukan CONT: permintaan HUD yang menumpuk saat beku tidak ikut muncul sekaligus.
        signal(&pids_named(HELPER), libc::SIGKILL);
        tracing::info!("HUD volume sistem dikembalikan");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mencari_proses_dari_nama() {
        // Proses tes ini sendiri: namanya = nama berkas tes (dipotong libproc ke 31 karakter).
        let exe = std::env::current_exe().unwrap();
        let name: String = exe
            .file_name()
            .unwrap()
            .to_string_lossy()
            .chars()
            .take(31)
            .collect();
        assert!(
            pids_named(&name).contains(&(std::process::id() as i32)),
            "nama: {name}"
        );
        assert!(pids_named("proses-yang-tidak-mungkin-ada-xyz").is_empty());
    }

    #[test]
    fn skrip_penjaga_menunggu_isla_lalu_menghentikan_helper() {
        let s = guardian_script(4242);
        assert!(s.contains("kill -0 4242"), "{s}");
        assert!(s.contains("pkill -KILL -x OSDUIHelper"), "{s}");
        assert!(s.find("kill -0").unwrap() < s.find("pkill").unwrap());
    }

    #[test]
    fn setelan_mati_tidak_menyentuh_proses_apa_pun() {
        let mut hud = SystemHudSuppressor::default();
        hud.tick(false);
        assert!(!hud.active);
    }
}
