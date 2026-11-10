//! macOS: tangkap tombol volume (keyboard dan tombol media) sebelum sistem, supaya HUD sistem tidak
//! muncul bersamaan dengan HUD pulau. Memakai `CGEventTap` aktif: tombol yang ditangani dibuang dari
//! aliran event, sisanya (dan semua tombol bila Isla mati) tetap sampai ke sistem. Umur tangkapan
//! terikat proses Isla, jadi tidak ada perubahan permanen pada sistem bila Isla crash.
//!
//! Butuh izin Accessibility; tanpanya `CGEventTapCreate` gagal dan tombol dibiarkan ke sistem.
//!
//! Batas: hanya tombol media fisik (keyboard biasa/eksternal, tombol headset). Strip kontrol Touch Bar
//! mengubah volume langsung ke sistem audio tanpa event tombol, jadi tidak bisa ditangkap di sini.
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use objc2_app_kit::NSEvent;
use objc2_core_foundation::{CFMachPort, CFRunLoop, kCFRunLoopCommonModes};
use objc2_core_graphics::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType,
};

use crate::domain::model::VolumeKey;

/// NX_SYSDEFINED: event tombol khusus (volume, kecerahan, media).
const SYSTEM_DEFINED: u32 = 14;
/// NX_SUBTYPE_AUX_CONTROL_BUTTONS.
const AUX_CONTROL_BUTTONS: i16 = 8;

/// Mengurai `data1` event tombol khusus: (tombol volume, sedang ditekan?). Tombol lain -> `None`.
/// `data1` = kode tombol di 16 bit atas, status di byte kedua (0xA = tekan, 0xB = lepas).
pub(crate) fn decode(data1: i64) -> Option<(VolumeKey, bool)> {
    let code = (data1 >> 16) & 0xFFFF;
    let state = (data1 >> 8) & 0xFF;
    let key = match code {
        0 => VolumeKey::Up,   // NX_KEYTYPE_SOUND_UP
        1 => VolumeKey::Down, // NX_KEYTYPE_SOUND_DOWN
        7 => VolumeKey::Mute, // NX_KEYTYPE_MUTE
        _ => return None,
    };
    Some((key, state == 0xA))
}

type Handler = Arc<dyn Fn(VolumeKey) -> bool + Send + Sync>;

/// Dipakai callback C: apakah tombol yang sedang ditekan tadi kita tangani (untuk pasangannya saat
/// dilepas, supaya tombol tekan dan lepas tidak terpisah).
static CONSUMING: AtomicBool = AtomicBool::new(false);

unsafe extern "C-unwind" fn callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: NonNull<CGEvent>,
    user_info: *mut c_void,
) -> *mut CGEvent {
    if event_type.0 != SYSTEM_DEFINED {
        return event.as_ptr(); // termasuk TapDisabled*: diteruskan apa adanya
    }
    // SAFETY: `user_info` adalah `Box<Handler>` yang dibocorkan di `start` dan tak pernah dibebaskan.
    let handler = unsafe { &*user_info.cast::<Handler>() };
    // SAFETY: `event` valid selama callback berjalan.
    let cg = unsafe { event.as_ref() };
    let Some(ns) = NSEvent::eventWithCGEvent(cg) else {
        return event.as_ptr();
    };
    if ns.subtype().0 != AUX_CONTROL_BUTTONS {
        return event.as_ptr();
    }
    let Some((key, down)) = decode(ns.data1() as i64) else {
        return event.as_ptr();
    };
    // Kombinasi dengan Shift/Option/Command/Control punya arti lain di sistem (volume halus,
    // Pengaturan Suara), jadi dibiarkan.
    let modifiers = CGEventFlags::MaskShift
        | CGEventFlags::MaskAlternate
        | CGEventFlags::MaskCommand
        | CGEventFlags::MaskControl;
    if CGEvent::flags(Some(cg)).intersects(modifiers) {
        return event.as_ptr();
    }
    let consumed = if down {
        let handled = handler(key);
        CONSUMING.store(handled, Ordering::SeqCst);
        handled
    } else {
        CONSUMING.load(Ordering::SeqCst)
    };
    if consumed {
        std::ptr::null_mut() // buang event: sistem tak melihat tombolnya
    } else {
        event.as_ptr()
    }
}

/// Mulai menangkap tombol volume di thread sendiri (run loop). `handler` mengembalikan `true` bila
/// tombol sudah ditangani. Mengembalikan galat bila tangkapan tak bisa dibuat (biasanya izin
/// Accessibility belum diberikan).
pub fn start(handler: Handler) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    std::thread::Builder::new()
        .name("volume-keys".into())
        .spawn(move || {
            let user_info = Box::into_raw(Box::new(handler)).cast::<c_void>();
            // SAFETY: `callback` cocok dengan CGEventTapCallBack dan `user_info` hidup selamanya.
            let port = unsafe {
                CGEvent::tap_create(
                    CGEventTapLocation::SessionEventTap,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::Default,
                    1u64 << SYSTEM_DEFINED,
                    Some(callback),
                    user_info,
                )
            };
            let Some(port) = port else {
                let _ = tx.send(Err("izin Accessibility belum diberikan".into()));
                return;
            };
            let source = CFMachPort::new_run_loop_source(None, Some(&port), 0);
            let run_loop = CFRunLoop::current();
            let (Some(source), Some(run_loop)) = (source, run_loop) else {
                let _ = tx.send(Err("run loop tidak tersedia".into()));
                return;
            };
            // SAFETY: `kCFRunLoopCommonModes` adalah konstanta global CoreFoundation.
            run_loop.add_source(Some(&source), unsafe { kCFRunLoopCommonModes });
            CGEvent::tap_enable(&port, true);
            let _ = tx.send(Ok(()));
            CFRunLoop::run();
        })
        .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data1_tombol_volume_diurai_tekan_dan_lepas() {
        // kode tombol <<16 | status <<8: volume naik ditekan (0xA) dan dilepas (0xB)
        assert_eq!(decode(0x0000_0A00), Some((VolumeKey::Up, true)));
        assert_eq!(decode(0x0000_0B00), Some((VolumeKey::Up, false)));
        assert_eq!(decode(0x0001_0A00), Some((VolumeKey::Down, true)));
        assert_eq!(decode(0x0007_0A00), Some((VolumeKey::Mute, true)));
        // tombol berulang (bit 0 = repeat) tetap dianggap ditekan
        assert_eq!(decode(0x0000_0A01), Some((VolumeKey::Up, true)));
    }

    #[test]
    fn tombol_khusus_lain_diabaikan() {
        // kecerahan naik/turun (2, 3), putar/jeda (16), dan lainnya
        for code in [2i64, 3, 16, 17, 18, 20, 255] {
            assert_eq!(decode(code << 16 | 0x0A00), None, "kode {code}");
        }
    }
}
