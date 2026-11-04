use std::time::Duration;

use crate::domain::error::GithubError;
use crate::domain::github::{Radar, parse_radar};
use crate::domain::ports::GithubApi;

const ENDPOINT: &str = "https://api.github.com/graphql";

/// Satu permintaan GraphQL: PR terbuka kita beserta status CI commit terakhirnya, dan PR yang
/// meminta review kita. `@me` dalam pencarian berarti pemilik token.
const QUERY: &str = r#"query {
  viewer { pullRequests(first: 10, states: OPEN, orderBy: {field: UPDATED_AT, direction: DESC}) {
    nodes { title url repository { nameWithOwner }
      commits(last: 1) { nodes { commit { statusCheckRollup { state } } } } } } }
  review: search(query: "is:pr is:open review-requested:@me archived:false", type: ISSUE, first: 10) {
    nodes { ... on PullRequest { title url repository { nameWithOwner } } } }
}"#;

/// Klien GitHub GraphQL dengan token pribadi (hanya baca).
pub struct GithubGraphql;

impl GithubApi for GithubGraphql {
    fn radar(&self, token: &str) -> Result<Radar, GithubError> {
        let unavailable = |e: &dyn std::fmt::Display| GithubError::Unavailable(e.to_string());
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(concat!("IslaDesk/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| unavailable(&e))?;
        let body = client
            .post(ENDPOINT)
            .bearer_auth(token)
            .json(&serde_json::json!({ "query": QUERY }))
            .send()
            .and_then(|r| r.text())
            .map_err(|e| unavailable(&e))?;
        parse_radar(&body)
    }
}
