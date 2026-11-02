use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::application::island_service::IslandService;
use crate::application::settings_service::SettingsService;
use crate::domain::error::{AiError, SettingsError};
use crate::domain::island::action::IslandAction;
use crate::domain::island::state::Content;
use crate::domain::model::ReviewId;
use crate::domain::model::review::{Decision, Finding, PushRef, ReviewReport, SecretHit, Severity};
use crate::domain::model::settings::ReviewTimeoutAction;
use crate::domain::ports::{AiProviderFactory, GitRepo, HookInstaller, SecretScanner};

use super::prompt::{
    PATCH_SYSTEM, REVIEW_SYSTEM, extract_patch, parse_report, patch_user, review_user,
};
use super::redact::redact;

/// Jawaban untuk hook `pre-push`: lanjutkan push atau hentikan dengan pesan.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Block(String),
}

pub struct ReviewService {
    git: Arc<dyn GitRepo>,
    hooks: Arc<dyn HookInstaller>,
    scanner: Arc<dyn SecretScanner>,
    ai: Arc<dyn AiProviderFactory>,
    settings: Arc<SettingsService>,
    island: Arc<IslandService>,
    decision_timeout: Duration,
    next_id: AtomicU64,
    /// Satu review aktif pada satu waktu; push berikutnya antre di sini.
    gate: Mutex<()>,
    pending: Mutex<Option<Sender<Decision>>>,
}

