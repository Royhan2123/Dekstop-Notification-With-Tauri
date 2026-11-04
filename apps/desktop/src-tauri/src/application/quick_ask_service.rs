use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use crate::application::island_service::IslandService;
use crate::application::settings_service::SettingsService;
use crate::domain::error::AiError;
use crate::domain::island::action::IslandAction;
use crate::domain::island::state::AskPhase;
use crate::domain::model::settings::Language;
use crate::domain::ports::AiProviderFactory;

const MAX_QUESTION_CHARS: usize = 1000;
const MAX_ANSWER_CHARS: usize = 4000;

/// Quick Ask: satu pertanyaan ke otak AI yang dipilih di setelan, jawabannya tampil di pulau.
pub struct QuickAskService {
    ai: Arc<dyn AiProviderFactory>,
    settings: Arc<SettingsService>,
    island: Arc<IslandService>,
    /// Naik tiap pertanyaan baru atau pulau dibuka ulang; jawaban yang terlambat jadi tidak berlaku.
    generation: AtomicU64,
}

impl QuickAskService {
    pub fn new(
        ai: Arc<dyn AiProviderFactory>,
        settings: Arc<SettingsService>,
        island: Arc<IslandService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            ai,
            settings,
            island,
            generation: AtomicU64::new(0),
        })
    }

    /// Tampilkan kotak pertanyaan di pulau.
    pub fn open(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.island.dispatch(IslandAction::AskOpen);
    }

    /// Kirim pertanyaan; jawabannya datang belakangan lewat pulau. Pertanyaan kosong diabaikan.
    pub fn submit(self: &Arc<Self>, question: &str) {
        let q: String = question.trim().chars().take(MAX_QUESTION_CHARS).collect();
        if q.is_empty() {
            return;
        }
        let generation = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        self.island
            .dispatch(IslandAction::AskUpdate(AskPhase::Thinking));
        let this = Arc::clone(self);
        thread::spawn(move || {
            let phase = this.answer(&q);
            this.finish(generation, phase);
        });
    }

    /// Uji otak AI yang dipilih dengan satu pertanyaan pendek; dipakai Setelan untuk memastikan
    /// kunci dan model benar begitu kunci ditempel. Mengembalikan balasan singkatnya.
    pub fn test_connection(&self) -> Result<String, AiError> {
        let settings = self.settings.get();
        let reply = self
            .ai
            .build(settings.ai_provider, &settings.ai_model)?
            .complete(
                "Balas hanya dengan satu kata: siap. Gunakan bahasa Indonesia.",
                "Tes koneksi",
            )?;
        Ok(reply.trim().chars().take(60).collect())
    }

    /// Model yang ditawarkan penyedia terpilih (butuh kunci yang sudah tersimpan), untuk menu model di Setelan.
    pub fn models(&self) -> Result<Vec<String>, AiError> {
        let settings = self.settings.get();
        self.ai
            .build(settings.ai_provider, &settings.ai_model)?
            .models()
    }

    fn answer(&self, question: &str) -> AskPhase {
        let settings = self.settings.get();
        let system = match settings.language {
            Language::Id => {
                "Kamu asisten singkat di pulau desktop. Jawab dalam bahasa Indonesia, \
                maksimal empat kalimat, teks polos tanpa markdown berat."
            }
            Language::En => {
                "You are a concise assistant in a desktop island. Answer in English, \
                at most four sentences, plain text without heavy markdown."
            }
        };
        let result = self
            .ai
            .build(settings.ai_provider, &settings.ai_model)
            .and_then(|client| client.complete(system, question));
        match result {
            Ok(text) => AskPhase::Done(text.trim().chars().take(MAX_ANSWER_CHARS).collect()),
            Err(e) => AskPhase::Failed(e.to_string()),
        }
    }

    fn finish(&self, generation: u64, phase: AskPhase) {
        if self.generation.load(Ordering::SeqCst) == generation {
            self.island.dispatch(IslandAction::AskUpdate(phase));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::island::state::{Content, View};
    use crate::domain::model::settings::{AiProvider, Settings};
    use crate::domain::ports::AiClient;

    struct Echo(Result<String, String>);
    impl AiClient for Echo {
        fn complete(&self, system: &str, user: &str) -> Result<String, AiError> {
            assert!(
                system.contains("Indonesia"),
                "prompt mengikuti bahasa setelan"
            );
            self.0
                .clone()
                .map(|t| format!("{t}: {user}"))
                .map_err(AiError::Unavailable)
        }
    }
    struct Factory(Result<String, String>);
    impl AiProviderFactory for Factory {
        fn build(&self, _: AiProvider, _: &str) -> Result<Arc<dyn AiClient>, AiError> {
            Ok(Arc::new(Echo(self.0.clone())))
        }
    }

    fn fx(
        reply: Result<String, String>,
    ) -> (
        Arc<QuickAskService>,
        Arc<IslandService>,
        tokio::runtime::Runtime,
    ) {
        let (settings, island, rt) = services(Settings::default());
        (
            QuickAskService::new(Arc::new(Factory(reply)), settings, island.clone()),
            island,
            rt,
        )
    }

    fn phase(island: &IslandService) -> Option<AskPhase> {
        match island.snapshot().view {
            View::Expanded(Content::Ask(p)) => Some(p),
            _ => None,
        }
    }

    #[test]
    fn jawaban_dan_galat_menjadi_fase_yang_sesuai() {
        let (svc, _island, _rt) = fx(Ok("halo".into()));
        assert_eq!(svc.answer("hai"), AskPhase::Done("halo: hai".into()));
        let (svc, _island, _rt) = fx(Err("mati".into()));
        assert!(matches!(svc.answer("hai"), AskPhase::Failed(m) if m.contains("mati")));
    }

    #[test]
    fn uji_koneksi_mengembalikan_balasan_atau_galat_penyedia() {
        let (svc, _island, _rt) = fx(Ok("siap".into()));
        assert_eq!(svc.test_connection().unwrap(), "siap: Tes koneksi");
        let (svc, _island, _rt) = fx(Err("kunci salah".into()));
        assert!(
            svc.test_connection()
                .unwrap_err()
                .to_string()
                .contains("kunci salah")
        );
    }

    #[test]
    fn jawaban_usang_dibuang_dan_pertanyaan_kosong_diabaikan() {
        let (svc, island, _rt) = fx(Ok("x".into()));
        svc.open();
        assert_eq!(phase(&island), Some(AskPhase::Input));
        svc.submit("   ");
        assert_eq!(phase(&island), Some(AskPhase::Input));

        svc.finish(99, AskPhase::Done("usang".into()));
        assert_eq!(phase(&island), Some(AskPhase::Input));
        let current = svc.generation.load(Ordering::SeqCst);
        svc.finish(current, AskPhase::Done("baru".into()));
        assert_eq!(phase(&island), Some(AskPhase::Done("baru".into())));

        // Pulau dibuka ulang: jawaban dari pertanyaan sebelumnya tidak boleh muncul.
        svc.open();
        svc.finish(current, AskPhase::Done("terlambat".into()));
        assert_eq!(phase(&island), Some(AskPhase::Input));
    }
}
