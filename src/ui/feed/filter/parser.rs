use std::fmt;

use super::kind_chips::KindFilter;
use crate::data::model::{Plan, QueryEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    SlowQueries,
    CollScanOnly,
}

impl Preset {
    pub fn label(self) -> &'static str {
        match self {
            Preset::SlowQueries => "slow queries",
            Preset::CollScanOnly => "COLLSCANs only",
        }
    }

    pub fn token(self) -> &'static str {
        match self {
            Preset::SlowQueries => "slow",
            Preset::CollScanOnly => "collscan",
        }
    }

    pub fn all() -> &'static [Preset] {
        &[Preset::SlowQueries, Preset::CollScanOnly]
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token())
    }
}

/// Typed filter — all filter state as typed fields.
#[derive(Debug, Clone)]
pub struct Filter {
    pub db: Option<String>,
    pub coll: Option<String>,
    pub app: Option<String>,
    pub kind: KindFilter,
    pub preset: Option<Preset>,
    pub text: Option<String>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            db: None,
            coll: None,
            app: None,
            kind: KindFilter::All,
            preset: None,
            text: None,
        }
    }
}

fn is_chip_token(token: &str) -> bool {
    token.starts_with("db:")
        || token.starts_with("coll:")
        || token.starts_with("app:")
        || token == "slow"
        || token == "slow:true"
        || token == "collscan"
        || token == "collscan:true"
}

impl Filter {
    pub fn parse(input: &str) -> Self {
        let mut f = Filter::default();
        for token in input.split_whitespace() {
            if let Some(val) = token.strip_prefix("db:") {
                f.db = Some(val.to_lowercase());
            } else if let Some(val) = token.strip_prefix("coll:") {
                f.coll = Some(val.to_lowercase());
            } else if let Some(val) = token.strip_prefix("app:") {
                f.app = Some(val.to_lowercase());
            } else if token == "slow" || token == "slow:true" {
                if f.preset.is_none() {
                    f.preset = Some(Preset::SlowQueries);
                }
            } else if token == "collscan" || token == "collscan:true" {
                if f.preset.is_none() {
                    f.preset = Some(Preset::CollScanOnly);
                }
            } else if !token.is_empty() {
                let t = token.to_lowercase();
                f.text = Some(match f.text.take() {
                    None => t,
                    Some(existing) => format!("{} {}", existing, t),
                });
            }
        }
        f
    }

    pub fn matches(&self, entry: &QueryEntry) -> bool {
        if !self.kind.matches(&entry.op) {
            return false;
        }
        if let Some(db) = &self.db {
            if !entry.db.as_str().to_lowercase().contains(db.as_str()) {
                return false;
            }
        }
        if let Some(coll) = &self.coll {
            if !entry.coll.as_str().to_lowercase().contains(coll.as_str()) {
                return false;
            }
        }
        if let Some(app) = &self.app {
            if !entry.app.as_str().to_lowercase().contains(app.as_str()) {
                return false;
            }
        }
        match self.preset {
            Some(Preset::SlowQueries) if !entry.slow => return false,
            Some(Preset::CollScanOnly) if entry.plan != Some(Plan::CollScan) => return false,
            _ => {}
        }
        if let Some(text) = &self.text {
            let haystack = format!(
                "{} {} {}",
                entry.db.as_str(),
                entry.coll.as_str(),
                entry.app.as_str()
            )
            .to_lowercase();
            if !haystack.contains(text.as_str()) {
                return false;
            }
        }
        true
    }

    pub fn chip_tokens(text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter(|t| is_chip_token(t))
            .map(str::to_string)
            .collect()
    }

