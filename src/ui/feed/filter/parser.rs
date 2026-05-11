use crate::data::model::QueryEntry;

/// Simple filter predicate parsed from a text expression.
/// Supports: `coll:name`, `app:name`, `slow:true`, `warn:true`, bare text.
#[derive(Debug, Clone, Default)]
pub struct FilterExpr {
    pub coll: Option<String>,
    pub app: Option<String>,
    pub slow: Option<bool>,
    pub warn: Option<bool>,
    pub text: Option<String>,
}

impl FilterExpr {
    pub fn parse(input: &str) -> Self {
        let mut expr = FilterExpr::default();
        for token in input.split_whitespace() {
            if let Some(val) = token.strip_prefix("coll:") {
                expr.coll = Some(val.to_lowercase());
            } else if let Some(val) = token.strip_prefix("app:") {
                expr.app = Some(val.to_lowercase());
            } else if token == "slow:true" || token == "slow" {
                expr.slow = Some(true);
            } else if token == "warn:true" || token == "warn" {
                expr.warn = Some(true);
            } else if !token.is_empty() {
                expr.text = Some(token.to_lowercase());
            }
        }
        expr
    }

    pub fn matches(&self, entry: &QueryEntry) -> bool {
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
            if !entry.slow { return false; }
        }
        if let Some(true) = self.warn {
            if entry.warn.is_none() { return false; }
        }
        if let Some(text) = &self.text {
            let haystack = format!("{} {}", entry.coll.as_str(), entry.app.as_str()).to_lowercase();
            if !haystack.contains(text.as_str()) { return false; }
        }
        true
    }
}
