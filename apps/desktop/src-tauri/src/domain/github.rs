//! Radar GitHub: PR yang menunggu review kita dan status CI PR kita, dari jawaban GraphQL.
use serde_json::Value;

use crate::domain::error::GithubError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ci {
    Passing,
    Failing,
    Pending,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pr {
    pub repo: String,
    pub title: String,
    pub url: String,
    pub ci: Ci,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Radar {
    /// PR orang lain yang meminta review kita.
    pub review_requests: Vec<Pr>,
    /// PR terbuka milik kita.
    pub mine: Vec<Pr>,
}

fn pr(node: &Value) -> Option<Pr> {
    let ci = match node["commits"]["nodes"][0]["commit"]["statusCheckRollup"]["state"].as_str() {
        Some("SUCCESS") => Ci::Passing,
        Some("FAILURE" | "ERROR") => Ci::Failing,
        Some("PENDING" | "EXPECTED") => Ci::Pending,
        _ => Ci::Unknown,
    };
    Some(Pr {
        repo: node["repository"]["nameWithOwner"].as_str()?.to_string(),
        title: node["title"].as_str()?.to_string(),
        url: node["url"].as_str()?.to_string(),
        ci,
    })
}

fn prs(nodes: &Value) -> Vec<Pr> {
    nodes
        .as_array()
        .map(|a| a.iter().filter_map(pr).collect())
        .unwrap_or_default()
}

/// Baca jawaban GraphQL. Galat dari GitHub (token salah, kuota) menjadi `Unavailable` dengan pesannya.
pub fn parse_radar(json: &str) -> Result<Radar, GithubError> {
    let v: Value = serde_json::from_str(json)
        .map_err(|_| GithubError::Unavailable("jawaban tidak bisa dibaca".into()))?;
    if let Some(msg) = v["errors"][0]["message"].as_str().or(v["message"].as_str()) {
        return Err(GithubError::Unavailable(msg.to_string()));
    }
    let data = &v["data"];
    if data.is_null() {
        return Err(GithubError::Unavailable("jawaban kosong".into()));
    }
    Ok(Radar {
        review_requests: prs(&data["review"]["nodes"]),
        mine: prs(&data["viewer"]["pullRequests"]["nodes"]),
    })
}

/// Kabar baru dibanding pengecekan sebelumnya: permintaan review baru dan PR kita yang baru gagal CI.
pub fn news(before: &Radar, now: &Radar) -> Vec<String> {
    let mut out = Vec::new();
    for p in &now.review_requests {
        if !before.review_requests.iter().any(|b| b.url == p.url) {
            out.push(format!("Review diminta: {} ({})", p.title, p.repo));
        }
    }
    for p in now.mine.iter().filter(|p| p.ci == Ci::Failing) {
        let was_failing = before
            .mine
            .iter()
            .any(|b| b.url == p.url && b.ci == Ci::Failing);
        if !was_failing {
            out.push(format!("CI gagal: {} ({})", p.title, p.repo));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPLY: &str = r#"{"data":{
        "viewer":{"pullRequests":{"nodes":[
            {"title":"Tambah lirik","url":"https://github.com/a/b/pull/1","repository":{"nameWithOwner":"a/b"},
             "commits":{"nodes":[{"commit":{"statusCheckRollup":{"state":"FAILURE"}}}]}},
            {"title":"Perbaiki bug","url":"https://github.com/a/b/pull/2","repository":{"nameWithOwner":"a/b"},
             "commits":{"nodes":[{"commit":{"statusCheckRollup":{"state":"SUCCESS"}}}]}},
            {"title":"Tanpa CI","url":"https://github.com/a/c/pull/3","repository":{"nameWithOwner":"a/c"},
             "commits":{"nodes":[{"commit":{"statusCheckRollup":null}}]}}
        ]}},
        "review":{"nodes":[
            {"title":"Tolong review","url":"https://github.com/x/y/pull/9","repository":{"nameWithOwner":"x/y"}},
            {}
        ]}}}"#;

    #[test]
    fn jawaban_dibaca_termasuk_status_ci_dan_simpul_rusak() {
        let r = parse_radar(REPLY).unwrap();
        assert_eq!(r.mine.len(), 3);
        assert_eq!(
            r.mine.iter().map(|p| p.ci).collect::<Vec<_>>(),
            [Ci::Failing, Ci::Passing, Ci::Unknown]
        );
        assert_eq!(r.review_requests.len(), 1, "simpul kosong dilewati");
        assert_eq!(r.review_requests[0].repo, "x/y");
    }

    #[test]
    fn galat_github_dan_sampah_menjadi_galat() {
        for bad in [
            r#"{"message":"Bad credentials"}"#,
            r#"{"errors":[{"message":"kuota habis"}],"data":null}"#,
            r#"{"data":null}"#,
            "<html>",
        ] {
            assert!(parse_radar(bad).is_err(), "{bad}");
        }
        assert!(
            parse_radar(r#"{"message":"Bad credentials"}"#)
                .unwrap_err()
                .to_string()
                .contains("Bad credentials")
        );
    }

    #[test]
    fn kabar_hanya_untuk_yang_baru() {
        let now = parse_radar(REPLY).unwrap();
        let empty = Radar::default();
        let first = news(&empty, &now);
        assert_eq!(first.len(), 2);
        assert!(first[0].starts_with("Review diminta: Tolong review"));
        assert!(first[1].starts_with("CI gagal: Tambah lirik"));
        assert!(news(&now, &now).is_empty(), "tidak mengulang");

        let mut fixed = now.clone();
        fixed.mine[0].ci = Ci::Passing;
        assert!(news(&now, &fixed).is_empty());
        assert_eq!(
            news(&fixed, &now).len(),
            1,
            "gagal lagi setelah sempat lolos"
        );
    }
}