    pub fn non_chip_text(text: &str) -> String {
        text.split_whitespace()
            .filter(|t| !is_chip_token(t))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn remove_token(text: &str, token: &str) -> String {
        let mut removed = false;
        text.split_whitespace()
            .filter(|t| {
                if !removed && *t == token {
                    removed = true;
                    false
                } else {
                    true
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts: Vec<String> = Vec::new();
        if let Some(db) = &self.db {
            parts.push(format!("db:{}", db));
        }
        if let Some(coll) = &self.coll {
            parts.push(format!("coll:{}", coll));
        }
        if let Some(app) = &self.app {
            parts.push(format!("app:{}", app));
        }
        if let Some(preset) = self.preset {
            parts.push(preset.to_string());
        }
        if let Some(text) = &self.text {
            parts.push(text.clone());
        }
        write!(f, "{}", parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{
        model::{Op, Plan},
        types::*,
    };

    fn entry(db: &str, coll: &str) -> QueryEntry {
        QueryEntry {
            id: QueryId::try_new(1).unwrap(),
            t_ms: TimestampMs::new(0),
            latency_ms: LatencyMs::new(1),
            op: Op::Find,
            db: DatabaseName::try_new(db).unwrap(),
            coll: CollectionName::try_new(coll).unwrap(),
            app: AppName::try_new("testapp").unwrap(),
            plan: None,
            index: None,
            docs_examined: None,
            docs_returned: None,
            filter: None,
            pipeline: None,
            update: None,
            doc: None,
            warn: None,
            slow: false,
            conn_id: ConnId::new(10001),
            lsid: None,
            cluster_time: None,
            response_docs: vec![],
            rejected_plan_count: 0,
        }
    }

    fn slow_entry() -> QueryEntry {
        QueryEntry {
            slow: true,
            ..entry("shop", "orders")
        }
    }

    fn collscan_entry() -> QueryEntry {
        QueryEntry {
            plan: Some(Plan::CollScan),
            ..entry("shop", "orders")
        }
    }

    #[test]
    fn parse_db_token() {
        let f = Filter::parse("db:shop");
        assert_eq!(f.db, Some("shop".into()));
    }

    #[test]
    fn parse_coll_token() {
        let f = Filter::parse("coll:orders");
        assert_eq!(f.coll, Some("orders".into()));
    }

    #[test]
    fn parse_app_token() {
        let f = Filter::parse("app:api");
        assert_eq!(f.app, Some("api".into()));
    }

    #[test]
    fn parse_slow_sets_preset() {
        let f = Filter::parse("slow");
        assert_eq!(f.preset, Some(Preset::SlowQueries));
    }

    #[test]
    fn parse_collscan_sets_preset() {
        let f = Filter::parse("collscan");
        assert_eq!(f.preset, Some(Preset::CollScanOnly));
    }

    #[test]
    fn parse_bare_text_goes_to_text_field() {
        let f = Filter::parse("foo bar");
        assert_eq!(f.text, Some("foo bar".into()));
    }

    #[test]
    fn display_round_trips() {
        let mut f = Filter::default();
        f.db = Some("shop".into());
        f.coll = Some("orders".into());
        f.preset = Some(Preset::SlowQueries);
        assert_eq!(f.to_string(), "db:shop coll:orders slow");
    }

    #[test]
    fn matches_db_filter() {
        let f = Filter::parse("db:shop");
        assert!(f.matches(&entry("shop", "orders")));
        assert!(!f.matches(&entry("analytics", "pageviews")));
    }

    #[test]
    fn matches_slow_preset() {
        let f = Filter::parse("slow");
        assert!(f.matches(&slow_entry()));
        assert!(!f.matches(&entry("shop", "orders")));
    }

    #[test]
    fn matches_collscan_preset() {
        let f = Filter::parse("collscan");
        assert!(f.matches(&collscan_entry()));
        assert!(!f.matches(&entry("shop", "orders")));
    }

    #[test]
    fn chip_tokens_extracts_known_prefixes() {
        let chips = Filter::chip_tokens("db:shop coll:orders foo");
        assert_eq!(chips, vec!["db:shop", "coll:orders"]);
    }

    #[test]
    fn chip_tokens_includes_collscan_not_warn() {
        let chips = Filter::chip_tokens("slow collscan warn app:api");
        assert_eq!(chips, vec!["slow", "collscan", "app:api"]);
    }

    #[test]
    fn non_chip_text_returns_remainder() {
        let rem = Filter::non_chip_text("db:shop coll:orders foo bar");
        assert_eq!(rem, "foo bar");
    }

    #[test]
    fn remove_token_removes_first_match() {
        let result = Filter::remove_token("db:shop coll:orders foo", "coll:orders");
        assert_eq!(result, "db:shop foo");
    }
}
