# Sidebar DB Drill-Down Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add multi-database support — hierarchical sidebar (db → collection) with selection that filters the feed and reflects as removable chips in the filter bar.

**Architecture:** Add `db: DatabaseName` to `QueryEntry` and mock templates. Restructure sidebar state from flat collections to `Vec<DatabaseItem>`. Sidebar selection calls `FilterState::set_scope` in `App::update`, which injects/removes `db:`/`coll:` tokens in the shared filter text. The filter bar renders valid tokens as removable chips.

**Tech Stack:** Rust, iced 0.13 (Elm-style), nutype 0.5, tokio

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src/data/types.rs` | Modify | Add `DatabaseName` newtype |
| `src/data/model.rs` | Modify | Add `db: DatabaseName` to `QueryEntry` |
| `src/data/mock/templates.rs` | Modify | Add `db` field to `Template`; add analytics/auth templates |
| `src/data/mock/mod.rs` | Modify | Populate `db` from template when building `QueryEntry` |
| `src/ui/feed/filter/parser.rs` | Modify | Add `db` field to `FilterExpr`; add `chip_tokens`, `non_chip_text`, `remove_token` helpers |
| `src/ui/feed/filter/mod.rs` | Modify | Add `FilterState::set_scope` |
| `src/ui/feed/filter/search_input.rs` | Modify | Chip-input: render chips for valid tokens + text_input for remainder |
| `src/ui/sidebar/databases.rs` | Create | `DatabaseItem`, `DatabasesMsg`, `databases_panel` |
| `src/ui/sidebar/mod.rs` | Modify | Replace `db_name`/`collections` with `databases: Vec<DatabaseItem>`; add `active_db()`/`active_coll()`; update `SidebarMsg`, `register_entries`, `update`, `view` |
| `src/ui/sidebar/collections.rs` | Modify | Keep `CollectionItem` struct; remove unused `CollectionsMsg` and `collections_panel` |
| `src/main.rs` | Modify | Wire `SidebarMsg::Databases(_)` → `feed.filter.set_scope` |

---

### Task 1: Add `DatabaseName` newtype

**Files:**
- Modify: `src/data/types.rs`

- [ ] **Step 1: Write the failing test**

Add at the bottom of `src/data/types.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_name_rejects_empty() {
        assert!(DatabaseName::try_new("").is_err());
        assert!(DatabaseName::try_new("  ").is_err());
    }

    #[test]
    fn database_name_trims_and_accepts() {
        let n = DatabaseName::try_new(" shop ").unwrap();
        assert_eq!(n.to_string(), "shop");
    }
}
```

- [ ] **Step 2: Run to confirm it fails**

```
cargo test database_name 2>&1 | head -20
```

Expected: compilation error — `DatabaseName` not found.

- [ ] **Step 3: Add `DatabaseName` to `src/data/types.rs`**

After the `AppName` line:

```rust
#[nutype(sanitize(trim), validate(not_empty), derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display))]
pub struct DatabaseName(String);
```

- [ ] **Step 4: Run to confirm tests pass**

```
cargo test database_name
```

Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/data/types.rs
git commit -m "feat(types): add DatabaseName newtype"
```

---

### Task 2: Add `db` field to `QueryEntry`

**Files:**
- Modify: `src/data/model.rs`

- [ ] **Step 1: Add `db` field to `QueryEntry` struct in `src/data/model.rs`**

After `pub coll: CollectionName,`:

```rust
pub db: DatabaseName,
```

- [ ] **Step 2: Confirm compilation fails on mock/mod.rs**

```
cargo build 2>&1 | grep "error\[" | head -10
```

Expected: `error` — `QueryEntry` struct literal missing field `db`.

- [ ] **Step 3: Commit the struct change (pre-fix)**

Skip — fix in Task 3 which updates mock. Build will be broken until Task 3.

---

### Task 3: Update mock templates with `db` field

**Files:**
- Modify: `src/data/mock/templates.rs`
- Modify: `src/data/mock/mod.rs`

- [ ] **Step 1: Add `db` field to `Template` in `src/data/mock/templates.rs`**

In the `Template` struct, after `pub coll: &'static str,`:

```rust
pub db: &'static str,
```

- [ ] **Step 2: Add `db` to every existing template in `all_templates()`**

All existing templates use collections in the `shop` database. Add `db: "shop",` to each. Also add 3 new templates for `analytics` and `auth` databases.

Complete updated `all_templates()` — replace the entire function body:

