//! Ekspresi Lumi diturunkan dari state pulau; tidak ada state tersendiri yang bisa menyimpang.
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::island::state::{Content, IslandState};
use crate::domain::model::review::Severity;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Mood {
    #[default]
    Idle,
    Thinking,
    Happy,
    Worried,
    Panic,
    Listening,
    Sleepy,
}

pub fn mood_for(s: &IslandState) -> Mood {
    let music = s.media.as_ref().is_some_and(|m| m.playing);
    match s.view.content() {
        Some(Content::ReviewRunning(_)) => Mood::Thinking,
        Some(Content::ReviewResult(r)) if r.findings.is_empty() => Mood::Happy,
        Some(Content::ReviewResult(r))
            if r.findings.iter().any(|f| f.severity == Severity::Critical) =>
        {
            Mood::Panic
        }
        Some(Content::ReviewResult(_)) => Mood::Worried,
        Some(Content::HealthAlert(_)) => Mood::Sleepy,
        Some(Content::Notice(_)) => Mood::Happy,
        Some(Content::Media) | None if music => Mood::Listening,
        Some(Content::Media) | None => Mood::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::island::state::View;
    use crate::domain::model::review::{Finding, ReviewReport};
    use crate::domain::model::{HealthAlert, MediaInfo, ReviewId};

    fn with(view: View) -> IslandState {
        IslandState {
            view,
            ..Default::default()
        }
    }

    fn report(sev: Option<Severity>) -> ReviewReport {
        ReviewReport {
            findings: sev
                .map(|severity| Finding {
                    severity,
                    ..Default::default()
                })
                .into_iter()
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn review_menentukan_ekspresi() {
        let m = |c| mood_for(&with(View::Expanded(c)));
        assert_eq!(
            mood_for(&with(View::Compact(Content::ReviewRunning(ReviewId(1))))),
            Mood::Thinking
        );
        assert_eq!(m(Content::ReviewResult(report(None))), Mood::Happy);
        assert_eq!(
            m(Content::ReviewResult(report(Some(Severity::Important)))),
            Mood::Worried
        );
        assert_eq!(
            m(Content::ReviewResult(report(Some(Severity::Critical)))),
            Mood::Panic
        );
    }

    #[test]
    fn kesehatan_mengantuk_dan_musik_membuat_lumi_mengangguk() {
        let alert = Content::HealthAlert(HealthAlert {
            message: "x".into(),
        });
        assert_eq!(mood_for(&with(View::Expanded(alert))), Mood::Sleepy);
        assert_eq!(mood_for(&IslandState::default()), Mood::Idle);
        let playing = IslandState {
            media: Some(MediaInfo {
                playing: true,
                ..Default::default()
            }),
            view: View::Compact(Content::Media),
            ..Default::default()
        };
        assert_eq!(mood_for(&playing), Mood::Listening);
        let paused = IslandState {
            media: Some(MediaInfo {
                playing: false,
                ..Default::default()
            }),
            ..playing
        };
        assert_eq!(mood_for(&paused), Mood::Idle);
    }
}
