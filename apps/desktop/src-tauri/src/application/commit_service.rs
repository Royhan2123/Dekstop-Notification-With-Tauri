use std::path::Path;
use std::sync::Arc;

use crate::application::review::redact::redact;
use crate::application::settings_service::SettingsService;
use crate::domain::error::{AiError, GitError};
use crate::domain::model::settings::Language;
use crate::domain::ports::{AiProviderFactory, GitRepo, SecretScanner};

const MAX_MESSAGE_CHARS: usize = 1000;

/// Kesalahan menyusun pesan commit; `Display`-nya langsung dibaca pengguna di terminal.
#[derive(Debug, thiserror::Error)]
pub enum CommitError {
    #[error("tidak ada perubahan staged; jalankan `git add` dulu")]
    NothingStaged,
    #[error("{0}")]
    Git(#[from] GitError),
    #[error("{0}")]
    Ai(#[from] AiError),
}

/// Saran pesan commit dari perubahan yang sudah di-`git add`.
pub struct CommitService {
    git: Arc<dyn GitRepo>,
    scanner: Arc<dyn SecretScanner>,
    ai: Arc<dyn AiProviderFactory>,
    settings: Arc<SettingsService>,
}

impl CommitService {
    pub fn new(
        git: Arc<dyn GitRepo>,
        scanner: Arc<dyn SecretScanner>,
        ai: Arc<dyn AiProviderFactory>,
        settings: Arc<SettingsService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            git,
            scanner,
            ai,
            settings,
        })
    }

    /// Pesan commit untuk diff staged di `repo`. Rahasia disensor sebelum diff dikirim ke AI,
    /// sama seperti pada review push.
    pub fn suggest(&self, repo: &str) -> Result<String, CommitError> {
        let root = self.git.toplevel(Path::new(repo))?;
        let diff = self.git.staged_diff(Path::new(&root))?;
        if diff.text.trim().is_empty() {
            return Err(CommitError::NothingStaged);
        }
        let hits = self.scanner.scan(&diff.text);
        let safe = redact(&diff.text, &hits);
        let s = self.settings.get();
        let system = match s.language {
            Language::Id => {
                "Kamu menulis pesan commit git dari diff. Baris pertama: ringkasan imperatif \
                maksimal 72 karakter dalam bahasa Indonesia, tanpa titik di akhir. Bila perlu, \
                baris kosong lalu satu sampai tiga butir singkat tentang alasan perubahan. \
                Balas hanya dengan pesan commit, tanpa penjelasan, tanda kutip, atau markdown."
            }
            Language::En => {
                "You write git commit messages from a diff. First line: imperative summary of \
                at most 72 characters, no trailing period. If useful, a blank line and one to \
                three short bullets about why. Reply with only the commit message, no \
                explanation, quotes, or markdown."
            }
        };
        let user = if diff.truncated {
            format!("{safe}\n(diff dipotong karena terlalu besar)")
        } else {
            safe
        };
        let raw = self
            .ai
            .build(s.ai_provider, &s.ai_model)?
            .complete(system, &user)?;
        Ok(clean_message(&raw))
    }
}

/// Buang pagar kode, tanda kutip pembungkus, dan awalan "Commit message:" yang sering ikut dari AI.
fn clean_message(raw: &str) -> String {
    let mut text = raw.trim();
    if let Some(inner) = text.strip_prefix("```") {
        let inner = inner.split_once('\n').map_or(inner, |(_, rest)| rest);
        text = inner
            .rsplit_once("```")
            .map_or(inner, |(body, _)| body)
            .trim();
    }
    let without_label = ["Commit message:", "Pesan commit:"]
        .iter()
        .find_map(|p| text.strip_prefix(p))
        .unwrap_or(text)
        .trim();
    let unquoted = without_label
        .strip_prefix('"')
        .and_then(|t| t.strip_suffix('"'))
        .unwrap_or(without_label);
    unquoted.trim().chars().take(MAX_MESSAGE_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::model::review::{Diff, PushRef};
    use crate::domain::model::settings::{AiProvider, Settings};
    use crate::domain::ports::AiClient;
    use crate::infrastructure::security::regex_scanner::RegexScanner;
    use std::sync::Mutex;

    struct Git(String);
    impl GitRepo for Git {
        fn diff_for_push(&self, _: &Path, _: &[PushRef]) -> Result<Diff, GitError> {
            unreachable!()
        }
        fn staged_diff(&self, _: &Path) -> Result<Diff, GitError> {
            Ok(Diff {
                text: self.0.clone(),
                truncated: false,
            })
        }
        fn toplevel(&self, p: &Path) -> Result<String, GitError> {
            Ok(p.to_string_lossy().into_owned())
        }
        fn check_patch(&self, _: &Path, _: &str) -> Result<(), GitError> {
            unreachable!()
        }
        fn apply_and_commit(&self, _: &Path, _: &str, _: &str) -> Result<(), GitError> {
            unreachable!()
        }
    }

    struct Ai(Mutex<Vec<String>>, String);
    impl AiClient for Ai {
        fn complete(&self, _: &str, user: &str) -> Result<String, AiError> {
            self.0.lock().unwrap().push(user.into());
            Ok(self.1.clone())
        }
    }
    struct Factory(Arc<Ai>);
    impl AiProviderFactory for Factory {
        fn build(&self, _: AiProvider, _: &str) -> Result<Arc<dyn AiClient>, AiError> {
            Ok(self.0.clone())
        }
    }

    fn svc(diff: &str, reply: &str) -> (Arc<CommitService>, Arc<Ai>, tokio::runtime::Runtime) {
        let (settings, _island, rt) = services(Settings::default());
        let ai = Arc::new(Ai(Mutex::new(Vec::new()), reply.into()));
        let svc = CommitService::new(
            Arc::new(Git(diff.into())),
            Arc::new(RegexScanner),
            Arc::new(Factory(ai.clone())),
            settings,
        );
        (svc, ai, rt)
    }

    #[test]
    fn diff_kosong_ditolak_tanpa_menghubungi_ai() {
        let (svc, ai, _rt) = svc("  \n", "x");
        assert!(matches!(svc.suggest("/r"), Err(CommitError::NothingStaged)));
        assert!(ai.0.lock().unwrap().is_empty());
    }

    #[test]
    fn pesan_dibersihkan_dan_rahasia_tidak_dikirim_ke_ai() {
        let key = format!("AKIA{}", "ABCDEFGHIJKLMNOP");
        let diff = format!("+++ b/a.env\n@@ -0,0 +1 @@\n+AWS_KEY={key}\n");
        let (svc, ai, _rt) = svc(&diff, "```\nfix: perbaiki kunci\n```");
        assert_eq!(svc.suggest("/r").unwrap(), "fix: perbaiki kunci");
        let sent = ai.0.lock().unwrap().join("\n");
        assert!(!sent.contains(&key), "rahasia bocor ke AI: {sent}");
    }

    #[test]
    fn pembersih_membuang_pagar_label_dan_kutip() {
        assert_eq!(
            clean_message("```text\nfeat: a\n\n- b\n```"),
            "feat: a\n\n- b"
        );
        assert_eq!(clean_message("Commit message: feat: a"), "feat: a");
        assert_eq!(clean_message("\"feat: a\""), "feat: a");
        assert_eq!(clean_message("  feat: a \n"), "feat: a");
        assert_eq!(
            clean_message(&"x".repeat(5000)).chars().count(),
            MAX_MESSAGE_CHARS
        );
    }
}
