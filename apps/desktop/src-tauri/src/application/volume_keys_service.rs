// Penangkap tombolnya baru ada di macOS; di OS lain layanan ini belum dipakai.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::sync::Arc;

use crate::application::settings_service::SettingsService;
use crate::application::volume_service::VolumeService;
use crate::domain::model::VolumeKey;
use crate::domain::ports::{VolumeControl, VolumeReader};

/// Langkah per ketukan tombol; macOS memakai 16 langkah (6,25 persen).
const STEP: u8 = 6;

/// Mengambil alih tombol volume: Isla mengubah volume sendiri lalu menampilkan HUD-nya di pulau,
/// sementara tombolnya tidak diteruskan ke sistem (jadi HUD sistem tidak ikut muncul).
pub struct VolumeKeysService {
    reader: Arc<dyn VolumeReader>,
    control: Arc<dyn VolumeControl>,
    settings: Arc<SettingsService>,
    hud: Arc<VolumeService>,
}

impl VolumeKeysService {
    pub fn new(
        reader: Arc<dyn VolumeReader>,
        control: Arc<dyn VolumeControl>,
        settings: Arc<SettingsService>,
        hud: Arc<VolumeService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            reader,
            control,
            settings,
            hud,
        })
    }

    /// `true` = tombol sudah ditangani dan tidak boleh diteruskan ke sistem. `false` (setelan mati,
    /// volume tak terbaca, atau perangkat menolak diatur) membiarkan sistem menanganinya seperti biasa,
    /// supaya tombol volume tidak pernah mati.
    pub fn handle(&self, key: VolumeKey) -> bool {
        if !self.settings.get().take_over_volume_keys {
            return false;
        }
        let Some(now) = self.reader.read() else {
            return false;
        };
        let done = match key {
            VolumeKey::Up => {
                // Menaikkan volume saat bisu membuka bisu dulu, seperti perilaku sistem.
                (!now.muted || self.control.set_muted(false))
                    && self
                        .control
                        .set_level(now.level.saturating_add(STEP).min(100))
            }
            VolumeKey::Down => self.control.set_level(now.level.saturating_sub(STEP)),
            VolumeKey::Mute => self.control.set_muted(!now.muted),
        };
        if done {
            self.hud.tick(); // HUD langsung muncul, tanpa menunggu pembacaan berkala
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::services;
    use crate::domain::island::state::{Content, View, VolumeInfo};
    use crate::domain::model::settings::Settings;
    use std::sync::Mutex;

    /// Perangkat suara palsu yang bisa dibaca dan diatur.
    struct Device {
        state: Mutex<VolumeInfo>,
        accepts: bool,
    }
    impl VolumeReader for Device {
        fn read(&self) -> Option<VolumeInfo> {
            Some(*self.state.lock().unwrap())
        }
    }
    impl VolumeControl for Device {
        fn set_level(&self, level: u8) -> bool {
            self.accepts && {
                self.state.lock().unwrap().level = level;
                true
            }
        }
        fn set_muted(&self, muted: bool) -> bool {
            self.accepts && {
                self.state.lock().unwrap().muted = muted;
                true
            }
        }
    }

    struct Fx {
        keys: Arc<VolumeKeysService>,
        device: Arc<Device>,
        island: Arc<crate::application::island_service::IslandService>,
        _rt: tokio::runtime::Runtime,
    }

    fn fx(enabled: bool, level: u8, muted: bool, accepts: bool) -> Fx {
        let (settings, island, rt) = services(Settings {
            take_over_volume_keys: enabled,
            ..Default::default()
        });
        let device = Arc::new(Device {
            state: Mutex::new(VolumeInfo { level, muted }),
            accepts,
        });
        let hud = VolumeService::new(device.clone(), island.clone());
        hud.tick(); // acuan awal: tanpa ini perubahan pertama tidak memunculkan HUD
        let keys = VolumeKeysService::new(device.clone(), device.clone(), settings, hud);
        Fx {
            keys,
            device,
            island,
            _rt: rt,
        }
    }

    #[test]
    fn naik_dan_turun_per_langkah_dengan_batas_0_dan_100() {
        let f = fx(true, 50, false, true);
        assert!(f.keys.handle(VolumeKey::Up));
        assert_eq!(f.device.read().unwrap().level, 56);
        assert!(f.keys.handle(VolumeKey::Down));
        assert!(f.keys.handle(VolumeKey::Down));
        assert_eq!(f.device.read().unwrap().level, 44);
        let top = fx(true, 98, false, true);
        top.keys.handle(VolumeKey::Up);
        assert_eq!(top.device.read().unwrap().level, 100);
        let low = fx(true, 3, false, true);
        low.keys.handle(VolumeKey::Down);
        assert_eq!(low.device.read().unwrap().level, 0);
    }

    #[test]
    fn naik_saat_bisu_membuka_bisu_dan_tombol_bisu_membalik_status() {
        let f = fx(true, 30, true, true);
        assert!(f.keys.handle(VolumeKey::Up));
        let now = f.device.read().unwrap();
        assert_eq!((now.level, now.muted), (36, false));
        assert!(f.keys.handle(VolumeKey::Mute));
        assert!(f.device.read().unwrap().muted);
        assert!(f.keys.handle(VolumeKey::Mute));
        assert!(!f.device.read().unwrap().muted);
    }

    #[test]
    fn hud_pulau_muncul_langsung_setelah_tombol() {
        let f = fx(true, 50, false, true);
        assert!(matches!(f.island.snapshot().view, View::Idle));
        f.keys.handle(VolumeKey::Up);
        let snapshot = f.island.snapshot();
        assert!(matches!(snapshot.view.content(), Some(Content::Volume(v)) if v.level == 56));
    }

    #[test]
    fn setelan_mati_atau_perangkat_menolak_meneruskan_tombol_ke_sistem() {
        let off = fx(false, 50, false, true);
        assert!(
            !off.keys.handle(VolumeKey::Up),
            "setelan mati: sistem yang menangani"
        );
        assert_eq!(off.device.read().unwrap().level, 50);
        let refuse = fx(true, 50, false, false);
        assert!(
            !refuse.keys.handle(VolumeKey::Down),
            "tombol tak boleh mati bila gagal diatur"
        );
    }
}