```rust
pub fn all_templates() -> Vec<Template> {
    vec![
        Template {
            op: Op::Find,
            db: "shop",
            coll: "orders",
            app: "checkout-svc",
            plan: Some(Plan::IxScan(IndexName::try_new("userId_1_createdAt_-1").unwrap())),
            index: Some("userId_1_createdAt_-1"),
            docs_examined: Some(20),
            docs_returned: Some(20),
            base_latency_ms: 4,
            warn: None,
            slow: false,
            filter_keys: &["userId", "status"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::Aggregate,
            db: "shop",
            coll: "orders",
            app: "analytics-worker",
            plan: Some(Plan::CollScan),
            index: None,
            docs_examined: Some(2_413_882),
            docs_returned: Some(100),
            base_latency_ms: 4821,
            warn: Some("collection scan"),
            slow: true,
            filter_keys: &[],
            pipeline_stages: &["$match", "$group", "$sort", "$limit"],
        },
        Template {
            op: Op::FindOne,
            db: "shop",
            coll: "products",
            app: "catalog-api",
            plan: Some(Plan::IxScan(IndexName::try_new("sku_1").unwrap())),
            index: Some("sku_1"),
            docs_examined: Some(1),
            docs_returned: Some(1),
            base_latency_ms: 1,
            warn: None,
            slow: false,
            filter_keys: &["sku"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::Find,
            db: "shop",
            coll: "products",
            app: "catalog-api",
            plan: Some(Plan::IxScan(IndexName::try_new("category_1_popularity_-1").unwrap())),
            index: Some("category_1_popularity_-1"),
            docs_examined: Some(612),
            docs_returned: Some(40),
            base_latency_ms: 18,
            warn: None,
            slow: false,
            filter_keys: &["category", "price", "inStock"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::UpdateOne,
            db: "shop",
            coll: "carts",
            app: "mobile-bff",
            plan: Some(Plan::IdHack),
            index: None,
            docs_examined: Some(1),
            docs_returned: Some(1),
            base_latency_ms: 2,
            warn: None,
            slow: false,
            filter_keys: &["_id"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::InsertOne,
            db: "shop",
            coll: "events",
            app: "mobile-bff",
            plan: None,
            index: None,
            docs_examined: None,
            docs_returned: None,
            base_latency_ms: 1,
            warn: None,
            slow: false,
            filter_keys: &[],
            pipeline_stages: &[],
        },
        Template {
            op: Op::Find,
            db: "shop",
            coll: "sessions",
            app: "mobile-bff",
            plan: Some(Plan::IxScan(IndexName::try_new("token_1").unwrap())),
            index: Some("token_1"),
            docs_examined: Some(1),
            docs_returned: Some(1),
            base_latency_ms: 1,
            warn: None,
            slow: false,
            filter_keys: &["token"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::Find,
            db: "shop",
            coll: "reviews",
            app: "catalog-api",
            plan: Some(Plan::IxScan(IndexName::try_new("productId_1_helpful_-1").unwrap())),
            index: Some("productId_1_helpful_-1"),
            docs_examined: Some(47),
            docs_returned: Some(10),
            base_latency_ms: 7,
            warn: None,
            slow: false,
            filter_keys: &["productId", "rating"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::Aggregate,
            db: "shop",
            coll: "products",
            app: "admin-portal",
            plan: Some(Plan::IxScanLookup(IndexName::try_new("category_1").unwrap())),
            index: Some("category_1"),
            docs_examined: Some(18_422),
            docs_returned: Some(3_201),
            base_latency_ms: 612,
            warn: Some("unbounded $lookup"),
            slow: false,
            filter_keys: &[],
            pipeline_stages: &["$match", "$lookup", "$addFields", "$sort"],
        },
        Template {
            op: Op::FindOne,
            db: "shop",
            coll: "users",
            app: "admin-portal",
            plan: Some(Plan::IxScan(IndexName::try_new("email_1").unwrap())),
            index: Some("email_1"),
            docs_examined: Some(1),
            docs_returned: Some(1),
            base_latency_ms: 2,
            warn: None,
            slow: false,
            filter_keys: &["email"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::UpdateMany,
            db: "shop",
            coll: "inventory",
            app: "checkout-svc",
            plan: Some(Plan::IxScan(IndexName::try_new("warehouse_1_sku_1").unwrap())),
            index: Some("warehouse_1_sku_1"),
            docs_examined: Some(2),
            docs_returned: Some(2),
            base_latency_ms: 3,
            warn: None,
            slow: false,
            filter_keys: &["warehouse", "sku"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::DeleteOne,
            db: "shop",
            coll: "carts",
            app: "checkout-svc",
            plan: Some(Plan::IdHack),
            index: None,
            docs_examined: Some(1),
            docs_returned: Some(1),
            base_latency_ms: 1,
            warn: None,
            slow: false,
            filter_keys: &["_id"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::Find,
            db: "shop",
            coll: "orders",
            app: "admin-portal",
            plan: Some(Plan::CollScan),
            index: None,
            docs_examined: Some(2_413_882),
            docs_returned: Some(50),
            base_latency_ms: 3104,
            warn: Some("no index on shipping.country"),
            slow: true,
            filter_keys: &["shipping.country", "status"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::Find,
            db: "shop",
            coll: "products",
            app: "catalog-api",
            plan: Some(Plan::IxScan(IndexName::try_new("tags_1").unwrap())),
            index: Some("tags_1"),
            docs_examined: Some(184),
            docs_returned: Some(24),
            base_latency_ms: 11,
            warn: None,
            slow: false,
            filter_keys: &["tags"],
            pipeline_stages: &[],
        },
        Template {
            op: Op::CountDocuments,
            db: "shop",
            coll: "orders",
            app: "admin-portal",
            plan: Some(Plan::IxScan(IndexName::try_new("status_1").unwrap())),
            index: Some("status_1"),
            docs_examined: Some(4_812),
            docs_returned: Some(1),
            base_latency_ms: 9,
            warn: None,
            slow: false,
            filter_keys: &["status"],
            pipeline_stages: &[],
        },
        // analytics database
        Template {
            op: Op::Aggregate,
            db: "analytics",
            coll: "pageviews",
            app: "analytics-worker",
            plan: Some(Plan::IxScan(IndexName::try_new("ts_1_page_1").unwrap())),
            index: Some("ts_1_page_1"),
            docs_examined: Some(45_000),
            docs_returned: Some(200),
            base_latency_ms: 88,
            warn: None,
            slow: false,
            filter_keys: &["ts", "page"],
            pipeline_stages: &["$match", "$group", "$sort"],
        },
        Template {
            op: Op::Find,
            db: "analytics",
            coll: "funnels",
            app: "analytics-worker",
            plan: Some(Plan::CollScan),
            index: None,
            docs_examined: Some(420_100),
            docs_returned: Some(1),
            base_latency_ms: 712,
            warn: Some("collection scan on funnels"),
            slow: true,
            filter_keys: &["campaignId"],
            pipeline_stages: &[],
        },
        // auth database
        Template {
            op: Op::FindOne,
            db: "auth",
            coll: "tokens",
            app: "mobile-bff",
            plan: Some(Plan::IxScan(IndexName::try_new("token_1_exp_1").unwrap())),
            index: Some("token_1_exp_1"),
            docs_examined: Some(1),
            docs_returned: Some(1),
            base_latency_ms: 1,
            warn: None,
            slow: false,
            filter_keys: &["token", "exp"],
            pipeline_stages: &[],
        },
    ]
}
```

