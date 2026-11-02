use crate::domain::model::review::SecretHit;

pub const MASK: &str = "[REDACTED]";

/// Sensor semua kemunculan rahasia (juga di baris konteks) sebelum diff dikirim ke AI.
pub fn redact(diff: &str, hits: &[SecretHit]) -> String {
    let mut secrets: Vec<&str> = hits
        .iter()
        .map(|h| h.secret.as_str())
        .filter(|s| !s.is_empty())
        .collect();
    // Yang terpanjang dulu agar rahasia yang memuat rahasia lain tersensor utuh.
    secrets.sort_by_key(|s| std::cmp::Reverse(s.len()));
    secrets.dedup();
    secrets
        .into_iter()
        .fold(diff.to_string(), |text, s| text.replace(s, MASK))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(secret: &str) -> SecretHit {
        SecretHit {
            file: "f".into(),
            line: 1,
            kind: "k",
            secret: secret.into(),
        }
    }

    #[test]
    fn semua_kemunculan_disensor_termasuk_yang_saling_memuat() {
        let diff = "+a=RAHASIA-PANJANG\n ctx RAHASIA\n+b=RAHASIA\n";
        let out = redact(diff, &[hit("RAHASIA"), hit("RAHASIA-PANJANG")]);
        assert_eq!(out, "+a=[REDACTED]\n ctx [REDACTED]\n+b=[REDACTED]\n");
        assert!(!out.contains("RAHASIA"));
    }

    #[test]
    fn tanpa_hit_diff_tidak_berubah() {
        assert_eq!(redact("+x\n", &[]), "+x\n");
    }
}
