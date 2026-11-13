use std::cmp::Reverse;

use super::action::{Effect, IslandAction};
use super::state::{AskPhase, Content, Ctx, IslandState, SHELF_MAX, View, VoicePhase};

pub fn reduce(state: &IslandState, action: IslandAction, ctx: &Ctx) -> (IslandState, Vec<Effect>) {
    let mut s = state.clone();
    let mut fx = Vec::new();
    match action {
        IslandAction::MediaChanged(media) => {
            s.media = media;
            let showing_media = s.view.content() == Some(&Content::Media);
            if s.view == View::Idle || (showing_media && s.media.is_none()) {
                settle(&mut s, ctx, &mut fx);
            }
        }
        IslandAction::ReviewStarted(id) => offer(&mut s, Content::ReviewRunning(id), ctx, &mut fx),
        IslandAction::ReviewFinished(id, report) => {
            s.queue.retain(|c| *c != Content::ReviewRunning(id));
            let result = Content::ReviewResult(report);
            // Hasil yang sedang tampil diperbarui di tempat (mis. setelah patch siap).
            if matches!(s.view.content(), Some(Content::ReviewResult(_)))
                || s.view.content() == Some(&Content::ReviewRunning(id))
            {
                show(&mut s, result, ctx, &mut fx);
            } else {
                offer(&mut s, result, ctx, &mut fx);
            }
        }
        IslandAction::HealthAlert(a) => offer(&mut s, Content::HealthAlert(a), ctx, &mut fx),
        IslandAction::Notice(text) => offer(&mut s, Content::Notice(text), ctx, &mut fx),
        IslandAction::UserNotice(text) => {
            offer_as(&mut s, Content::Notice(text), ctx, &mut fx, false)
        }
        IslandAction::Hover(h) => {
            s.hovered = h;
            if s.view == View::Expanded(Content::Media) {
                s.token += 1;
                if !h {
                    fx.push(Effect::StartTimer {
                        token: s.token,
                        ms: ctx.collapse_ms,
                    });
                }
            }
        }
        IslandAction::Clicked => {
            if s.view == View::Compact(Content::Media) {
                s.token += 1;
                s.view = View::Expanded(Content::Media);
            }
        }
        IslandAction::MediaShow => {
            if s.media.is_some() {
                s.token += 1;
                s.view = View::Expanded(Content::Media);
            }
        }
        IslandAction::Dismissed => match s.view {
            View::Idle | View::Compact(Content::Media) => {}
            View::Expanded(Content::Media) => s.view = View::Compact(Content::Media),
            _ => settle(&mut s, ctx, &mut fx),
        },
        IslandAction::Timeout(token) => {
            if token == s.token {
                match s.view {
                    View::Expanded(Content::Media) => s.view = View::Compact(Content::Media),
                    View::Compact(Content::Volume(_) | Content::Brightness(_)) => {
                        settle(&mut s, ctx, &mut fx)
                    }
                    View::Expanded(ref c) if is_soft(c) || matches!(c, Content::Approval(_)) => {
                        settle(&mut s, ctx, &mut fx)
                    }
                    _ => {}
                }
            }
        }
        IslandAction::HideForFullscreen(b) => s.fullscreen = b,
        IslandAction::ExtensionConnected(b) => s.extension = b,
        IslandAction::VolumeChanged(v) => show_hud(&mut s, Content::Volume(v), ctx, &mut fx),
        IslandAction::BrightnessChanged(b) => {
            show_hud(&mut s, Content::Brightness(b), ctx, &mut fx)
        }
        IslandAction::AskOpen => open_card(&mut s, Content::Ask(AskPhase::Input), ctx, &mut fx),
        IslandAction::AskUpdate(phase) => {
            if matches!(s.view.content(), Some(Content::Ask(_))) {
                s.view = View::Expanded(Content::Ask(phase));
            }
        }
        IslandAction::ShelfAdd(paths) => {
            for p in paths {
                s.shelf.retain(|x| *x != p);
                s.shelf.push(p);
            }
            let extra = s.shelf.len().saturating_sub(SHELF_MAX);
            s.shelf.drain(..extra);
            show_shelf(&mut s, ctx, &mut fx);
        }
        IslandAction::ShelfShow => open_card(&mut s, Content::Shelf, ctx, &mut fx),
        IslandAction::ShelfRemove(p) => s.shelf.retain(|x| *x != p),
        IslandAction::ShelfClear => s.shelf.clear(),
        IslandAction::ApprovalRequested(a) => offer(&mut s, Content::Approval(a), ctx, &mut fx),
        IslandAction::VoiceChanged(phase) => voice_changed(&mut s, phase, ctx, &mut fx),
        IslandAction::DoNotDisturb(b) => {
            s.dnd = b;
            if b {
                // Hanya antrean yang dibuang. Kartu yang sedang dilihat (tempat pengguna baru saja
                // menekan tombol bulan) dibiarkan habis sendiri; menutupnya seketika terasa seperti hang.
                s.queue.retain(|c| !is_soft(c));
            }
        }
    }
    (s, fx)
}