- [ ] **Step 3: Update `src/data/mock/mod.rs` to populate `db`**

Add `use crate::data::types::DatabaseName;` to the existing `use crate::data::{...}` block (add `DatabaseName` to the `types::*` import — it's already covered by `types::*`).

In the `QueryEntry { ... }` struct literal inside `MockSource::start`, add after `coll:`:

```rust
db: DatabaseName::try_new(tpl.db).unwrap(),
```

- [ ] **Step 4: Confirm build passes**

```
cargo build 2>&1 | grep "^error" | head -10
```

Expected: no errors (warnings about unused `CollectionsMsg` are OK for now).

- [ ] **Step 5: Commit**

```bash
git add src/data/model.rs src/data/mock/templates.rs src/data/mock/mod.rs src/data/types.rs
git commit -m "feat(data): add DatabaseName type and db field to QueryEntry; mock 3 databases"
```

---

### Task 4: Add `db:` support to `FilterExpr` + chip helpers

**Files:**
- Modify: `src/ui/feed/filter/parser.rs`

- [ ] **Step 1: Write failing tests**

Add a `#[cfg(test)]` block at the bottom of `src/ui/feed/filter/parser.rs`:

```rust
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
}
```

- [ ] **Step 2: Run to confirm tests fail**

```
cargo test parser::tests 2>&1 | head -30
```

Expected: compilation errors — `db` field missing, `chip_tokens`/`non_chip_text`/`remove_token` not found.

- [ ] **Step 3: Update `src/ui/feed/filter/parser.rs`**

Replace the entire file:

```rust
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
                expr.text = Some(token.to_lowercase());
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
}
```

- [ ] **Step 4: Run tests**

```
cargo test parser::tests
```

Expected: 6 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/ui/feed/filter/parser.rs
git commit -m "feat(filter): add db: token support and chip helper methods to FilterExpr"
```

---

### Task 5: Add `FilterState::set_scope`

**Files:**
- Modify: `src/ui/feed/filter/mod.rs`

- [ ] **Step 1: Write failing tests**

Add at the bottom of `src/ui/feed/filter/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_scope_injects_db_token() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), None);
        assert_eq!(fs.text, "db:shop");
        assert_eq!(fs.expr.db, Some("shop".into()));
        assert_eq!(fs.expr.coll, None);
    }

    #[test]
    fn set_scope_injects_db_and_coll() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        assert_eq!(fs.text, "db:shop coll:orders");
    }

    #[test]
    fn set_scope_replaces_existing_db_token() {
        let mut fs = FilterState::new();
        fs.text = "db:old coll:x foo".into();
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        assert_eq!(fs.text, "db:shop coll:orders foo");
    }

    #[test]
    fn set_scope_none_removes_tokens() {
        let mut fs = FilterState::new();
        fs.text = "db:shop coll:orders foo".into();
        fs.set_scope(None, None);
        assert_eq!(fs.text, "foo");
    }

    #[test]
    fn set_scope_db_only_removes_coll() {
        let mut fs = FilterState::new();
        fs.text = "db:shop coll:orders".into();
        fs.set_scope(Some("shop".into()), None);
        assert_eq!(fs.text, "db:shop");
    }
}
```

- [ ] **Step 2: Run to confirm tests fail**

```
cargo test filter::tests 2>&1 | head -20
```

Expected: error — `set_scope` not found.

- [ ] **Step 3: Add `set_scope` to `FilterState` in `src/ui/feed/filter/mod.rs`**

After the `update` method, add:

```rust
/// Replace any existing `db:` and `coll:` tokens in `self.text` with the given values,
/// preserving all other tokens. Calls `None` removes the token.
pub fn set_scope(&mut self, db: Option<String>, coll: Option<String>) {
    // Strip existing db: and coll: tokens
    let rest: String = self.text
        .split_whitespace()
        .filter(|t| !t.starts_with("db:") && !t.starts_with("coll:"))
        .collect::<Vec<_>>()
        .join(" ");

    let mut parts: Vec<String> = Vec::new();
    if let Some(d) = db {
        parts.push(format!("db:{}", d));
    }
    if let Some(c) = coll {
        parts.push(format!("coll:{}", c));
    }
    if !rest.is_empty() {
        parts.push(rest);
    }

    self.text = parts.join(" ");
    self.expr = FilterExpr::parse(&self.text);
}
```

- [ ] **Step 4: Run tests**

```
cargo test filter::tests
```

Expected: 5 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/ui/feed/filter/mod.rs
git commit -m "feat(filter): add FilterState::set_scope for sidebar-driven db/coll filtering"
```

