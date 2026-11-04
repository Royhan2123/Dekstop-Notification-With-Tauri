//! Riwayat Tanya AI: pasangan pertanyaan dan jawaban yang tersimpan antar sesi.

/// Riwayat disimpan paling banyak sejumlah ini; yang terlama dibuang lebih dulu.
pub const MAX_CHAT_TURNS: usize = 50;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatTurn {
    pub question: String,
    pub answer: String,
    /// AI yang menjawab, mis. "Claude · claude-sonnet-5-5".
    pub by: String,
}
