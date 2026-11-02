use std::fmt;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    Important,
    /// Nilai tak dikenal dari AI jatuh ke sini.
    #[default]
    #[serde(other)]
    Suggestion,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub line: Option<u32>,
    #[serde(default)]
    pub severity: Severity,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub suggestion: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewReport {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub findings: Vec<Finding>,
    /// Ada patch perbaikan yang menunggu persetujuan; tidak berasal dari AI.
    #[serde(default, skip_deserializing)]
    pub patch_ready: bool,
}

impl ReviewReport {
    pub fn count(&self, severity: Severity) -> u32 {
        self.findings
            .iter()
            .filter(|f| f.severity == severity)
            .count() as u32
    }
}

/// Satu baris dari stdin hook `pre-push`: `<local ref> <local sha> <remote ref> <remote sha>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushRef {
    pub local_ref: String,
    pub local_sha: String,
    pub remote_ref: String,
    pub remote_sha: String,
}

impl PushRef {
    pub fn parse(line: &str) -> Option<Self> {
        let mut it = line.split_whitespace();
        let r = Self {
            local_ref: it.next()?.to_string(),
            local_sha: it.next()?.to_string(),
            remote_ref: it.next()?.to_string(),
            remote_sha: it.next()?.to_string(),
        };
        it.next().is_none().then_some(r)
    }
}

/// Diff yang akan direview. `truncated` bila dipotong karena terlalu besar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Diff {
    pub text: String,
    pub truncated: bool,
}

/// Rahasia yang terdeteksi di baris yang ditambahkan. Nilainya tidak pernah dilog atau dikirim.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretHit {
    pub file: String,
    pub line: u32,
    pub kind: &'static str,
    pub secret: String,
}

impl fmt::Debug for SecretHit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretHit({}:{} {})", self.file, self.line, self.kind)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    /// Minta AI membuat patch perbaikan.
    Fix,
    /// Lanjutkan push apa adanya.
    Continue,
    Cancel,
    /// Setujui patch lalu commit.
    ApplyPatch,
    RejectPatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_ref_parse_menolak_baris_rusak() {
        let r = PushRef::parse("refs/heads/main aaa refs/heads/main bbb").unwrap();
        assert_eq!(
            (r.local_sha.as_str(), r.remote_sha.as_str()),
            ("aaa", "bbb")
        );
        assert!(PushRef::parse("a b c").is_none());
        assert!(PushRef::parse("a b c d e").is_none());
    }

    #[test]
    fn severity_tak_dikenal_menjadi_saran_dan_hit_tidak_membocorkan_nilai() {
        let f: Finding = serde_json::from_str(r#"{"severity":"aneh","title":"x"}"#).unwrap();
        assert_eq!(f.severity, Severity::Suggestion);
        let hit = SecretHit {
            file: "a".into(),
            line: 1,
            kind: "k",
            secret: "RAHASIA".into(),
        };
        assert!(!format!("{hit:?}").contains("RAHASIA"));
    }
}