---

### Task 6: Chip-input search widget

**Files:**
- Modify: `src/ui/feed/filter/search_input.rs`
- Modify: `src/ui/feed/filter/mod.rs` (update call site)

- [ ] **Step 1: Replace `src/ui/feed/filter/search_input.rs`**

The widget now takes `value: String` (owned), returns `Element<'static, Msg>`. It renders chips for recognized tokens and a text_input for the remainder.

```rust
use iced::{
    widget::{button, container, row, text, text_input},
    Border, Element, Length, Padding,
};
use crate::{theme::Palette, ui::feed::filter::parser::FilterExpr};

pub fn search_input<Msg: Clone + 'static>(
    value: String,
    placeholder: &'static str,
    on_change: impl Fn(String) -> Msg + 'static + Copy,
    on_submit: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg2 = palette.bg2;
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let border = palette.border;
    let accent = palette.accent;
    let bg_sel = palette.bg_sel;

    let chips = FilterExpr::chip_tokens(&value);
    let remaining = FilterExpr::non_chip_text(&value);
    let chips_prefix = chips.join(" ");

    let chip_els: Vec<Element<'static, Msg>> = chips
        .into_iter()
        .map(|tok| {
            let value_clone = value.clone();
            let tok_label = tok.clone();
            button(
                row![
                    text(tok_label.clone())
                        .size(11)
                        .color(fg)
                        .font(iced::Font::MONOSPACE),
                    text("×")
                        .size(11)
                        .color(fg_dim)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .padding(Padding { top: 2.0, bottom: 2.0, left: 6.0, right: 6.0 })
            .on_press(on_change(FilterExpr::remove_token(&value_clone, &tok)))
            .style(move |_, _| button::Style {
                background: Some(iced::Background::Color(bg_sel)),
                border: Border {
                    color: accent,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .into()
        })
        .collect();

    let input_el = container(
        text_input(placeholder, &remaining)
            .size(12)
            .padding(Padding { top: 5.0, bottom: 5.0, left: 10.0, right: 10.0 })
            .on_input(move |new_remaining: String| {
                let new_full = if chips_prefix.is_empty() {
                    new_remaining
                } else if new_remaining.is_empty() {
                    chips_prefix.clone()
                } else {
                    format!("{} {}", chips_prefix, new_remaining)
                };
                on_change(new_full)
            })
            .on_submit(on_submit)
            .font(iced::Font::MONOSPACE)
            .style(move |_, _| text_input::Style {
                background: iced::Background::Color(bg2),
                border: Border {
                    color: border,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                icon: fg,
                placeholder: fg_dim2,
                value: fg,
                selection: accent,
            }),
    )
    .width(Length::Fill);

    let mut contents: Vec<Element<'static, Msg>> = chip_els;
    contents.push(input_el.into());

    container(
        row(contents)
            .spacing(4)
            .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}
```

- [ ] **Step 2: Update call site in `src/ui/feed/filter/mod.rs`**

In `FilterState::view`, change the `search_input` call from:

```rust
search_input(
    &self.text,
    "filter: coll:orders app:api slow",
    move |t| on_msg(FilterMsg::TextChanged(t)),
    on_msg(FilterMsg::TextSubmit),
    &palette,
),
```

to:

```rust
search_input(
    self.text.clone(),
    "filter: db:shop coll:orders app:api slow",
    move |t| on_msg(FilterMsg::TextChanged(t)),
    on_msg(FilterMsg::TextSubmit),
    &palette,
),
```

- [ ] **Step 3: Confirm build passes**

```
cargo build 2>&1 | grep "^error" | head -10
```

Expected: no errors.

- [ ] **Step 4: Run app and verify chips appear**

```
cargo run
```

Verify:
- Typing `db:shop` in the filter bar → token becomes a chip `[db:shop ×]`
- Clicking `×` on chip → chip removed, feed shows all entries again
- Typing `db:shop coll:orders slow` → three chips appear
- Text after chips (unrecognized) stays as raw text in the input

- [ ] **Step 5: Commit**

```bash
git add src/ui/feed/filter/search_input.rs src/ui/feed/filter/mod.rs
git commit -m "feat(filter): chip-input — valid tokens render as removable chips in filter bar"
```

