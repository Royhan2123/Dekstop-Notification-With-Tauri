//! Lirik berwaktu: parser format LRC, pembersih judul dari YouTube, dan pemilih hasil pencarian.

/// Satu baris lirik yang mulai dinyanyikan pada `ms`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LyricLine {
    pub ms: u32,
    pub text: String,
}

/// Hasil pencarian dari sumber lirik.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub artist: String,
    pub track: String,
    pub duration_secs: f64,
    /// Lirik berformat LRC; `None` bila sumber hanya punya teks polos.
    pub synced: Option<String>,
}

/// Selisih durasi terbesar (ms) antara lagu yang diputar dan hasil pencarian agar dianggap lagu yang sama.
pub const DURATION_TOLERANCE_MS: u32 = 3000;

/// Tangkap `[mm:ss]`, `[mm:ss.x]`, `[mm:ss.xx]`, atau `[mm:ss.xxx]`; tag metadata (`[ar:...]`) dilewati.
fn timestamp(tag: &str) -> Option<u32> {
    let (min, rest) = tag.split_once(':')?;
    let (sec, frac) = rest.split_once('.').unwrap_or((rest, ""));
    let min: u32 = min.parse().ok()?;
    let sec: u32 = sec.parse().ok()?;
    if sec >= 60 || frac.len() > 3 || !frac.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let frac_ms = match frac.len() {
        0 => 0,
        n => frac.parse::<u32>().ok()? * 10u32.pow(3 - n as u32),
    };
    Some(min.checked_mul(60_000)? + sec * 1000 + frac_ms)
}

/// Ubah teks LRC menjadi baris berurutan waktu. Satu baris boleh punya beberapa penanda waktu
/// (refrain); baris tanpa teks dipertahankan sebagai jeda supaya baris sebelumnya tidak menggantung.
pub fn parse_lrc(text: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let mut rest = raw.trim_start();
        let mut stamps = Vec::new();
        while let Some(after) = rest.strip_prefix('[') {
            let Some((tag, tail)) = after.split_once(']') else {
                break;
            };
            match timestamp(tag) {
                Some(ms) => stamps.push(ms),
                None if stamps.is_empty() => {}
                None => break,
            }
            rest = tail.trim_start();
        }
        let content = rest.trim();
        lines.extend(stamps.into_iter().map(|ms| LyricLine {
            ms,
            text: content.to_string(),
        }));
    }
    lines.sort_by_key(|l| l.ms);
    lines
}

/// Indeks baris yang sedang dinyanyikan pada `position_ms`; `None` sebelum baris pertama.
pub fn current_index(lines: &[LyricLine], position_ms: u32) -> Option<usize> {
    lines
        .partition_point(|l| l.ms <= position_ms)
        .checked_sub(1)
}

/// Huruf kecil tanpa tanda baca dan spasi, untuk membandingkan nama.
fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Apakah hasil pencarian memang lagu yang diputar. Judul hasil harus sama dengan (atau diawali) judul
/// pemutar atau salah satu bagiannya ("Artis - Lagu" di YouTube); artisnya harus mirip nama artis
/// pemutar atau muncul di judulnya. Durasi saja tidak cukup: lagu lain bisa kebetulan sama panjang.
fn same_song(c: &Candidate, artist: &str, title: &str) -> bool {
    let track = norm(&c.track);
    let title_clean = clean(title);
    let track_ok = track.len() >= 2
        && std::iter::once(title_clean.as_str())
            .chain(title_clean.split(" - "))
            .map(norm)
            .any(|s| {
                s.len() >= 2
                    && (s == track
                        || (s.len().min(track.len()) >= 3
                            && (s.starts_with(&track) || track.starts_with(&s))))
            });
    let artist = norm(&clean(artist));
    let found = norm(&c.artist);
    let artist_ok = artist.is_empty()
        || (!found.is_empty()
            && (artist.contains(&found)
                || found.contains(&artist)
                || norm(title).contains(&found)));
    track_ok && artist_ok
}

/// Hasil berlirik berwaktu yang cocok dengan lagu dan durasinya paling dekat (dalam toleransi).
pub fn pick_best<'a>(
    candidates: &'a [Candidate],
    artist: &str,
    title: &str,
    duration_ms: u32,
) -> Option<&'a Candidate> {
    let target = f64::from(duration_ms);
    candidates
        .iter()
        .filter(|c| c.synced.as_deref().is_some_and(|s| !s.trim().is_empty()))
        .filter(|c| same_song(c, artist, title))
        .map(|c| (c, (c.duration_secs * 1000.0 - target).abs()))
        .filter(|(_, d)| *d <= f64::from(DURATION_TOLERANCE_MS))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(c, _)| c)
}

