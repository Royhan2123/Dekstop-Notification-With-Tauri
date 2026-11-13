//! Sumber media macOS: Music.app dan Spotify lewat AppleScript (`osascript`).
//!
//! MediaRemote (sistem) dibatasi Apple sejak macOS 15.4, jadi hanya dua pemutar ini yang didukung.
//! Keduanya tidak mengirim event ke proses lain, sehingga dibaca berkala (polling) dan hanya
//! dilaporkan bila ada perubahan. Pertama kali dipakai, macOS meminta izin "Automation" untuk
//! mengontrol Music/Spotify. Aplikasi yang tidak berjalan tidak pernah disentuh (AppleScript akan
//! membukanya).
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::domain::error::MediaError;
use crate::domain::model::{MediaCommand, MediaInfo};
use crate::domain::ports::MediaSource;

use super::{artwork, position_now};

const SEP: char = '\u{1f}';
const POLL_ACTIVE: Duration = Duration::from_millis(1000);
const POLL_IDLE: Duration = Duration::from_millis(3000);
/// Selisih posisi (ms) yang dianggap "loncat" (seek) dan perlu dilaporkan ulang. Kecil sengaja:
/// lirik berganti tiap beberapa detik, jadi selisih di atas ±0,4 dtk sudah terasa.
const SEEK_TOLERANCE_MS: i64 = 400;
const APPS: [&str; 2] = ["Music", "Spotify"];

/// Satu skrip per pemutar, dijalankan HANYA bila pemutarnya berjalan: menyebut aplikasi yang tidak
/// terpasang di `tell application` membuat AppleScript gagal (atau menanyakan lokasinya).
/// Keluaran: satu baris bila sedang memutar atau dijeda. Pemisah kolom: karakter 31. Posisi dan
/// durasi dalam milidetik (Music memberi detik, Spotify memberi milidetik untuk durasi).
const QUERY_MUSIC: &str = r#"
set sep to (ASCII character 31)
tell application "Music"
	try
		if player state is playing or player state is paused then
			set t to current track
			return "Music" & sep & (player state as text) & sep & (name of t) & sep & (artist of t) & sep & (round ((duration of t) * 1000)) & sep & (round (player position * 1000)) & sep & "" & linefeed
		end if
	end try
end tell
return ""
"#;

const QUERY_SPOTIFY: &str = r#"
set sep to (ASCII character 31)
tell application "Spotify"
	try
		if player state is playing or player state is paused then
			return "Spotify" & sep & (player state as text) & sep & (name of current track) & sep & (artist of current track) & sep & (duration of current track) & sep & (round (player position * 1000)) & sep & (artwork url of current track) & linefeed
		end if
	end try
end tell
return ""
"#;

fn query(app: &str) -> &'static str {
    if app == "Spotify" {
        QUERY_SPOTIFY
    } else {
        QUERY_MUSIC
    }
}

/// Hasil bacaan mentah satu pemutar, sebelum sampul dilampirkan.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Raw {
    pub app: String,
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub duration_ms: u32,
    pub position_ms: u32,
    pub art_url: String,
}

/// Mengurai keluaran `QUERY`; baris rusak atau berstatus berhenti dilewati.
pub(crate) fn parse(output: &str) -> Vec<Raw> {
    output
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split(SEP).collect();
            if f.len() < 7 || !APPS.contains(&f[0]) {
                return None;
            }
            let playing = match f[1].trim() {
                "playing" => true,
                "paused" => false,
                _ => return None,
            };
            Some(Raw {
                app: f[0].to_string(),
                playing,
                title: f[2].trim().to_string(),
                artist: f[3].trim().to_string(),
                duration_ms: f[4].trim().parse().unwrap_or(0),
                position_ms: f[5].trim().parse().unwrap_or(0),
                art_url: f[6].trim().to_string(),
            })
        })
        .filter(|r| !r.title.is_empty())
        .collect()
}

/// Posisi disusulkan ke saat laporan sampai (`age_ms` sejak dibaca), supaya lirik tidak tertinggal.
pub(crate) fn aged(raw: &Raw, age_ms: i64) -> u32 {
    position_now(
        raw.position_ms as i64,
        age_ms,
        1.0,
        raw.playing,
        raw.duration_ms as i64,
    )
}