---

### Task 7: Sidebar `DatabaseItem` and `databases_panel`

**Files:**
- Create: `src/ui/sidebar/databases.rs`
- Modify: `src/ui/sidebar/collections.rs` (remove unused items)

- [ ] **Step 1: Write failing tests for selection logic**

Create `src/ui/sidebar/databases.rs` with the struct definitions and tests:

```rust
use iced::{widget::{button, column, row, text}, Border, Element, Length, Padding};
use crate::theme::Palette;
use crate::ui::sidebar::collections::CollectionItem;

#[derive(Debug, Clone)]
pub struct DatabaseItem {
    pub name: String,
    pub expanded: bool,
    pub active: bool,
    pub collections: Vec<CollectionItem>,
}

#[derive(Debug, Clone)]
pub enum DatabasesMsg {
    ToggleDb(String),
    ToggleCollection(String, String),
}

pub fn apply_toggle_db(databases: &mut Vec<DatabaseItem>, name: &str) {
    for db in databases.iter_mut() {
        if db.name == name {
            db.active = !db.active;
            if db.active {
                db.expanded = true;
            }
            for c in &mut db.collections {
                c.active = false;
            }
        } else {
            db.active = false;
            for c in &mut db.collections {
                c.active = false;
            }
        }
    }
}

pub fn apply_toggle_collection(databases: &mut Vec<DatabaseItem>, db_name: &str, coll_name: &str) {
    for db in databases.iter_mut() {
        if db.name == db_name {
            db.active = true;
            db.expanded = true;
            for c in &mut db.collections {
                if c.name == coll_name {
                    c.active = !c.active;
                } else {
                    c.active = false;
                }
            }
        } else {
            db.active = false;
            for c in &mut db.collections {
                c.active = false;
            }
        }
    }
}

pub fn databases_panel<Msg: Clone + 'static>(
    databases: &[DatabaseItem],
    on_msg: impl Fn(DatabasesMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0 = palette.bg;
    let bg_sel = palette.bg_sel;
    let bg_hover = palette.bg_hover;
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;

    let rows: Vec<Element<Msg>> = databases
        .iter()
        .flat_map(|db| {
            let db_name = db.name.clone();
            let is_db_active = db.active;
            let chevron = if db.expanded { "▾" } else { "▸" };
            let db_bg = if is_db_active { bg_sel } else { bg0 };
            let db_name_click = db_name.clone();

            let db_row: Element<Msg> = button(
                row![
                    text(chevron)
                        .size(10)
                        .color(fg_dim)
                        .font(iced::Font::MONOSPACE),
                    text(db_name.clone())
                        .size(11)
                        .color(fg)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .padding(Padding { top: 5.0, bottom: 5.0, left: 8.0, right: 8.0 })
            .width(Length::Fill)
            .on_press(on_msg(DatabasesMsg::ToggleDb(db_name_click)))
            .style(move |_, status| button::Style {
                background: Some(iced::Background::Color(match status {
                    iced::widget::button::Status::Hovered if !is_db_active => bg_hover,
                    _ => db_bg,
                })),
                border: Border::default(),
                ..Default::default()
            })
            .into();

            let mut items: Vec<Element<Msg>> = vec![db_row];

            if db.expanded {
                for coll in &db.collections {
                    let is_coll_active = coll.active;
                    let coll_bg = if is_coll_active { bg_sel } else { bg0 };
                    let coll_name = coll.name.clone();
                    let sub = format!("{} · {}", coll.docs_str(), coll.size);
                    let idx = format!("{}i", coll.idx);
                    let db_for_coll = db.name.clone();
                    let coll_name_click = coll_name.clone();

                    let coll_row: Element<Msg> = button(
                        row![
                            text("◧")
                                .size(11)
                                .color(fg_dim2)
                                .font(iced::Font::MONOSPACE),
                            column![
                                text(coll_name.clone())
                                    .size(11)
                                    .color(fg)
                                    .font(iced::Font::MONOSPACE),
                                text(sub)
                                    .size(9)
                                    .color(fg_dim2)
                                    .font(iced::Font::MONOSPACE),
                            ]
                            .spacing(1)
                            .width(Length::Fill),
                            text(idx)
                                .size(9)
                                .color(fg_dim)
                                .font(iced::Font::MONOSPACE),
                        ]
                        .spacing(5)
                        .align_y(iced::Alignment::Center),
                    )
                    .padding(Padding { top: 5.0, bottom: 5.0, left: 20.0, right: 8.0 })
                    .width(Length::Fill)
                    .on_press(on_msg(DatabasesMsg::ToggleCollection(
                        db_for_coll,
                        coll_name_click,
                    )))
                    .style(move |_, status| button::Style {
                        background: Some(iced::Background::Color(match status {
                            iced::widget::button::Status::Hovered if !is_coll_active => bg_hover,
                            _ => coll_bg,
                        })),
                        border: Border::default(),
                        ..Default::default()
                    })
                    .into();

                    items.push(coll_row);
                }
            }

            items
        })
        .collect();

    column(rows)
        .spacing(1)
        .padding(Padding { top: 4.0, bottom: 4.0, left: 0.0, right: 0.0 })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db(name: &str, expanded: bool, colls: &[&str]) -> DatabaseItem {
        DatabaseItem {
            name: name.into(),
            expanded,
            active: false,
            collections: colls
                .iter()
                .map(|c| CollectionItem {
                    name: c.to_string(),
                    docs: 0,
                    size: "".into(),
                    idx: 0,
                    active: false,
                })
                .collect(),
        }
    }

    #[test]
    fn toggle_db_activates_and_expands() {
        let mut dbs = vec![make_db("shop", false, &["orders"]), make_db("auth", false, &["tokens"])];
        apply_toggle_db(&mut dbs, "shop");
        assert!(dbs[0].active);
        assert!(dbs[0].expanded);
        assert!(!dbs[1].active);
    }

    #[test]
    fn toggle_db_deactivates_when_already_active() {
        let mut dbs = vec![make_db("shop", true, &["orders"])];
        dbs[0].active = true;
        apply_toggle_db(&mut dbs, "shop");
        assert!(!dbs[0].active);
    }

    #[test]
    fn toggle_db_deactivates_other_dbs() {
        let mut dbs = vec![make_db("shop", true, &[]), make_db("auth", false, &[])];
        dbs[0].active = true;
        apply_toggle_db(&mut dbs, "auth");
        assert!(!dbs[0].active);
        assert!(dbs[1].active);
    }

    #[test]
    fn toggle_collection_activates_parent_db() {
        let mut dbs = vec![make_db("shop", true, &["orders", "products"])];
        apply_toggle_collection(&mut dbs, "shop", "orders");
        assert!(dbs[0].active);
        assert!(dbs[0].collections[0].active);
        assert!(!dbs[0].collections[1].active);
    }

    #[test]
    fn toggle_collection_deactivates_when_already_active() {
        let mut dbs = vec![make_db("shop", true, &["orders"])];
        dbs[0].active = true;
        dbs[0].collections[0].active = true;
        apply_toggle_collection(&mut dbs, "shop", "orders");
        assert!(dbs[0].active);
        assert!(!dbs[0].collections[0].active);
    }

    #[test]
    fn toggle_collection_clears_other_dbs() {
        let mut dbs = vec![
            make_db("shop", true, &["orders"]),
            make_db("auth", true, &["tokens"]),
        ];
        dbs[0].active = true;
        apply_toggle_collection(&mut dbs, "auth", "tokens");
        assert!(!dbs[0].active);
        assert!(dbs[1].active);
        assert!(dbs[1].collections[0].active);
    }
}
```