/// Buang bagian dalam kurung `(...)`/`[...]`, ekor setelah `|`, dan penanda kanal YouTube.
fn clean(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for c in s.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '|' if depth == 0 => break,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.replace(['\u{2013}', '\u{2014}'], "-")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Kueri pencarian dari artis dan judul yang dilaporkan pemutar, paling spesifik dulu.
pub fn search_queries(artist: &str, title: &str) -> Vec<String> {
    let artist = clean(artist);
    let artist = artist
        .trim_end_matches("- Topic")
        .trim_end_matches("VEVO")
        .trim();
    let title = clean(title);
    let mut queries = Vec::new();
    if !title.is_empty() {
        if !artist.is_empty() {
            queries.push(format!("{artist} {title}"));
        }
        queries.push(title);
    }
    queries.dedup();
    queries
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(ms: u32, text: &str) -> LyricLine {
        LyricLine {
            ms,
            text: text.into(),
        }
    }

    #[test]
    fn parse_lrc_membaca_format_waktu_dan_melewati_metadata() {
        let lrc = "[ar:Seseorang]\n[ti:Judul]\n[00:12.50] halo\n[00:15.5]dua\n[01:02.345]tiga\n[00:20]empat\n";
        assert_eq!(
            parse_lrc(lrc),
            [
                line(12_500, "halo"),
                line(15_500, "dua"),
                line(20_000, "empat"),
                line(62_345, "tiga"),
            ]
        );
    }

    #[test]
    fn parse_lrc_beberapa_penanda_per_baris_jeda_kosong_dan_sampah() {
        let lrc = "[00:30.00][01:10.00]refrain\n[00:40.00]\nbaris tanpa waktu\n[99:99.99]ngawur\n[00:xx.00]rusak\n";
        let l = parse_lrc(lrc);
        assert_eq!(
            l,
            [
                line(30_000, "refrain"),
                line(40_000, ""),
                line(70_000, "refrain")
            ]
        );
        assert!(parse_lrc("").is_empty());
    }

    #[test]
    fn baris_aktif_mengikuti_posisi() {
        let l = [line(1000, "a"), line(5000, "b"), line(9000, "")];
        assert_eq!(current_index(&l, 0), None);
        assert_eq!(current_index(&l, 1000), Some(0));
        assert_eq!(current_index(&l, 4999), Some(0));
        assert_eq!(current_index(&l, 5000), Some(1));
        assert_eq!(current_index(&l, 99_999), Some(2));
        assert_eq!(current_index(&[], 500), None);
    }

    fn cand(secs: f64, synced: Option<&str>) -> Candidate {
        named("The Weeknd", "Blinding Lights", secs, synced)
    }

    fn named(artist: &str, track: &str, secs: f64, synced: Option<&str>) -> Candidate {
        Candidate {
            artist: artist.into(),
            track: track.into(),
            duration_secs: secs,
            synced: synced.map(Into::into),
        }
    }

    #[test]
    fn pilihan_terbaik_butuh_lirik_berwaktu_dan_durasi_yang_cocok() {
        let cs = [
            cand(4.0, Some("[00:01.00]salah durasi")),
            cand(200.0, None),
            cand(201.5, Some("[00:01.00]hampir")),
            cand(199.2, Some("[00:01.00]paling dekat")),
            cand(199.0, Some("   ")),
        ];
        let best = pick_best(&cs, "The Weeknd", "Blinding Lights", 199_000).unwrap();
        assert_eq!(best.synced.as_deref(), Some("[00:01.00]paling dekat"));
        assert!(pick_best(&cs, "The Weeknd", "Blinding Lights", 500_000).is_none());
        assert!(pick_best(&[], "a", "b", 1000).is_none());
    }

    #[test]
    fn lagu_lain_dengan_durasi_sama_ditolak_tetapi_variasi_penulisan_diterima() {
        let lrc = Some("[00:01.00]x");
        let other = [named("Seseorang", "In The Void", 278.0, lrc)];
        assert!(
            pick_best(&other, "HBRP", "VOID", 278_000).is_none(),
            "judul beda"
        );
        let wrong_artist = [named("Orang Lain", "Void", 278.0, lrc)];
        assert!(
            pick_best(&wrong_artist, "HBRP", "VOID", 278_000).is_none(),
            "artis beda"
        );

        let yt = [named("Tulus", "Hati-hati di jalan", 246.0, lrc)];
        // YouTube: kanal bukan artis, judul berformat "Artis - Lagu"
        assert!(
            pick_best(
                &yt,
                "Kanal Musik",
                "Tulus - Hati-Hati di Jalan (Official)",
                246_000
            )
            .is_some()
        );
        assert!(pick_best(&yt, "Tulus - Topic", "Hati-Hati di Jalan", 246_000).is_some());
        // artis pemutar tidak diketahui: cukup judul
        assert!(pick_best(&yt, "", "Hati-Hati di Jalan", 246_000).is_some());
        // hasil dengan tambahan (feat.) di belakang judul tetap cocok
        let feat = [named(
            "Weird Genius",
            "Lathi (feat. Sara Fajira)",
            189.0,
            lrc,
        )];
        assert!(pick_best(&feat, "Weird Genius", "Lathi", 189_000).is_some());
    }

    #[test]
    fn kueri_dibersihkan_dari_judul_youtube() {
        assert_eq!(
            search_queries("Tulus - Topic", "Hati-Hati di Jalan (Official Video) [HD]"),
            ["Tulus Hati-Hati di Jalan", "Hati-Hati di Jalan"]
        );
        assert_eq!(
            search_queries("SOUND OF RAX", "VOID \u{2013} HBRP | REMIX | DARK"),
            ["SOUND OF RAX VOID - HBRP", "VOID - HBRP"]
        );
        assert_eq!(search_queries("", "Lathi"), ["Lathi"]);
        assert!(search_queries("x", "(cuma kurung)").is_empty());
    }
}
