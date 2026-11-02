use std::sync::LazyLock;

use regex::Regex;

use crate::domain::diff::added_lines;
use crate::domain::model::review::SecretHit;
use crate::domain::ports::SecretScanner;

/// (jenis, pola, indeks grup yang berisi nilai rahasia; 0 = seluruh kecocokan)
const PATTERNS: &[(&str, &str, usize)] = &[
    ("AWS access key", r"\bAKIA[0-9A-Z]{16}\b", 0),
    ("GitHub token", r"\bgh[pousr]_[A-Za-z0-9]{36,}\b", 0),
    ("Anthropic API key", r"\bsk-ant-[A-Za-z0-9_\-]{20,}", 0),
    ("OpenAI API key", r"\bsk-[A-Za-z0-9]{32,}\b", 0),
    ("Slack token", r"\bxox[abprs]-[A-Za-z0-9\-]{10,}", 0),
    ("Google API key", r"\bAIza[0-9A-Za-z_\-]{35}\b", 0),
    ("Private key", r"-----BEGIN (?:[A-Z]+ )?PRIVATE KEY-----", 0),
    (
        "Hardcoded credential",
        r#"(?i)\b(?:api[_-]?key|secret|token|passw(?:or)?d|pwd|private[_-]?key)\b["']?\s*[:=]\s*["']([^"'\s]{8,})["']"#,
        1,
    ),
];

/// Nilai contoh yang bukan rahasia sungguhan.
const PLACEHOLDERS: &[&str] = &[
    "example",
    "changeme",
    "your",
    "placeholder",
    "xxxx",
    "****",
    "dummy",
];

static COMPILED: LazyLock<Vec<(&'static str, Regex, usize)>> = LazyLock::new(|| {
    PATTERNS
        .iter()
        // Pola ada di atas sebagai konstanta; yang tidak valid dilewati dan ditangkap tes.
        .filter_map(|(kind, pat, group)| Regex::new(pat).ok().map(|re| (*kind, re, *group)))
        .collect()
});

pub struct RegexScanner;

fn is_placeholder(value: &str) -> bool {
    let v = value.to_lowercase();
    v.starts_with("${")
        || v.starts_with('<')
        || v.starts_with("{{")
        || PLACEHOLDERS.iter().any(|p| v.contains(p))
}

impl SecretScanner for RegexScanner {
    fn scan(&self, diff: &str) -> Vec<SecretHit> {
        let mut hits: Vec<SecretHit> = Vec::new();
        let mut in_key_file: Option<&str> = None;
        for line in added_lines(diff) {
            // Badan kunci privat: semua baris sampai penutup ikut dianggap rahasia.
            if in_key_file == Some(line.file) {
                if line.text.contains("-----END") {
                    in_key_file = None;
                } else if !line.text.trim().is_empty() {
                    hits.push(SecretHit {
                        file: line.file.to_string(),
                        line: line.line,
                        kind: "Private key",
                        secret: line.text.to_string(),
                    });
                }
                continue;
            }
            for (kind, re, group) in COMPILED.iter() {
                for caps in re.captures_iter(line.text) {
                    let Some(m) = caps.get(*group) else { continue };
                    if *group > 0 && is_placeholder(m.as_str()) {
                        continue;
                    }
                    if *kind == "Private key" {
                        in_key_file = Some(line.file);
                    }
                    let hit = SecretHit {
                        file: line.file.to_string(),
                        line: line.line,
                        kind,
                        secret: m.as_str().to_string(),
                    };
                    if !hits.contains(&hit) {
                        hits.push(hit);
                    }
                }
            }
        }
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff_with(lines: &[&str]) -> String {
        let body: String = lines.iter().map(|l| format!("+{l}\n")).collect();
        format!(
            "--- a/x.env\n+++ b/x.env\n@@ -0,0 +1,{} @@\n{body}",
            lines.len()
        )
    }

    fn kinds(lines: &[&str]) -> Vec<&'static str> {
        RegexScanner
            .scan(&diff_with(lines))
            .iter()
            .map(|h| h.kind)
            .collect()
    }

    #[test]
    fn semua_pola_terkompilasi() {
        assert_eq!(COMPILED.len(), PATTERNS.len());
    }

    #[test]
    fn rahasia_umum_terdeteksi() {
        let cases = [
            ("aws = \"AKIAIOSFODNN7EXAMPLQ\"", "AWS access key"),
            ("t=ghp_abcdefghijklmnopqrstuvwxyz0123456789", "GitHub token"),
            ("k=sk-ant-api03-abcdefghijklmnopqrstuv", "Anthropic API key"),
            ("k=sk-abcdefghijklmnopqrstuvwxyz012345", "OpenAI API key"),
            ("s=xoxb-1234567890-abcdefghij", "Slack token"),
            (
                "g=AIzaSyA-abcdefghijklmnopqrstuvwxyz01234",
                "Google API key",
            ),
            ("-----BEGIN RSA PRIVATE KEY-----", "Private key"),
            (
                "const password = \"hunter2hunter2\";",
                "Hardcoded credential",
            ),
            ("api_key: 'abcd1234efgh5678'", "Hardcoded credential"),
        ];
        for (line, kind) in cases {
            assert_eq!(kinds(&[line]), vec![kind], "{line}");
        }
    }

    #[test]
    fn bukan_rahasia_tidak_ditandai() {
        for line in [
            "password = process.env.PASSWORD",
            "const token = getToken();",
            "api_key: \"changeme123\"",
            "secret = \"${SECRET_VALUE}\"",
            "let k = \"sk-pendek\"",
            "// tidak ada apa-apa",
        ] {
            assert!(kinds(&[line]).is_empty(), "{line}");
        }
    }

    #[test]
    fn hanya_baris_yang_ditambahkan_dipindai_dan_nomor_baris_benar() {
        let diff = "--- a/c.txt\n+++ b/c.txt\n@@ -1,2 +1,2 @@\n-password = \"lama-lama-1\"\n ctx\n+password = \"baru-baru-22\"\n";
        let hits = RegexScanner.scan(diff);
        assert_eq!(hits.len(), 1);
        assert_eq!((hits[0].file.as_str(), hits[0].line), ("c.txt", 2));
        assert_eq!(hits[0].secret, "baru-baru-22");
    }

    #[test]
    fn badan_kunci_privat_ikut_ditandai() {
        let hits = RegexScanner.scan(&diff_with(&[
            "-----BEGIN PRIVATE KEY-----",
            "MIIEvQIBADANBgkqhkiG9w0BAQEFAASC",
            "-----END PRIVATE KEY-----",
            "biasa",
        ]));
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[1].secret, "MIIEvQIBADANBgkqhkiG9w0BAQEFAASC");
    }
}