- [ ] **Step 2: Run to confirm tests fail**

```
cargo test databases::tests 2>&1 | head -20
```

Expected: compilation error — `databases` module not found.

- [ ] **Step 3: Add `pub mod databases;` to `src/ui/sidebar/mod.rs`**

At the top of `src/ui/sidebar/mod.rs`, add:

```rust
pub mod databases;
```

And add to the pub use block:

```rust
pub use databases::{DatabaseItem, DatabasesMsg, databases_panel, apply_toggle_db, apply_toggle_collection};
```

- [ ] **Step 4: Run tests**

```
cargo test databases::tests
```

Expected: 6 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/ui/sidebar/databases.rs src/ui/sidebar/mod.rs
git commit -m "feat(sidebar): add DatabaseItem, DatabasesMsg, databases_panel with toggle logic"
```

---

### Task 8: Restructure `SidebarState` to use databases

**Files:**
- Modify: `src/ui/sidebar/mod.rs`
- Modify: `src/ui/sidebar/collections.rs`

- [ ] **Step 1: Remove `CollectionsMsg` and `collections_panel` from `collections.rs`**

In `src/ui/sidebar/collections.rs`, remove the `CollectionsMsg` enum and the `collections_panel` function entirely (they are replaced by `databases.rs`). Keep `CollectionItem` and its `impl`.

The file should contain only:

```rust
#[derive(Debug, Clone)]
pub struct CollectionItem {
    pub name: String,
    pub docs: u64,
    pub size: String,
    pub idx: u8,
    pub active: bool,
}

impl CollectionItem {
    pub fn docs_str(&self) -> String {
        let d = self.docs;
        if d >= 1_000_000 {
            format!("{:.1}M docs", d as f64 / 1_000_000.0)
        } else if d >= 1_000 {
            format!("{:.0}K docs", d as f64 / 1_000.0)
        } else {
            format!("{} docs", d)
        }
    }
}
```

- [ ] **Step 2: Restructure `SidebarState` in `src/ui/sidebar/mod.rs`**

Replace the entire `mod.rs` with:

```rust
pub mod clients;
pub mod collections;
pub mod connections;
pub mod databases;
pub mod saved_views;

pub use clients::{clients_panel, ClientItem, ClientsMsg};
pub use collections::CollectionItem;
pub use connections::{connections_panel, ConnectionItem, ConnectionsMsg};
pub use databases::{
    apply_toggle_collection, apply_toggle_db, databases_panel, DatabaseItem, DatabasesMsg,
};
pub use saved_views::{saved_views_panel, SavedView, SavedViewsMsg};

use iced::{
    widget::{column, container, row, scrollable, text},
    Border, Element, Length, Padding,
};
use crate::{data::model::QueryEntry, theme::Palette};

