use crate::domain::ports::SoundPlayer;

/// Nada notifikasi bawaan (WAV mono 16-bit) yang ikut terkompilasi ke aplikasi.
#[cfg(windows)]
static CHIME: &[u8] = include_bytes!("../../../assets/chime.wav");

/// Memutar nada lewat PlaySound Windows dari memori, tanpa menunggu selesai. Platform lain: diam.
pub struct SystemSound;

impl SoundPlayer for SystemSound {
    fn chime(&self) {
        #[cfg(windows)]
        {
            use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY};
            use windows::core::PCWSTR;
            // SAFETY: SND_MEMORY membaca WAV dari penunjuk; `CHIME` bersifat 'static sehingga tetap
            // valid selama pemutaran asinkron berjalan.
            let _ =
                unsafe { PlaySoundW(PCWSTR(CHIME.as_ptr().cast()), None, SND_MEMORY | SND_ASYNC) };
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[test]
    fn berkas_nada_adalah_wav_mono_16_bit_yang_wajar() {
        let b = super::CHIME;
        assert_eq!(&b[0..4], b"RIFF");
        assert_eq!(&b[8..12], b"WAVE");
        assert_eq!(u16::from_le_bytes([b[22], b[23]]), 1, "mono");
        assert_eq!(u16::from_le_bytes([b[34], b[35]]), 16, "16 bit");
        assert!(b.len() > 20_000 && b.len() < 400_000);
    }
}