/// Jarvis berganti fase. Indikator hanya menumpang di pulau kosong atau media/HUD ringkas (seperti
/// HUD), tetapi fasenya disimpan supaya muncul begitu kartu yang lebih penting selesai.
fn voice_changed(s: &mut IslandState, phase: Option<VoicePhase>, ctx: &Ctx, fx: &mut Vec<Effect>) {
    s.voice = phase;
    let free = matches!(
        s.view,
        View::Idle
            | View::Compact(
                Content::Media | Content::Volume(_) | Content::Brightness(_) | Content::Voice(_)
            )
    );
    match phase {
        Some(p) if free => show(s, Content::Voice(p), ctx, fx),
        None if matches!(s.view, View::Compact(Content::Voice(_))) => settle(s, ctx, fx),
        _ => {}
    }
}

/// Kartu yang dibuka sengaja lewat dok atau pintasan (Tanya AI, Rak): menggantikan kartu buatan pengguna
/// yang sedang tampil, jangan diantrekan di belakangnya; selain itu ikut aturan prioritas.
fn open_card(s: &mut IslandState, c: Content, ctx: &Ctx, fx: &mut Vec<Effect>) {
    if matches!(s.view.content(), Some(Content::Ask(_) | Content::Shelf)) {
        show(s, c, ctx, fx);
    } else {
        offer(s, c, ctx, fx);
    }
}

/// Rak yang sudah tampil diperbarui di tempat; selain itu ikut aturan prioritas.
fn show_shelf(s: &mut IslandState, ctx: &Ctx, fx: &mut Vec<Effect>) {
    if s.view.content() == Some(&Content::Shelf) {
        show(s, Content::Shelf, ctx, fx);
    } else {
        offer(s, Content::Shelf, ctx, fx);
    }
}

/// HUD (volume, kecerahan) hanya menumpang di pulau kosong atau media ringkas; konten lain lebih penting.
fn show_hud(s: &mut IslandState, c: Content, ctx: &Ctx, fx: &mut Vec<Effect>) {
    let free = matches!(
        s.view,
        View::Idle | View::Compact(Content::Media | Content::Volume(_) | Content::Brightness(_))
    );
    if free {
        show(s, c, ctx, fx);
    }
}

/// Konten informatif: boleh ditahan Jangan Ganggu dan menutup sendiri. Hasil review yang
/// bersih (tanpa temuan) juga begini; yang berisi temuan menunggu keputusan pengguna.
fn is_soft(c: &Content) -> bool {
    match c {
        Content::HealthAlert(_)
        | Content::Notice(_)
        | Content::Volume(_)
        | Content::Brightness(_) => true,
        Content::ReviewResult(r) => r.findings.is_empty(),
        _ => false,
    }
}

/// Tampilkan `c` bila lebih penting dari yang sedang tampil; kalau tidak, antrikan.
fn offer(s: &mut IslandState, c: Content, ctx: &Ctx, fx: &mut Vec<Effect>) {
    offer_as(s, c, ctx, fx, true);
}

/// `respect_dnd = false` untuk konten yang diminta pengguna sendiri.
fn offer_as(s: &mut IslandState, c: Content, ctx: &Ctx, fx: &mut Vec<Effect>, respect_dnd: bool) {
    if respect_dnd && s.dnd && is_soft(&c) {
        return;
    }
    match s.view.content().cloned() {
        Some(cur) if cur != Content::Media && cur.priority() >= c.priority() => enqueue(s, c, ctx),
        Some(cur) if cur != Content::Media => {
            enqueue(s, cur, ctx);
            show(s, c, ctx, fx);
        }
        _ => show(s, c, ctx, fx),
    }
}