#[derive(Debug, Clone)]
pub enum SidebarMsg {
    Connections(ConnectionsMsg),
    Databases(DatabasesMsg),
    Clients(ClientsMsg),
    SavedViews(SavedViewsMsg),
}

pub struct SidebarState {
    pub databases: Vec<DatabaseItem>,
    pub connections: Vec<ConnectionItem>,
    pub clients: Vec<ClientItem>,
    pub saved_views: Vec<SavedView>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            connections: vec![ConnectionItem {
                id: 0,
                label: "localhost".into(),
                topology: "direct".into(),
                active: true,
                live: true,
            }],
            databases: vec![
                DatabaseItem {
                    name: "shop".into(),
                    expanded: true,
                    active: false,
                    collections: vec![
                        CollectionItem { name: "orders".into(),    docs: 2_413_882,  size: "8.4 GB".into(),   idx: 7, active: false },
                        CollectionItem { name: "products".into(),  docs: 184_302,    size: "412 MB".into(),   idx: 5, active: false },
                        CollectionItem { name: "users".into(),     docs: 892_014,    size: "1.8 GB".into(),   idx: 6, active: false },
                        CollectionItem { name: "carts".into(),     docs: 71_205,     size: "98 MB".into(),    idx: 3, active: false },
                        CollectionItem { name: "sessions".into(),  docs: 12_044_119, size: "4.2 GB".into(),   idx: 4, active: false },
                        CollectionItem { name: "reviews".into(),   docs: 3_201_885,  size: "2.1 GB".into(),   idx: 5, active: false },
                        CollectionItem { name: "inventory".into(), docs: 48_112,     size: "64 MB".into(),    idx: 4, active: false },
                        CollectionItem { name: "events".into(),    docs: 88_912_004, size: "41.2 GB".into(),  idx: 2, active: false },
                    ],
                },
                DatabaseItem {
                    name: "analytics".into(),
                    expanded: false,
                    active: false,
                    collections: vec![
                        CollectionItem { name: "pageviews".into(), docs: 12_500_000, size: "8.2 GB".into(),  idx: 3, active: false },
                        CollectionItem { name: "funnels".into(),   docs: 420_100,   size: "312 MB".into(),  idx: 2, active: false },
                    ],
                },
                DatabaseItem {
                    name: "auth".into(),
                    expanded: false,
                    active: false,
                    collections: vec![
                        CollectionItem { name: "tokens".into(), docs: 2_100_000, size: "1.4 GB".into(), idx: 2, active: false },
                    ],
                },
            ],
            clients: vec![],
            saved_views: vec![
                SavedView { id: 0, label: "slow queries (>500ms)".into() },
                SavedView { id: 1, label: "COLLSCANs only".into() },
                SavedView { id: 2, label: "writes to orders".into() },
            ],
        }
    }

    pub fn active_db(&self) -> Option<String> {
        self.databases.iter().find(|d| d.active).map(|d| d.name.clone())
    }

    pub fn active_coll(&self) -> Option<String> {
        self.databases
            .iter()
            .find(|d| d.active)
            .and_then(|d| d.collections.iter().find(|c| c.active))
            .map(|c| c.name.clone())
    }

    pub fn register_entries(&mut self, entries: &[QueryEntry]) {
        for entry in entries {
            // Register client app
            let app_name = entry.app.to_string();
            if !self.clients.iter().any(|c| c.name == app_name) {
                let color = clients::app_color_for(&app_name);
                self.clients.push(ClientItem { name: app_name, color, active: false });
            }
            // Register database/collection (in case live traffic reveals new ones)
            let db_name = entry.db.to_string();
            let coll_name = entry.coll.to_string();
            if let Some(db) = self.databases.iter_mut().find(|d| d.name == db_name) {
                if !db.collections.iter().any(|c| c.name == coll_name) {
                    db.collections.push(CollectionItem {
                        name: coll_name,
                        docs: 0,
                        size: "".into(),
                        idx: 0,
                        active: false,
                    });
                }
            } else {
                self.databases.push(DatabaseItem {
                    name: db_name,
                    expanded: true,
                    active: false,
                    collections: vec![CollectionItem {
                        name: coll_name,
                        docs: 0,
                        size: "".into(),
                        idx: 0,
                        active: false,
                    }],
                });
            }
        }
    }

    pub fn update(&mut self, msg: SidebarMsg) {
        match msg {
            SidebarMsg::Connections(m) => match m {
                ConnectionsMsg::Select(id) => {
                    for c in &mut self.connections {
                        c.active = c.id == id;
                    }
                }
                ConnectionsMsg::Add => {}
            },
            SidebarMsg::Databases(m) => match m {
                DatabasesMsg::ToggleDb(name) => apply_toggle_db(&mut self.databases, &name),
                DatabasesMsg::ToggleCollection(db, coll) => {
                    apply_toggle_collection(&mut self.databases, &db, &coll)
                }
            },
            SidebarMsg::Clients(m) => match m {
                ClientsMsg::Toggle(name) => {
                    for c in &mut self.clients {
                        if c.name == name {
                            c.active = !c.active;
                        }
                    }
                }
            },
            SidebarMsg::SavedViews(m) => match m {
                SavedViewsMsg::Delete(id) => self.saved_views.retain(|v| v.id != id),
                SavedViewsMsg::Load(_) | SavedViewsMsg::Save => {}
            },
        }
    }

    pub fn view<Msg: Clone + 'static>(
        &self,
        on_msg: impl Fn(SidebarMsg) -> Msg + 'static + Copy,
        palette: &Palette,
    ) -> Element<'static, Msg> {
        let bg = palette.bg;
        let bg1 = palette.bg1;
        let border_color = palette.border;
        let fg_dim2 = palette.fg_dim2;
        let fg_dim = palette.fg_dim;

        let section_header = move |label: String, right: Option<String>| -> Element<'static, Msg> {
            let label_el = text(label)
                .size(9)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE);

            let inner: Element<Msg> = if let Some(r) = right {
                row![
                    label_el,
                    iced::widget::Space::new(Length::Fill, 0),
                    text(r).size(9).color(fg_dim).font(iced::Font::MONOSPACE),
                ]
                .into()
            } else {
                label_el.into()
            };

            container(inner)
                .padding(Padding { top: 8.0, bottom: 3.0, left: 8.0, right: 8.0 })
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg1)),
                    border: Border { color: border_color, width: 0.0, radius: 0.0.into() },
                    ..Default::default()
                })
                .into()
        };

        let db_count = self.databases.len();

        let content = column![
            section_header("CONNECTIONS".into(), None),
            connections_panel(
                &self.connections,
                move |m| on_msg(SidebarMsg::Connections(m)),
                palette,
            ),
            section_header("DATABASES".into(), Some(format!("{} dbs", db_count))),
            databases_panel(
                &self.databases,
                move |m| on_msg(SidebarMsg::Databases(m)),
                palette,
            ),
            section_header("CLIENTS".into(), None),
            clients_panel(
                &self.clients,
                move |m| on_msg(SidebarMsg::Clients(m)),
                palette,
            ),
            section_header("SAVED VIEWS".into(), None),
            saved_views_panel(
                &self.saved_views,
                move |m| on_msg(SidebarMsg::SavedViews(m)),
                palette,
            ),
        ]
        .spacing(0)
        .width(Length::Fill);

        container(
            scrollable(content)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(200)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: border_color, width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }
}
```

- [ ] **Step 3: Confirm build passes**

```
cargo build 2>&1 | grep "^error" | head -10
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/ui/sidebar/mod.rs src/ui/sidebar/collections.rs
git commit -m "feat(sidebar): restructure SidebarState to hierarchical DatabaseItem list"
```

---

### Task 9: Wire sidebar selection → feed filter in `main.rs`

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Update `App::update` in `src/main.rs`**

In the `Message::Sidebar(m)` arm, replace:

```rust
Message::Sidebar(m) => {
    self.sidebar.update(m);
}
```

with:

```rust
Message::Sidebar(m) => {
    self.sidebar.update(m.clone());
    if let ui::sidebar::SidebarMsg::Databases(_) = &m {
        self.feed.filter.set_scope(
            self.sidebar.active_db(),
            self.sidebar.active_coll(),
        );
    }
}
```

Also add `use ui::sidebar;` if not already imported, or use the full path. The `SidebarMsg` is already in scope via `use ui::sidebar::SidebarMsg`.

The `m.clone()` requires `SidebarMsg: Clone` — it already derives `Clone`.

- [ ] **Step 2: Confirm build passes**

```
cargo build 2>&1 | grep "^error" | head -10
```

Expected: no errors.

- [ ] **Step 3: Run the app and verify end-to-end**

```
cargo run
```

Verify:
1. Sidebar shows `DATABASES` section with `▾ shop` expanded (8 collections) and `▸ analytics`, `▸ auth` collapsed.
2. Click `shop` → db row highlights, `[db:shop ×]` chip appears in filter bar, feed shows only shop events.
3. Click `orders` collection within shop → `[db:shop ×] [coll:orders ×]` chips appear, feed narrows to orders only.
4. Click `orders` again → `coll:orders` chip disappears, feed widens to all shop events, shop db remains selected.
5. Click `shop` again → both chips disappear, feed shows all events.
6. Click `analytics` → `[db:analytics ×]` chip appears, analytics section expands, feed shows only analytics events.
7. Typing extra text (e.g. `slow`) in filter bar while db chip is active → `slow` stays as chip alongside `db:shop`.
8. Clicking `×` on `db:shop` chip in filter bar → same as clicking shop in sidebar... (sidebar selection state won't update, but feed filter clears — this is acceptable for MVP).

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(app): wire sidebar database selection to feed filter via set_scope"
```

---

### Task 10: Final smoke test

- [ ] **Step 1: Run all tests**

```
cargo test 2>&1 | tail -20
```

Expected: all tests pass, no failures.

- [ ] **Step 2: Run app, full scenario**

```
cargo run
```

Check:
- No panics on startup
- Events stream from all 3 databases
- Sidebar db/coll selection filters correctly
- Chips appear/disappear as expected
- Clearing filter bar (backspace all text) removes all chips and shows all events
- Kind chips (ALL/R/W/D) still work alongside db/coll chips
