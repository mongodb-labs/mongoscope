use crate::data::model::QueryEntry;

/// Simple filter predicate parsed from a text expression.
/// Supports: `db:name`, `coll:name`, `app:name`, `slow`, `warn`, bare text.
#[derive(Debug, Clone, Default)]
pub struct FilterExpr {
    pub db: Option<String>,
    pub coll: Option<String>,
    pub app: Option<String>,
    pub slow: Option<bool>,
    pub warn: Option<bool>,
    pub text: Option<String>,
}

fn is_chip_token(token: &str) -> bool {
    token.starts_with("db:")
        || token.starts_with("coll:")
        || token.starts_with("app:")
        || token == "slow"
        || token == "slow:true"
        || token == "warn"
        || token == "warn:true"
}

impl FilterExpr {
    pub fn parse(input: &str) -> Self {
        let mut expr = FilterExpr::default();
        for token in input.split_whitespace() {
            if let Some(val) = token.strip_prefix("db:") {
                expr.db = Some(val.to_lowercase());
            } else if let Some(val) = token.strip_prefix("coll:") {
                expr.coll = Some(val.to_lowercase());
            } else if let Some(val) = token.strip_prefix("app:") {
                expr.app = Some(val.to_lowercase());
            } else if token == "slow:true" || token == "slow" {
                expr.slow = Some(true);
            } else if token == "warn:true" || token == "warn" {
                expr.warn = Some(true);
            } else if !token.is_empty() {
                let t = token.to_lowercase();
                expr.text = Some(match expr.text.take() {
                    None => t,
                    Some(existing) => format!("{} {}", existing, t),
                });
            }
        }
        expr
    }

    pub fn matches(&self, entry: &QueryEntry) -> bool {
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
        if let Some(true) = self.slow {
            if !entry.slow {
                return false;
            }
        }
        if let Some(true) = self.warn {
            if entry.warn.is_none() {
                return false;
            }
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

    /// Returns the recognized filter tokens from `text` (those that will render as chips).
    pub fn chip_tokens(text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter(|t| is_chip_token(t))
            .map(str::to_string)
            .collect()
    }

    /// Returns the part of `text` that is NOT recognized filter tokens.
    pub fn non_chip_text(text: &str) -> String {
        text.split_whitespace()
            .filter(|t| !is_chip_token(t))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Returns `text` with the first occurrence of `token` removed.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{model::Op, types::*};

    fn entry(db: &str, coll: &str) -> crate::data::model::QueryEntry {
        crate::data::model::QueryEntry {
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
        }
    }

    #[test]
    fn parse_db_token() {
        let expr = FilterExpr::parse("db:shop");
        assert_eq!(expr.db, Some("shop".into()));
    }

    #[test]
    fn matches_db_filter() {
        let expr = FilterExpr::parse("db:shop");
        assert!(expr.matches(&entry("shop", "orders")));
        assert!(!expr.matches(&entry("analytics", "pageviews")));
    }

    #[test]
    fn chip_tokens_extracts_known_prefixes() {
        let chips = FilterExpr::chip_tokens("db:shop coll:orders foo");
        assert_eq!(chips, vec!["db:shop", "coll:orders"]);
    }

    #[test]
    fn non_chip_text_returns_remainder() {
        let rem = FilterExpr::non_chip_text("db:shop coll:orders foo bar");
        assert_eq!(rem, "foo bar");
    }

    #[test]
    fn remove_token_removes_first_match() {
        let result = FilterExpr::remove_token("db:shop coll:orders foo", "coll:orders");
        assert_eq!(result, "db:shop foo");
    }

    #[test]
    fn chip_tokens_slow_warn() {
        let chips = FilterExpr::chip_tokens("slow warn app:api");
        assert_eq!(chips, vec!["slow", "warn", "app:api"]);
    }

    #[test]
    fn parse_accumulates_bare_text() {
        let expr = FilterExpr::parse("foo bar");
        assert_eq!(expr.text, Some("foo bar".into()));
    }

    #[test]
    fn parse_bare_text_matches_haystack() {
        let expr = FilterExpr::parse("foo bar");
        // "foo bar" is the text filter; haystack is "foo bar testapp"
        assert!(expr.matches(&entry("foo", "bar")));
        // "shop baz testapp" doesn't contain "foo bar" as substring
        assert!(!expr.matches(&entry("shop", "baz")));
    }
}