fn enqueue(s: &mut IslandState, c: Content, ctx: &Ctx) {
    s.queue.push(c);
    s.queue.sort_by_key(|c| Reverse(c.priority())); // stabil: FIFO untuk prioritas sama
    s.queue.truncate(ctx.max_queue);
}

fn show(s: &mut IslandState, c: Content, ctx: &Ctx, fx: &mut Vec<Effect>) {
    s.token += 1;
    if is_soft(&c) || matches!(c, Content::Approval(_)) {
        let ms = if matches!(c, Content::Volume(_) | Content::Brightness(_)) {
            ctx.hud_ms
        } else if matches!(c, Content::Approval(_)) {
            ctx.approval_ms
        } else {
            ctx.auto_dismiss_ms
        };
        fx.push(Effect::StartTimer { token: s.token, ms });
    }
    s.view = match c {
        Content::Media
        | Content::ReviewRunning(_)
        | Content::Volume(_)
        | Content::Brightness(_)
        | Content::Voice(_) => View::Compact(c),
        _ => View::Expanded(c),
    };
}

/// Pilih tampilan berikutnya: antrian teratas, lalu media, lalu idle.
fn settle(s: &mut IslandState, ctx: &Ctx, fx: &mut Vec<Effect>) {
    if s.queue.is_empty() {
        s.token += 1;
        s.view = if let Some(p) = s.voice {
            View::Compact(Content::Voice(p))
        } else if s.media.is_some() {
            View::Compact(Content::Media)
        } else {
            View::Idle
        };
    } else {
        let next = s.queue.remove(0);
        show(s, next, ctx, fx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::island::state::VolumeInfo;
    use crate::domain::model::{HealthAlert, MediaInfo, ReviewId, ReviewReport};

    fn run(s: IslandState, a: IslandAction) -> (IslandState, Vec<Effect>) {
        reduce(&s, a, &Ctx::default())
    }
    fn media() -> Option<MediaInfo> {
        Some(MediaInfo {
            title: "t".into(),
            artist: "a".into(),
            ..Default::default()
        })
    }
    fn health() -> IslandAction {
        IslandAction::HealthAlert(HealthAlert {
            message: "istirahat".into(),
        })
    }
    fn report() -> ReviewReport {
        ReviewReport {
            summary: "ok".into(),
            ..Default::default()
        }
    }

    #[test]
    fn media_menampilkan_compact_lalu_idle_saat_berhenti() {
        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        assert_eq!(s.view, View::Compact(Content::Media));
        let (s, _) = run(s, IslandAction::MediaChanged(None));
        assert_eq!(s.view, View::Idle);
    }

    #[test]
    fn klik_melebarkan_media_dan_dismiss_mengecilkan() {
        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        let (s, _) = run(s, IslandAction::Clicked);
        assert_eq!(s.view, View::Expanded(Content::Media));
        let (s, _) = run(s, IslandAction::Dismissed);
        assert_eq!(s.view, View::Compact(Content::Media));
    }

    #[test]
    fn review_mengalahkan_media_dan_media_kembali_setelah_dismiss() {
        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        let (s, _) = run(s, IslandAction::ReviewStarted(ReviewId(1)));
        assert_eq!(s.view, View::Compact(Content::ReviewRunning(ReviewId(1))));
        let (s, _) = run(s, IslandAction::ReviewFinished(ReviewId(1), report()));
        assert_eq!(s.view, View::Expanded(Content::ReviewResult(report())));
        let (s, _) = run(s, IslandAction::Dismissed);
        assert_eq!(s.view, View::Compact(Content::Media));
    }

    #[test]
    fn prioritas_rendah_diantrikan_lalu_muncul_setelah_dismiss() {
        let (s, _) = run(
            IslandState::default(),
            IslandAction::ReviewStarted(ReviewId(1)),
        );
        let (s, fx) = run(s, health());
        assert_eq!(s.view, View::Compact(Content::ReviewRunning(ReviewId(1))));
        assert!(fx.is_empty());
        assert_eq!(s.queue.len(), 1);
        let (s, _) = run(s, IslandAction::ReviewFinished(ReviewId(1), report()));
        let (s, _) = run(s, IslandAction::Dismissed);
        assert!(matches!(s.view, View::Expanded(Content::HealthAlert(_))));
        assert!(s.queue.is_empty());
    }

    #[test]
    fn prioritas_tinggi_menggeser_dan_mengantrikan_yang_lama() {
        let (s, _) = run(IslandState::default(), health());
        let (s, _) = run(s, IslandAction::ReviewStarted(ReviewId(2)));
        assert_eq!(s.view, View::Compact(Content::ReviewRunning(ReviewId(2))));
        assert!(matches!(s.queue[0], Content::HealthAlert(_)));
    }

    #[test]
    fn antrian_dibatasi_dan_membuang_prioritas_terendah() {
        let mut s = IslandState::default();
        s = run(s, IslandAction::ReviewStarted(ReviewId(1))).0;
        for i in 0..7 {
            s = run(s, IslandAction::Notice(format!("n{i}"))).0;
        }
        s = run(s, health()).0;
        assert_eq!(s.queue.len(), 5);
        assert!(matches!(s.queue[0], Content::HealthAlert(_)));
    }

    #[test]
    fn dnd_menahan_kesehatan_tapi_tidak_review() {
        let (s, _) = run(IslandState::default(), IslandAction::DoNotDisturb(true));
        let (s, _) = run(s, health());
        assert_eq!(s.view, View::Idle);
        let (s, _) = run(s, IslandAction::ReviewStarted(ReviewId(1)));
        assert_eq!(s.view, View::Compact(Content::ReviewRunning(ReviewId(1))));
    }

    #[test]
    fn dnd_menahan_notice_otomatis_tapi_tidak_klik_pengguna() {
        let (s, _) = run(IslandState::default(), IslandAction::DoNotDisturb(true));
        let (s, _) = run(s, IslandAction::Notice("otomatis".into()));
        assert_eq!(s.view, View::Idle, "pemberitahuan otomatis ditahan");
        let (s, _) = run(s, IslandAction::UserNotice("klik saya".into()));
        assert_eq!(
            s.view,
            View::Expanded(Content::Notice("klik saya".into())),
            "klik pengguna harus tetap membuka pulau, kalau tidak Jangan Ganggu tak bisa dimatikan"
        );
    }

    #[test]
    fn menyalakan_dnd_tidak_menutup_kartu_yang_sedang_tampil() {
        let (s, _) = run(IslandState::default(), IslandAction::Notice("halo".into()));
        let before = s.view.clone();
        let (s, _) = run(s, IslandAction::DoNotDisturb(true));
        assert_eq!(s.view, before);
        assert!(s.dnd);
    }

    #[test]
    fn timeout_usang_diabaikan_dan_timeout_valid_menutup_alert() {
        let (s, fx) = run(IslandState::default(), health());
        let Effect::StartTimer { token, .. } = fx[0];
        let (s, _) = run(s, IslandAction::Timeout(token + 99));
        assert!(matches!(s.view, View::Expanded(Content::HealthAlert(_))));
        let (s, _) = run(s, IslandAction::Timeout(token));
        assert_eq!(s.view, View::Idle);
    }

    #[test]
    fn tombol_musik_membuka_kartu_musik_dari_kartu_lain_hanya_bila_ada_lagu() {
        let (s, _) = run(IslandState::default(), IslandAction::MediaShow);
        assert_eq!(s.view, View::Idle);
        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        let (s, _) = run(s, IslandAction::AskOpen);
        assert_eq!(s.view, View::Expanded(Content::Ask(AskPhase::Input)));
        let (s, _) = run(s, IslandAction::MediaShow);
        assert_eq!(s.view, View::Expanded(Content::Media));
    }

    #[test]
    fn rak_dan_tanya_ai_dari_dok_saling_menggantikan_bukan_mengantre() {
        let (s, _) = run(IslandState::default(), IslandAction::AskOpen);
        let (s, _) = run(s, IslandAction::ShelfShow);
        assert_eq!(s.view, View::Expanded(Content::Shelf));
        assert!(s.queue.is_empty());
        let (s, _) = run(s, IslandAction::AskOpen);
        assert_eq!(s.view, View::Expanded(Content::Ask(AskPhase::Input)));
        assert!(s.queue.is_empty());
    }

    #[test]
    fn hover_keluar_dari_media_lebar_menjadwalkan_collapse_dan_hover_masuk_membatalkan() {
        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        let (s, _) = run(s, IslandAction::Clicked);
        let (s, fx) = run(s, IslandAction::Hover(false));
        let Effect::StartTimer { token, .. } = fx[0];
        let (s2, _) = run(s.clone(), IslandAction::Hover(true));
        let (s2, _) = run(s2, IslandAction::Timeout(token));
        assert_eq!(s2.view, View::Expanded(Content::Media));
        let (s, _) = run(s, IslandAction::Timeout(token));
        assert_eq!(s.view, View::Compact(Content::Media));
    }

    #[test]
    fn hud_volume_singkat_diperbarui_di_tempat_dan_kembali_ke_media() {
        let v = |level| {
            IslandAction::VolumeChanged(VolumeInfo {
                level,
                muted: false,
            })
        };
        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        let (s, fx) = run(s, v(40));
        assert!(matches!(s.view, View::Compact(Content::Volume(i)) if i.level == 40));
        let Effect::StartTimer { token: first, ms } = fx[0];
        assert_eq!(ms, Ctx::default().hud_ms);

        let (s, fx) = run(s, v(42));
        assert!(matches!(s.view, View::Compact(Content::Volume(i)) if i.level == 42));
        let Effect::StartTimer { token: second, .. } = fx[0];
        let (s2, _) = run(s.clone(), IslandAction::Timeout(first));
        assert!(
            matches!(s2.view, View::Compact(Content::Volume(_))),
            "timer lama diabaikan"
        );
        let (s, _) = run(s, IslandAction::Timeout(second));
        assert_eq!(s.view, View::Compact(Content::Media));
    }

    #[test]
    fn hud_kecerahan_tampil_singkat_lalu_kembali_ke_media_dan_tidak_menimpa_review() {
        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        let (s, fx) = run(s, IslandAction::BrightnessChanged(60));
        assert_eq!(s.view, View::Compact(Content::Brightness(60)));
        let Effect::StartTimer { token, ms } = fx[0];
        assert_eq!(ms, Ctx::default().hud_ms);
        let (s, _) = run(s, IslandAction::Timeout(token));
        assert_eq!(s.view, View::Compact(Content::Media));

        let (s, _) = run(
            IslandState::default(),
            IslandAction::ReviewStarted(ReviewId(1)),
        );
        let (s, fx) = run(s, IslandAction::BrightnessChanged(10));
        assert_eq!(s.view, View::Compact(Content::ReviewRunning(ReviewId(1))));
        assert!(fx.is_empty());
    }

    #[test]
    fn hud_volume_tidak_menimpa_konten_yang_lebih_penting() {
        let v = IslandAction::VolumeChanged(VolumeInfo {
            level: 10,
            muted: true,
        });
        let (s, _) = run(
            IslandState::default(),
            IslandAction::ReviewStarted(ReviewId(1)),
        );
        let (s, fx) = run(s, v);
        assert_eq!(s.view, View::Compact(Content::ReviewRunning(ReviewId(1))));
        assert!(fx.is_empty());

        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        let (s, _) = run(s, IslandAction::Clicked);
        let (s, _) = run(
            s,
            IslandAction::VolumeChanged(VolumeInfo {
                level: 5,
                muted: false,
            }),
        );
        assert_eq!(
            s.view,
            View::Expanded(Content::Media),
            "media lebar tidak ditutup"
        );
    }

    #[test]
    fn rak_menampung_tanpa_duplikat_terbatas_dan_tetap_ada_setelah_ditutup() {
        let add =
            |names: &[&str]| IslandAction::ShelfAdd(names.iter().map(|n| n.to_string()).collect());
        let (s, _) = run(IslandState::default(), add(&["a", "b"]));
        assert_eq!(s.view, View::Expanded(Content::Shelf));
        let (s, _) = run(s, add(&["a"]));
        assert_eq!(s.shelf, ["b", "a"], "duplikat dipindah ke akhir");

        let many: Vec<String> = (0..25).map(|i| format!("f{i}")).collect();
        let (s, _) = run(s, IslandAction::ShelfAdd(many));
        assert_eq!(s.shelf.len(), SHELF_MAX);
        assert_eq!(s.shelf.last().map(String::as_str), Some("f24"));

        let (s, _) = run(s, IslandAction::Dismissed);
        assert_eq!(s.view, View::Idle);
        assert_eq!(
            s.shelf.len(),
            SHELF_MAX,
            "isi rak tidak hilang saat ditutup"
        );
        let (s, _) = run(s, IslandAction::ShelfShow);
        assert_eq!(s.view, View::Expanded(Content::Shelf));
        let (s, _) = run(s, IslandAction::ShelfRemove("f24".into()));
        assert_eq!(s.shelf.len(), SHELF_MAX - 1);
        let (s, _) = run(s, IslandAction::ShelfClear);
        assert!(s.shelf.is_empty());
    }

    #[test]
    fn rak_tidak_menggeser_hasil_review_yang_menunggu() {
        let mut r = report();
        r.findings.push(Default::default());
        let (s, _) = run(
            IslandState::default(),
            IslandAction::ReviewFinished(ReviewId(1), r),
        );
        let (s, _) = run(s, IslandAction::ShelfAdd(vec!["a".into()]));
        assert!(matches!(s.view, View::Expanded(Content::ReviewResult(_))));
        assert_eq!(s.queue, [Content::Shelf]);
    }

    #[test]
    fn fullscreen_hanya_mengubah_flag() {
        let (s, _) = run(
            IslandState::default(),
            IslandAction::HideForFullscreen(true),
        );
        assert!(s.fullscreen);
        assert_eq!(s.view, View::Idle);
    }

    #[test]
    fn suara_menumpang_di_pulau_kosong_dan_hilang_saat_selesai() {
        let (s, _) = run(
            IslandState::default(),
            IslandAction::VoiceChanged(Some(VoicePhase::Listening)),
        );
        assert_eq!(s.view, View::Compact(Content::Voice(VoicePhase::Listening)));
        let (s, fx) = run(s, IslandAction::VoiceChanged(Some(VoicePhase::Speaking)));
        assert_eq!(s.view, View::Compact(Content::Voice(VoicePhase::Speaking)));
        assert!(fx.is_empty(), "fase suara tidak punya timer");
        let (s, _) = run(s, IslandAction::VoiceChanged(None));
        assert_eq!(s.view, View::Idle);
    }

    #[test]
    fn suara_kembali_ke_media_dan_tidak_menimpa_kartu_penting() {
        let (s, _) = run(IslandState::default(), IslandAction::MediaChanged(media()));
        let (s, _) = run(s, IslandAction::VoiceChanged(Some(VoicePhase::Thinking)));
        assert_eq!(s.view, View::Compact(Content::Voice(VoicePhase::Thinking)));
        let (s, _) = run(s, IslandAction::VoiceChanged(None));
        assert_eq!(s.view, View::Compact(Content::Media));

        // Kartu penting sedang tampil: fase disimpan, tampilan tidak berubah.
        let (s, _) = run(s, health());
        let (s, _) = run(s, IslandAction::VoiceChanged(Some(VoicePhase::Speaking)));
        assert!(matches!(s.view, View::Expanded(Content::HealthAlert(_))));
        // Setelah kartu selesai, indikator muncul bila Jarvis masih aktif.
        let (s, _) = run(s, IslandAction::Dismissed);
        assert_eq!(s.view, View::Compact(Content::Voice(VoicePhase::Speaking)));
    }

    fn approval() -> IslandAction {
        IslandAction::ApprovalRequested(crate::domain::island::state::Approval {
            id: "a1".into(),
            text: "Jarvis ingin menjalankan perintah: ls".into(),
        })
    }

    #[test]
    fn kartu_izin_menimpa_yang_lain_dan_tidak_ditahan_jangan_ganggu() {
        let (s, _) = run(IslandState::default(), IslandAction::DoNotDisturb(true));
        let (s, fx) = run(s, approval());
        assert!(matches!(s.view, View::Expanded(Content::Approval(_))));
        assert_eq!(fx.len(), 1, "kartu izin punya timer sendiri");
        // Notifikasi biasa tidak menimpa; antre di belakang kartu izin.
        let (s, _) = run(s, IslandAction::Notice("x".into()));
        assert!(matches!(s.view, View::Expanded(Content::Approval(_))));
    }

    #[test]
    fn kartu_izin_tanpa_jawaban_tertutup_oleh_timer_dan_dismiss_menutup() {
        let (s, fx) = run(IslandState::default(), approval());
        let Effect::StartTimer { token, ms } = fx[0];
        assert_eq!(ms, Ctx::default().approval_ms);
        let (after, _) = run(s.clone(), IslandAction::Timeout(token));
        assert_eq!(after.view, View::Idle);
        let (after, _) = run(s, IslandAction::Dismissed);
        assert_eq!(after.view, View::Idle);
    }
}