/// Ada perubahan yang perlu dilaporkan? `elapsed_ms` = waktu sejak laporan terakhir.
pub(crate) fn changed(old: &[MediaInfo], new: &[MediaInfo], elapsed_ms: i64) -> bool {
    if old.len() != new.len() {
        return true;
    }
    old.iter().zip(new).any(|(o, n)| {
        let expected = position_now(
            o.position_ms as i64,
            elapsed_ms,
            o.rate,
            o.playing,
            o.duration_ms as i64,
        ) as i64;
        o.session_id != n.session_id
            || o.title != n.title
            || o.artist != n.artist
            || o.playing != n.playing
            || o.duration_ms != n.duration_ms
            || (expected - n.position_ms as i64).abs() > SEEK_TOLERANCE_MS
    })
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn osascript(script: &str) -> Result<String, String> {
    let out = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("osascript: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn is_running(app: &str) -> bool {
    Command::new("pgrep")
        .args(["-xq", app])
        .status()
        .is_ok_and(|s| s.success())
}

/// Sampul lagu yang sedang diputar: Music lewat berkas sementara, Spotify lewat URL gambarnya.
fn fetch_artwork(raw: &Raw) -> Option<artwork::Artwork> {
    if raw.app == "Spotify" {
        return artwork::fetch(&raw.art_url);
    }
    let bytes = {
        let path = std::env::temp_dir().join("isla-desk-artwork.bin");
        let script = format!(
            "tell application \"Music\"\nset d to raw data of artwork 1 of current track\nend tell\n\
             set f to open for access POSIX file \"{}\" with write permission\n\
             set eof f to 0\nwrite d to f\nclose access f",
            escape(&path.to_string_lossy())
        );
        osascript(&script).ok()?;
        let data = std::fs::read(&path).ok()?;
        let _ = std::fs::remove_file(&path);
        data
    };
    artwork::process(&bytes)
}

#[derive(Default)]
pub struct MacMedia;

impl MediaSource for MacMedia {
    fn start(&self, on_change: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>) {
        let spawned = thread::Builder::new()
            .name("macos-media".into())
            .spawn(move || poll_loop(on_change));
        if let Err(e) = spawned {
            tracing::error!("gagal memulai pembaca media macOS: {e}");
        }
    }

    fn control(&self, session_id: &str, cmd: MediaCommand) -> Result<(), MediaError> {
        let app = APPS
            .iter()
            .find(|a| **a == session_id)
            .ok_or(MediaError::NoSession)?;
        let verb = match cmd {
            MediaCommand::Previous => "previous track",
            MediaCommand::TogglePlay => "playpause",
            MediaCommand::Next => "next track",
        };
        osascript(&format!("tell application \"{app}\" to {verb}"))
            .map(|_| ())
            .map_err(MediaError::Failed)
    }
}

fn poll_loop(on_change: Box<dyn Fn(Vec<MediaInfo>) + Send + Sync>) {
    let mut last: Vec<MediaInfo> = Vec::new();
    let mut last_at = Instant::now();
    // Sampul disimpan per lagu agar tidak diunduh/diekstrak tiap detik.
    let mut art: HashMap<String, Option<artwork::Artwork>> = HashMap::new();
    let mut warned = false;
    loop {
        let running: Vec<&str> = APPS.iter().copied().filter(|a| is_running(a)).collect();
        let mut outputs = String::new();
        let queried_at = Instant::now();
        for app in &running {
            match osascript(query(app)) {
                Ok(out) => outputs.push_str(&out),
                Err(e) if !warned => {
                    tracing::warn!(
                        "tidak bisa membaca media {app} (izinkan Automation di Pengaturan Sistem): {e}"
                    );
                    warned = true;
                }
                Err(_) => {}
            }
        }
        // Posisi dibaca di tengah-tengah panggilan osascript (±100-500 ms): umurnya separuh lamanya.
        let age_ms = (queried_at.elapsed().as_millis() / 2) as i64;
        let sessions: Vec<MediaInfo> = parse(&outputs)
            .into_iter()
            .map(|raw| {
                let position_ms = aged(&raw, age_ms);
                let key = format!("{}|{}|{}", raw.app, raw.title, raw.artist);
                let cover = art.entry(key).or_insert_with(|| fetch_artwork(&raw));
                MediaInfo {
                    session_id: raw.app,
                    title: raw.title,
                    artist: raw.artist,
                    playing: raw.playing,
                    position_ms,
                    duration_ms: raw.duration_ms,
                    rate: 1.0,
                    thumbnail: cover.as_ref().map(|c| c.data_url.clone()),
                    color: cover.as_ref().map(|c| c.color.clone()),
                    ..Default::default()
                }
            })
            .collect();
        if art.len() > 64 {
            art.clear();
        }
        let elapsed = last_at.elapsed().as_millis() as i64;
        if changed(&last, &sessions, elapsed) {
            last = sessions.clone();
            last_at = Instant::now();
            on_change(sessions);
        }
        thread::sleep(if running.is_empty() {
            POLL_IDLE
        } else {
            POLL_ACTIVE
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(f: [&str; 7]) -> String {
        f.join(&SEP.to_string())
    }

    fn info(title: &str, playing: bool, pos: u32) -> MediaInfo {
        MediaInfo {
            session_id: "Music".into(),
            title: title.into(),
            artist: "A".into(),
            playing,
            position_ms: pos,
            duration_ms: 200_000,
            rate: 1.0,
            ..Default::default()
        }
    }

    #[test]
    fn mengurai_dua_pemutar_dan_melewati_baris_rusak_atau_berhenti() {
        let out = [
            line(["Music", "playing", "Lagu A", "Artis", "215000", "12000", ""]),
            line([
                "Spotify",
                "paused",
                "Lagu B",
                "Band",
                "180000",
                "5000",
                "https://x/y.jpg",
            ]),
            line(["Spotify", "stopped", "Lagu C", "X", "1", "1", ""]),
            line(["Podcast", "playing", "Lain", "X", "1", "1", ""]),
            "rusak".to_string(),
            line(["Music", "playing", "", "Artis", "1", "1", ""]),
        ]
        .join("\n");
        let r = parse(&out);
        assert_eq!(r.len(), 2);
        assert_eq!(
            (r[0].app.as_str(), r[0].playing, r[0].duration_ms),
            ("Music", true, 215_000)
        );
        assert_eq!(r[0].position_ms, 12_000);
        assert_eq!((r[1].app.as_str(), r[1].playing), ("Spotify", false));
        assert_eq!(r[1].art_url, "https://x/y.jpg");
    }

    #[test]
    fn judul_dengan_garis_tegak_dan_unicode_tetap_utuh() {
        let out = line(["Music", "playing", "A | B — 夜", "Artis", "1000", "0", ""]);
        assert_eq!(parse(&out)[0].title, "A | B — 夜");
    }

    #[test]
    fn angka_rusak_menjadi_nol_bukan_galat() {
        let out = line(["Music", "playing", "T", "A", "abc", "xyz", ""]);
        let r = parse(&out);
        assert_eq!((r[0].duration_ms, r[0].position_ms), (0, 0));
    }

    #[test]
    fn perubahan_lagu_status_dan_loncat_posisi_dilaporkan() {
        let base = vec![info("T", true, 10_000)];
        // diputar normal: 3 detik berlalu, posisi 13 detik -> tidak berubah
        assert!(!changed(&base, &[info("T", true, 13_000)], 3_000));
        assert!(changed(&base, &[info("Lain", true, 13_000)], 3_000));
        assert!(changed(&base, &[info("T", false, 13_000)], 3_000));
        // loncat (seek) jauh
        assert!(changed(&base, &[info("T", true, 90_000)], 3_000));
        // jeda: posisi tetap -> tidak berubah
        let paused = vec![info("T", false, 10_000)];
        assert!(!changed(&paused, &[info("T", false, 10_000)], 5_000));
        // jumlah sesi berbeda
        assert!(changed(&[], &base, 0));
        assert!(!changed(&[], &[], 0));
    }

    #[test]
    fn posisi_disusulkan_hanya_saat_diputar_dan_tidak_melewati_durasi() {
        let raw = |playing, pos, dur| Raw {
            app: "Music".into(),
            playing,
            title: "t".into(),
            artist: "a".into(),
            duration_ms: dur,
            position_ms: pos,
            art_url: String::new(),
        };
        assert_eq!(aged(&raw(true, 10_000, 200_000), 250), 10_250);
        assert_eq!(aged(&raw(false, 10_000, 200_000), 250), 10_000);
        assert_eq!(aged(&raw(true, 199_900, 200_000), 250), 200_000);
    }

    #[test]
    fn selisih_posisi_di_atas_ambang_kecil_dilaporkan_ulang() {
        let base = vec![info("T", true, 10_000)];
        // 3 dtk berlalu, posisi nyata 13,6 dtk (buffering/loncat kecil): lirik bisa tertinggal 0,6 dtk
        assert!(changed(&base, &[info("T", true, 13_600)], 3_000));
        // selisih 0,2 dtk masih dianggap lancar
        assert!(!changed(&base, &[info("T", true, 13_200)], 3_000));
    }

    #[test]
    fn kontrol_menolak_sesi_asing() {
        let m = MacMedia;
        assert!(matches!(
            m.control("VLC", MediaCommand::Next),
            Err(MediaError::NoSession)
        ));
    }

    #[test]
    fn escape_melindungi_tanda_kutip_di_jalur() {
        assert_eq!(escape(r#"a"b\c"#), r#"a\"b\\c"#);
    }
}
