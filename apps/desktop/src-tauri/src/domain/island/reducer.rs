use std::cmp::Reverse;

use super::action::{Effect, IslandAction};
use super::state::{Content, Ctx, IslandState, View};

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
        IslandAction::Dismissed => match s.view {
            View::Idle | View::Compact(Content::Media) => {}
            View::Expanded(Content::Media) => s.view = View::Compact(Content::Media),
            _ => settle(&mut s, ctx, &mut fx),
        },
        IslandAction::Timeout(token) => {
            if token == s.token {
                match s.view {
                    View::Expanded(Content::Media) => s.view = View::Compact(Content::Media),
                    View::Expanded(ref c) if is_soft(c) => settle(&mut s, ctx, &mut fx),
                    _ => {}
                }
            }
        }
        IslandAction::HideForFullscreen(b) => s.fullscreen = b,
        IslandAction::ExtensionConnected(b) => s.extension = b,
        IslandAction::DoNotDisturb(b) => {
            s.dnd = b;
            if b {
                s.queue.retain(|c| !is_soft(c));
                if s.view.content().is_some_and(is_soft) {
                    settle(&mut s, ctx, &mut fx);
                }
            }
        }
    }
    (s, fx)
}

/// Konten informatif: boleh ditahan Jangan Ganggu dan menutup sendiri. Hasil review yang
/// bersih (tanpa temuan) juga begini; yang berisi temuan menunggu keputusan pengguna.
fn is_soft(c: &Content) -> bool {
    match c {
        Content::HealthAlert(_) | Content::Notice(_) => true,
        Content::ReviewResult(r) => r.findings.is_empty(),
        _ => false,
    }
}

/// Tampilkan `c` bila lebih penting dari yang sedang tampil; kalau tidak, antrikan.
fn offer(s: &mut IslandState, c: Content, ctx: &Ctx, fx: &mut Vec<Effect>) {
    if s.dnd && is_soft(&c) {
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
    if is_soft(&c) {
        fx.push(Effect::StartTimer {
            token: s.token,
            ms: ctx.auto_dismiss_ms,
        });
    }
    s.view = match c {
        Content::Media | Content::ReviewRunning(_) => View::Compact(c),
        _ => View::Expanded(c),
    };
}

/// Pilih tampilan berikutnya: antrian teratas, lalu media, lalu idle.
fn settle(s: &mut IslandState, ctx: &Ctx, fx: &mut Vec<Effect>) {
    if s.queue.is_empty() {
        s.token += 1;
        s.view = if s.media.is_some() {
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
    fn timeout_usang_diabaikan_dan_timeout_valid_menutup_alert() {
        let (s, fx) = run(IslandState::default(), health());
        let Effect::StartTimer { token, .. } = fx[0];
        let (s, _) = run(s, IslandAction::Timeout(token + 99));
        assert!(matches!(s.view, View::Expanded(Content::HealthAlert(_))));
        let (s, _) = run(s, IslandAction::Timeout(token));
        assert_eq!(s.view, View::Idle);
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
    fn fullscreen_hanya_mengubah_flag() {
        let (s, _) = run(
            IslandState::default(),
            IslandAction::HideForFullscreen(true),
        );
        assert!(s.fullscreen);
        assert_eq!(s.view, View::Idle);
    }
}