fn norm(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

fn repo_error(e: impl ToString) -> SettingsError {
    SettingsError::InvalidRepo(e.to_string())
}

impl ReviewService {
    pub fn new(
        git: Arc<dyn GitRepo>,
        hooks: Arc<dyn HookInstaller>,
        scanner: Arc<dyn SecretScanner>,
        ai: Arc<dyn AiProviderFactory>,
        settings: Arc<SettingsService>,
        island: Arc<IslandService>,
        decision_timeout: Duration,
    ) -> Arc<Self> {
        Arc::new(Self {
            git,
            hooks,
            scanner,
            ai,
            settings,
            island,
            decision_timeout,
            next_id: AtomicU64::new(1),
            gate: Mutex::new(()),
            pending: Mutex::new(None),
        })
    }

    /// Pasang hook di repo dan catat sebagai repo yang diawasi.
    pub fn watch(&self, path: &str) -> Result<(), SettingsError> {
        let top = self
            .git
            .toplevel(Path::new(path.trim()))
            .map_err(repo_error)?;
        self.hooks.install(Path::new(&top)).map_err(repo_error)?;
        let mut s = self.settings.get();
        if !s.watched_repos.iter().any(|r| norm(r) == norm(&top)) {
            s.watched_repos.push(top);
            self.settings.update(s)?;
        }
        Ok(())
    }

    pub fn unwatch(&self, path: &str) -> Result<(), SettingsError> {
        // Folder yang sudah hilang tetap boleh dilepas dari daftar.
        let _ = self.hooks.uninstall(Path::new(path));
        let mut s = self.settings.get();
        s.watched_repos.retain(|r| norm(r) != norm(path));
        self.settings.update(s)?;
        Ok(())
    }

    /// Keputusan pengguna dari kartu review; diabaikan bila tidak ada review yang menunggu.
    pub fn decide(&self, decision: Decision) {
        if let Some(tx) = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            let _ = tx.send(decision);
        }
    }

    /// Dipanggil hook `pre-push` (lewat IPC). Memblokir sampai ada keputusan; thread non-UI.
    pub fn review_push(&self, repo: &str, refs: &[PushRef]) -> Verdict {
        let Ok(top) = self.git.toplevel(Path::new(repo)) else {
            return Verdict::Allow;
        };
        let watched = self.settings.get().watched_repos;
        if !watched.iter().any(|r| norm(r) == norm(&top)) {
            return Verdict::Allow;
        }
        let _turn = self.gate.lock().unwrap_or_else(|e| e.into_inner());
        let id = ReviewId(self.next_id.fetch_add(1, Ordering::Relaxed));
        let verdict = self.run(id, Path::new(&top), refs);
        *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = None;
        // Hasil bersih menutup sendiri lewat timer pulau; yang berisi temuan ditutup di sini.
        if matches!(self.island.snapshot().view.content(), Some(Content::ReviewResult(r)) if !r.findings.is_empty())
        {
            self.island.dispatch(IslandAction::Dismissed);
        }
        verdict
    }

    fn run(&self, id: ReviewId, repo: &Path, refs: &[PushRef]) -> Verdict {
        let diff = match self.git.diff_for_push(repo, refs) {
            Ok(d) if !d.text.trim().is_empty() => d,
            Ok(_) => return Verdict::Allow,
            // Gagal membaca diff tidak boleh menahan push.
            Err(_) => return Verdict::Allow,
        };
        self.island.dispatch(IslandAction::ReviewStarted(id));
        let hits = self.scanner.scan(&diff.text);
        let safe = redact(&diff.text, &hits);
        let mut report = self.review(&review_user(&safe, diff.truncated), &hits);
        let rx = {
            let (tx, rx) = mpsc::channel();
            *self.pending.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
            rx
        };
        self.island
            .dispatch(IslandAction::ReviewFinished(id, report.clone()));
        if report.findings.is_empty() {
            return Verdict::Allow;
        }

        let mut patch: Option<String> = None;
        loop {
            match self.wait(&rx) {
                Decision::Continue => return Verdict::Allow,
                Decision::Cancel => return Verdict::Block("Push dibatalkan lewat Isla.".into()),
                Decision::Fix if patch.is_none() => {
                    match self.make_patch(repo, &safe, &report) {
                        Ok(p) => {
                            patch = Some(p);
                            report.patch_ready = true;
                        }
                        Err(e) => {
                            report.summary = format!("{}\nPerbaikan gagal: {e}", report.summary)
                        }
                    }
                    self.island
                        .dispatch(IslandAction::ReviewFinished(id, report.clone()));
                }
                Decision::ApplyPatch if patch.is_some() => {
                    let p = patch.take().unwrap_or_default();
                    match self
                        .git
                        .apply_and_commit(repo, &p, "fix: perbaikan dari review Isla")
                    {
                        Ok(()) => {
                            return Verdict::Block(
                                "Isla sudah meng-commit perbaikan. Jalankan git push lagi.".into(),
                            );
                        }
                        Err(e) => {
                            report.patch_ready = false;
                            report.summary = format!("{}\nPatch ditolak: {e}", report.summary);
                            self.island
                                .dispatch(IslandAction::ReviewFinished(id, report.clone()));
                        }
                    }
                }
                Decision::RejectPatch if patch.is_some() => {
                    patch = None;
                    report.patch_ready = false;
                    self.island
                        .dispatch(IslandAction::ReviewFinished(id, report.clone()));
                }
                // Keputusan yang tidak cocok dengan keadaan saat ini diabaikan.
                _ => {}
            }
        }
    }

    /// Temuan dari pemindai lokal selalu ikut, walau AI gagal.
    fn review(&self, user_prompt: &str, hits: &[SecretHit]) -> ReviewReport {
        let s = self.settings.get();
        let asked = self
            .ai
            .build(s.ai_provider, &s.ai_model)
            .and_then(|ai| ai.complete(REVIEW_SYSTEM, user_prompt))
            .and_then(|text| parse_report(&text));
        let mut report = match asked {
            Ok(r) => r,
            Err(e) => ReviewReport {
                summary: match e {
                    AiError::BadResponse => format!("Review AI dilewati: {e}"),
                    _ => format!("Review AI dilewati. {e}"),
                },
                ..Default::default()
            },
        };
        let local = hits.iter().map(|h| Finding {
            file: h.file.clone(),
            line: Some(h.line),
            severity: Severity::Critical,
            title: format!("Rahasia terdeteksi: {}", h.kind),
            detail: "Nilainya tidak dikirim ke AI. Hapus dari commit dan putar ulang kuncinya."
                .into(),
            suggestion: None,
        });
        report.findings.splice(0..0, local);
        report
    }

    fn make_patch(&self, repo: &Path, diff: &str, report: &ReviewReport) -> Result<String, String> {
        let s = self.settings.get();
        let ai = self
            .ai
            .build(s.ai_provider, &s.ai_model)
            .map_err(|e| e.to_string())?;
        let text = ai
            .complete(PATCH_SYSTEM, &patch_user(diff, report))
            .map_err(|e| e.to_string())?;
        let patch = extract_patch(&text).map_err(|e| e.to_string())?;
        self.git
            .check_patch(repo, &patch)
            .map_err(|e| e.to_string())?;
        Ok(patch)
    }

    fn wait(&self, rx: &mpsc::Receiver<Decision>) -> Decision {
        match rx.recv_timeout(self.decision_timeout) {
            Ok(d) => d,
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => {
                match self.settings.get().review_timeout_action {
                    ReviewTimeoutAction::Continue => Decision::Continue,
                    ReviewTimeoutAction::Cancel => Decision::Cancel,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::GitError;
    use crate::domain::island::state::{IslandState, View};
    use crate::domain::model::review::Diff;
    use crate::domain::model::settings::Settings;
    use crate::domain::ports::{
        AiClient, Autostart, SecretStore, SettingsRepository, SettingsSink, SnapshotSink,
    };
    use crate::infrastructure::security::regex_scanner::RegexScanner;
    use std::thread;

    struct NullSink;
    impl SnapshotSink for NullSink {
        fn publish(&self, _: &IslandState) {}
    }
    impl SettingsSink for NullSink {
        fn publish(&self, _: &Settings) {}
    }
    struct MemRepo(Mutex<Settings>);
    impl SettingsRepository for MemRepo {
        fn load(&self) -> Settings {
            self.0.lock().unwrap().clone()
        }
        fn save(&self, s: &Settings) -> Result<(), SettingsError> {
            *self.0.lock().unwrap() = s.clone();
            Ok(())
        }
    }
    struct NoSecrets;
    impl SecretStore for NoSecrets {
        fn get(&self, _: &str) -> Result<Option<String>, SettingsError> {
            Ok(None)
        }
        fn set(&self, _: &str, _: &str) -> Result<(), SettingsError> {
            Ok(())
        }
        fn delete(&self, _: &str) -> Result<(), SettingsError> {
            Ok(())
        }
    }
    struct NoAutostart;
    impl Autostart for NoAutostart {
        fn set(&self, _: bool) -> Result<(), SettingsError> {
            Ok(())
        }
    }

    struct FakeGit {
        diff: String,
        committed: Mutex<Vec<String>>,
    }
    impl GitRepo for FakeGit {
        fn diff_for_push(&self, _: &Path, _: &[PushRef]) -> Result<Diff, GitError> {
            Ok(Diff {
                text: self.diff.clone(),
                truncated: false,
            })
        }
        fn toplevel(&self, p: &Path) -> Result<String, GitError> {
            Ok(p.to_string_lossy().into_owned())
        }
        fn check_patch(&self, _: &Path, _: &str) -> Result<(), GitError> {
            Ok(())
        }
        fn apply_and_commit(&self, _: &Path, patch: &str, _: &str) -> Result<(), GitError> {
            self.committed.lock().unwrap().push(patch.into());
            Ok(())
        }
    }
    struct FakeHooks;
    impl HookInstaller for FakeHooks {
        fn install(&self, _: &Path) -> Result<(), GitError> {
            Ok(())
        }
        fn uninstall(&self, _: &Path) -> Result<(), GitError> {
            Ok(())
        }
    }

    /// Menjawab review dengan `review_json`, dan permintaan patch dengan `PATCH`.
    struct FakeAi {
        review_json: String,
        prompts: Arc<Mutex<Vec<String>>>,
    }
    const PATCH: &str = "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n";
    impl AiClient for FakeAi {
        fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
            self.prompts.lock().unwrap().push(user.into());
            Ok(if system == PATCH_SYSTEM {
                PATCH.into()
            } else {
                self.review_json.clone()
            })
        }
    }
    struct FakeFactory(Arc<FakeAi>);
    impl AiProviderFactory for FakeFactory {
        fn build(
            &self,
            _: crate::domain::model::settings::AiProvider,
            _: &str,
        ) -> Result<Arc<dyn AiClient>, AiError> {
            Ok(self.0.clone())
        }
    }

    const ONE_FINDING: &str = r#"{"summary":"ada bug","findings":[{"file":"x","line":1,"severity":"important","title":"t","detail":"d"}]}"#;

    struct Fixture {
        svc: Arc<ReviewService>,
        island: Arc<IslandService>,
        git: Arc<FakeGit>,
        prompts: Arc<Mutex<Vec<String>>>,
        _rt: tokio::runtime::Runtime,
    }

    fn fixture(
        diff: &str,
        review_json: &str,
        timeout_action: ReviewTimeoutAction,
        timeout_ms: u64,
    ) -> Fixture {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let island = IslandService::new(Arc::new(NullSink), rt.handle().clone());
        let initial = Settings {
            watched_repos: vec!["/repo".into()],
            review_timeout_action: timeout_action,
            ..Default::default()
        };
        let settings = SettingsService::new(
            Arc::new(MemRepo(Mutex::new(initial))),
            Arc::new(NoSecrets),
            Arc::new(NoAutostart),
            Arc::new(NullSink),
            island.clone(),
        );
        let git = Arc::new(FakeGit {
            diff: diff.into(),
            committed: Mutex::new(vec![]),
        });
        let prompts = Arc::new(Mutex::new(vec![]));
        let ai = Arc::new(FakeAi {
            review_json: review_json.into(),
            prompts: prompts.clone(),
        });
        let svc = ReviewService::new(
            git.clone(),
            Arc::new(FakeHooks),
            Arc::new(RegexScanner),
            Arc::new(FakeFactory(ai)),
            settings,
            island.clone(),
            Duration::from_millis(timeout_ms),
        );
        Fixture {
            svc,
            island,
            git,
            prompts,
            _rt: rt,
        }
    }

    fn push() -> Vec<PushRef> {
        vec![PushRef::parse("refs/heads/m aaa refs/heads/m bbb").unwrap()]
    }

    fn showing_result(f: &Fixture) -> bool {
        matches!(
            f.island.snapshot().view,
            View::Expanded(Content::ReviewResult(_))
        )
    }

    /// Jalankan review di thread lain; `script` menjawab kartu setelah hasil tampil.
    fn run_with(f: &Fixture, script: Vec<Decision>) -> Verdict {
        let svc = f.svc.clone();
        let worker = thread::spawn(move || svc.review_push("/repo", &push()));
        for d in script {
            while !showing_result(f) {
                thread::sleep(Duration::from_millis(5));
            }
            thread::sleep(Duration::from_millis(30));
            f.svc.decide(d);
            thread::sleep(Duration::from_millis(100));
        }
        worker.join().unwrap()
    }

    const DIFF: &str = "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n";

    #[test]
    fn repo_tak_diawasi_atau_diff_kosong_langsung_lolos_tanpa_ai() {
        let f = fixture(DIFF, ONE_FINDING, ReviewTimeoutAction::Continue, 50);
        assert_eq!(f.svc.review_push("/lain", &push()), Verdict::Allow);
        let f2 = fixture("  \n", ONE_FINDING, ReviewTimeoutAction::Continue, 50);
        assert_eq!(f2.svc.review_push("/repo", &push()), Verdict::Allow);
        assert!(f.prompts.lock().unwrap().is_empty() && f2.prompts.lock().unwrap().is_empty());
    }

    #[test]
    fn tanpa_temuan_lolos_tanpa_menunggu_pengguna() {
        let f = fixture(
            DIFF,
            r#"{"summary":"bersih","findings":[]}"#,
            ReviewTimeoutAction::Cancel,
            60_000,
        );
        assert_eq!(f.svc.review_push("/repo", &push()), Verdict::Allow);
    }

    #[test]
    fn lanjut_dan_batal_menentukan_nasib_push() {
        let f = fixture(DIFF, ONE_FINDING, ReviewTimeoutAction::Cancel, 60_000);
        assert_eq!(run_with(&f, vec![Decision::Continue]), Verdict::Allow);
        assert_eq!(
            f.island.snapshot().view,
            View::Idle,
            "kartu ditutup setelah keputusan"
        );
        let f = fixture(DIFF, ONE_FINDING, ReviewTimeoutAction::Continue, 60_000);
        assert!(matches!(
            run_with(&f, vec![Decision::Cancel]),
            Verdict::Block(_)
        ));
    }

    #[test]
    fn perbaiki_lalu_setujui_commit_dan_menahan_push() {
        let f = fixture(DIFF, ONE_FINDING, ReviewTimeoutAction::Cancel, 60_000);
        let v = run_with(&f, vec![Decision::Fix, Decision::ApplyPatch]);
        assert!(matches!(v, Verdict::Block(m) if m.contains("push lagi")));
        assert_eq!(f.git.committed.lock().unwrap().as_slice(), [PATCH]);
    }

    #[test]
    fn patch_ditolak_tidak_di_commit_dan_kartu_kembali_biasa() {
        let f = fixture(DIFF, ONE_FINDING, ReviewTimeoutAction::Cancel, 60_000);
        let v = run_with(
            &f,
            vec![Decision::Fix, Decision::RejectPatch, Decision::Continue],
        );
        assert_eq!(v, Verdict::Allow);
        assert!(f.git.committed.lock().unwrap().is_empty());
    }

    #[test]
    fn waktu_habis_mengikuti_setelan() {
        let f = fixture(DIFF, ONE_FINDING, ReviewTimeoutAction::Cancel, 60);
        assert!(matches!(
            f.svc.review_push("/repo", &push()),
            Verdict::Block(_)
        ));
        let f = fixture(DIFF, ONE_FINDING, ReviewTimeoutAction::Continue, 60);
        assert_eq!(f.svc.review_push("/repo", &push()), Verdict::Allow);
    }

    #[test]
    fn rahasia_tidak_dikirim_ke_ai_dan_menjadi_temuan_kritis() {
        let diff = "--- a/c.env\n+++ b/c.env\n@@ -0,0 +1 @@\n+k=ghp_abcdefghijklmnopqrstuvwxyz0123456789\n";
        let f = fixture(
            diff,
            r#"{"summary":"s","findings":[]}"#,
            ReviewTimeoutAction::Cancel,
            60,
        );
        assert!(matches!(
            f.svc.review_push("/repo", &push()),
            Verdict::Block(_)
        ));
        let sent = f.prompts.lock().unwrap().join("\n");
        assert!(!sent.contains("ghp_abc") && sent.contains("[REDACTED]"));
        let report = f.svc.review("p", &RegexScanner.scan(diff));
        assert_eq!(report.findings[0].severity, Severity::Critical);
        assert!(!format!("{report:?}").contains("ghp_abc"));
    }

    #[test]
    fn watch_memasang_hook_dan_mencatat_repo_tanpa_duplikat() {
        let f = fixture(DIFF, ONE_FINDING, ReviewTimeoutAction::Continue, 50);
        f.svc.watch("/baru").unwrap();
        f.svc.watch("/BARU/").unwrap();
        assert_eq!(f.svc.settings.get().watched_repos, vec!["/repo", "/baru"]);
        f.svc.unwatch("/baru").unwrap();
        assert_eq!(f.svc.settings.get().watched_repos, vec!["/repo"]);
    }
}
