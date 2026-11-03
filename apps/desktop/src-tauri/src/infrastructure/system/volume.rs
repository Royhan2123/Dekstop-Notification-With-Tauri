use crate::domain::island::state::VolumeInfo;
use crate::domain::ports::VolumeReader;

/// Volume utama perangkat keluaran bawaan. Platform tanpa dukungan melapor `None`.
pub struct SystemVolume;

impl VolumeReader for SystemVolume {
    fn read(&self) -> Option<VolumeInfo> {
        #[cfg(windows)]
        {
            use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
            use windows::Win32::Media::Audio::{
                IMMDeviceEnumerator, MMDeviceEnumerator, eConsole, eRender,
            };
            use windows::Win32::System::Com::{
                CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
            };
            // SAFETY: pemanggilan COM standar; pointer yang dikembalikan dikelola wrapper `windows`.
            // CoInitializeEx boleh dipanggil berulang di thread yang sama (hasilnya diabaikan).
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                let enumerator: IMMDeviceEnumerator =
                    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
                let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
                let volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).ok()?;
                let level = volume.GetMasterVolumeLevelScalar().ok()?;
                Some(VolumeInfo {
                    level: (level.clamp(0.0, 1.0) * 100.0).round() as u8,
                    muted: volume.GetMute().ok()?.as_bool(),
                })
            }
        }
        #[cfg(not(windows))]
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membaca_volume_tidak_crash_dan_dalam_rentang() {
        if let Some(v) = SystemVolume.read() {
            assert!(v.level <= 100);
        }
    }
}
