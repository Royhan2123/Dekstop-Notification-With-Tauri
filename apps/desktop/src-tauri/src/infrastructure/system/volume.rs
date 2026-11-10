use crate::domain::island::state::VolumeInfo;
use crate::domain::ports::VolumeReader;

/// Volume utama perangkat keluaran bawaan. Platform tanpa dukungan melapor `None`.
pub struct SystemVolume;

/// macOS: CoreAudio langsung. AppleScript (`osascript`) butuh ±0,5 dtk per panggilan, padahal volume
/// dibaca 4 kali per detik.
#[cfg(target_os = "macos")]
mod coreaudio {
    use std::ffi::c_void;

    #[repr(C)]
    struct Address {
        selector: u32,
        scope: u32,
        element: u32,
    }

    #[link(name = "CoreAudio", kind = "framework")]
    unsafe extern "C" {
        fn AudioObjectGetPropertyData(
            object: u32,
            address: *const Address,
            qualifier_size: u32,
            qualifier: *const c_void,
            data_size: *mut u32,
            data: *mut c_void,
        ) -> i32;
    }

    const fn four(code: &[u8; 4]) -> u32 {
        u32::from_be_bytes(*code)
    }

    /// Baca satu properti bertipe `T` (angka sederhana); `None` bila perangkat tak punya properti itu.
    fn get<T: Default + Copy>(object: u32, selector: u32, scope: u32) -> Option<T> {
        let address = Address {
            selector,
            scope,
            element: 0, // kAudioObjectPropertyElementMain
        };
        let mut value = T::default();
        let mut size = std::mem::size_of::<T>() as u32;
        // SAFETY: `address` dan `value` hidup selama pemanggilan; `size` memberi tahu CoreAudio
        // ukuran buffer `value` (sizeof T), jadi tulisan tidak melewati batas. Tanpa qualifier.
        let status = unsafe {
            AudioObjectGetPropertyData(
                object,
                &address,
                0,
                std::ptr::null(),
                &mut size,
                (&mut value as *mut T).cast(),
            )
        };
        (status == 0).then_some(value)
    }

    /// (volume 0.0-1.0, bisu) perangkat keluaran bawaan; `None` bila tak ada atau tak punya volume
    /// (mis. HDMI).
    pub fn read() -> Option<(f32, bool)> {
        // kAudioObjectSystemObject (1) -> kAudioHardwarePropertyDefaultOutputDevice 'dOut'
        let device: u32 = get(1, four(b"dOut"), four(b"glob"))?;
        if device == 0 {
            return None;
        }
        // kAudioHardwareServiceDeviceProperty_VirtualMainVolume 'vmvc', scope output 'outp'
        let level: f32 = get(device, four(b"vmvc"), four(b"outp"))?;
        let muted: u32 = get(device, four(b"mute"), four(b"outp")).unwrap_or(0);
        Some((level, muted != 0))
    }
}
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
        #[cfg(target_os = "macos")]
        {
            let (level, muted) = coreaudio::read()?;
            Some(VolumeInfo {
                level: (level.clamp(0.0, 1.0) * 100.0).round() as u8,
                muted,
            })
        }
        #[cfg(not(any(windows, target_os = "macos")))]
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
