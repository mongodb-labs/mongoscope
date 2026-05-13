# Filters Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace string-parsed `FilterExpr` with a typed `Filter` struct + `Preset` enum, and replace the "SAVED VIEWS" sidebar section with a hardcoded "FILTERS" panel.

**Architecture:** `Filter` owns all filter state as typed fields (`db`, `coll`, `app`, `kind`, `preset`, `text`) and implements `Display` to produce the filter text box string. The sidebar FILTERS panel sends `FilterPanelMsg::Toggle(Preset)` which sets/clears `filter.preset` directly. No string token manipulation anywhere.

**Tech Stack:** Rust, iced 0.13, project at `/Users/jeroen.vervaeke/git/github.com/mongodb-labs/mongoscope`

---

## File Map

| File | Change |
|------|--------|
| `src/ui/feed/filter/parser.rs` | Rewrite: `FilterExpr` → `Filter` + `Preset`; add `Display`, update `chip_tokens`/`non_chip_text`/`remove_token` |
| `src/ui/feed/filter/search_input.rs` | Update `FilterExpr::*` → `Filter::*` |
| `src/ui/feed/filter/mod.rs` | `FilterState`: `text`→`input`, `expr`→`filter`; add `set_preset`, `matches`; simplify `set_scope`/`set_app` |
| `src/ui/feed/mod.rs` | `visible_entries`: use `self.filter.matches(e)` |
| `src/ui/sidebar/saved_views.rs` → `src/ui/sidebar/filters.rs` | Rewrite as hardcoded preset panel; `FilterPanelMsg::Toggle(Preset)` |
| `src/ui/sidebar/mod.rs` | `SidebarMsg::Filters`; section header "FILTERS"; `view()` gains `active_preset: Option<Preset>` param |
| `src/main.rs` | Handle `SidebarMsg::Filters`; pass `active_preset` to `sidebar.view()` |

---

## Task 1: Typed `Filter` and `Preset` — rewrite filter core

**Files:**
- Modify: `src/ui/feed/filter/parser.rs`
- Modify: `src/ui/feed/filter/search_input.rs`
- Modify: `src/ui/feed/filter/mod.rs`
- Modify: `src/ui/feed/mod.rs`

- [ ] **Step 1: Rewrite `parser.rs`**

Replace the entire file with:

```rust
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
                f.preset = Some(Preset::SlowQueries);
            } else if token == "collscan" || token == "collscan:true" {
                f.preset = Some(Preset::CollScanOnly);
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
            Some(Preset::SlowQueries) => {
                if !entry.slow {
                    return false;
                }
            }
            Some(Preset::CollScanOnly) => {
                if entry.plan != Some(Plan::CollScan) {
                    return false;
                }
            }
            None => {}
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
        }
    }

    fn slow_entry() -> QueryEntry {
        QueryEntry { slow: true, ..entry("shop", "orders") }
    }

    fn collscan_entry() -> QueryEntry {
        QueryEntry { plan: Some(Plan::CollScan), ..entry("shop", "orders") }
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
```

- [ ] **Step 2: Update `search_input.rs`** — change `FilterExpr::*` to `Filter::*`

In `src/ui/feed/filter/search_input.rs`, replace:
```rust
use crate::{theme::Palette, ui::feed::filter::parser::FilterExpr};
```
with:
```rust
use crate::{theme::Palette, ui::feed::filter::parser::Filter};
```

Replace all three call sites:
```rust
// line 22
let chips = Filter::chip_tokens(&value);
// line 23
let remaining = Filter::non_chip_text(&value);
// line 48
.on_press(on_change(Filter::remove_token(&value_clone, &tok)))
```

- [ ] **Step 3: Rewrite `FilterState` in `filter/mod.rs`**

Replace the entire file with:

```rust
pub mod kind_chips;
pub mod parser;
pub mod search_input;

pub use kind_chips::{kind_chips, KindFilter};
pub use parser::{Filter, Preset};
pub use search_input::search_input;

use crate::{data::model::QueryEntry, theme::Palette};
use iced::{
    widget::{button, container, row, text},
    Border, Element, Length, Padding,
};

#[derive(Debug, Clone)]
pub enum FilterMsg {
    TextChanged(String),
    TextSubmit,
    KindSelected(KindFilter),
    #[allow(dead_code)]
    ClearFilter,
}

pub struct FilterState {
    pub input: String,
    pub filter: Filter,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            filter: Filter::default(),
        }
    }

    fn sync_input(&mut self) {
        self.input = self.filter.to_string();
    }

    pub fn set_scope(&mut self, db: Option<String>, coll: Option<String>) {
        self.filter.db = db;
        self.filter.coll = coll;
        self.sync_input();
    }

    pub fn set_app(&mut self, app: Option<String>) {
        self.filter.app = app;
        self.sync_input();
    }

    pub fn set_preset(&mut self, preset: Option<Preset>) {
        self.filter.preset = preset;
        self.sync_input();
    }

    pub fn matches(&self, entry: &QueryEntry) -> bool {
        self.filter.matches(entry)
    }

    pub fn update(&mut self, msg: FilterMsg) {
        match msg {
            FilterMsg::TextChanged(t) => {
                let kind = self.filter.kind;
                self.filter = Filter::parse(&t);
                self.filter.kind = kind;
                self.input = t;
            }
            FilterMsg::TextSubmit => {
                let kind = self.filter.kind;
                self.filter = Filter::parse(&self.input);
                self.filter.kind = kind;
            }
            FilterMsg::KindSelected(k) => {
                self.filter.kind = k;
            }
            FilterMsg::ClearFilter => {
                self.input.clear();
                self.filter = Filter::default();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn view<'a, Msg: Clone + 'static>(
        &'a self,
        on_msg: impl Fn(FilterMsg) -> Msg + 'static + Copy,
        on_pause: Msg,
        on_clear: Msg,
        scroll_locked: bool,
        visible_count: usize,
        total_count: usize,
        palette: Palette,
    ) -> Element<'a, Msg> {
        let bg1 = palette.bg1;
        let border_color = palette.border;
        let fg_dim = palette.fg_dim;
        let fg_dim2 = palette.fg_dim2;
        let accent = palette.accent;
        let warn = palette.warn;

        let count_str = format!("{}/{}", visible_count, total_count);
        let count_color = if visible_count < total_count {
            accent
        } else {
            fg_dim2
        };

        let pause_label = if scroll_locked { "▶" } else { "||" };
        let pause_color = if scroll_locked { warn } else { fg_dim };

        let pause_btn = button(
            text(pause_label)
                .size(11)
                .color(pause_color)
                .font(iced::Font::MONOSPACE),
        )
        .padding(Padding { top: 3.0, bottom: 3.0, left: 6.0, right: 6.0 })
        .on_press(on_pause)
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        });

        let clear_btn = button(
            text("✕")
                .size(11)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE),
        )
        .padding(Padding { top: 3.0, bottom: 3.0, left: 6.0, right: 6.0 })
        .on_press(on_clear)
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        });

        container(
            row![
                search_input(
                    self.input.clone(),
                    "filter: db:shop coll:orders app:api slow",
                    move |t| on_msg(FilterMsg::TextChanged(t)),
                    on_msg(FilterMsg::TextSubmit),
                    &palette,
                ),
                kind_chips(
                    self.filter.kind,
                    move |k| on_msg(FilterMsg::KindSelected(k)),
                    &palette
                ),
                iced::widget::Space::new(Length::Fill, 0),
                text(count_str)
                    .size(11)
                    .color(count_color)
                    .font(iced::Font::MONOSPACE),
                pause_btn,
                clear_btn,
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding(Padding { top: 6.0, bottom: 6.0, left: 10.0, right: 6.0 }),
        )
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg1)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_scope_updates_filter_and_input() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), None);
        assert_eq!(fs.filter.db, Some("shop".into()));
        assert_eq!(fs.input, "db:shop");
    }

    #[test]
    fn set_scope_db_and_coll() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        assert_eq!(fs.input, "db:shop coll:orders");
    }

    #[test]
    fn set_scope_replaces_existing() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("old".into()), Some("x".into()));
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        assert_eq!(fs.input, "db:shop coll:orders");
    }

    #[test]
    fn set_scope_none_clears_fields() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        fs.set_scope(None, None);
        assert_eq!(fs.filter.db, None);
        assert_eq!(fs.filter.coll, None);
        assert_eq!(fs.input, "");
    }

    #[test]
    fn set_app_updates_filter_and_input() {
        let mut fs = FilterState::new();
        fs.set_app(Some("myapi".into()));
        assert_eq!(fs.filter.app, Some("myapi".into()));
        assert_eq!(fs.input, "app:myapi");
    }

    #[test]
    fn set_preset_slow_queries() {
        let mut fs = FilterState::new();
        fs.set_preset(Some(Preset::SlowQueries));
        assert_eq!(fs.filter.preset, Some(Preset::SlowQueries));
        assert_eq!(fs.input, "slow");
    }

    #[test]
    fn set_preset_collscan_only() {
        let mut fs = FilterState::new();
        fs.set_preset(Some(Preset::CollScanOnly));
        assert_eq!(fs.filter.preset, Some(Preset::CollScanOnly));
        assert_eq!(fs.input, "collscan");
    }

    #[test]
    fn set_preset_none_clears() {
        let mut fs = FilterState::new();
        fs.set_preset(Some(Preset::SlowQueries));
        fs.set_preset(None);
        assert_eq!(fs.filter.preset, None);
        assert_eq!(fs.input, "");
    }

    #[test]
    fn text_changed_preserves_kind() {
        let mut fs = FilterState::new();
        fs.filter.kind = KindFilter::Find;
        fs.update(FilterMsg::TextChanged("db:shop".into()));
        assert_eq!(fs.filter.kind, KindFilter::Find);
        assert_eq!(fs.filter.db, Some("shop".into()));
    }
}
```

- [ ] **Step 4: Update `visible_entries` in `feed/mod.rs`**

Find line (roughly line 127):
```rust
.filter(|e| self.filter.kind.matches(&e.op) && self.filter.expr.matches(e))
```
Replace with:
```rust
.filter(|e| self.filter.matches(e))
```

- [ ] **Step 5: Run tests**

```bash
cargo fmt && cargo build && cargo test && cargo clippy
```

Expected: all 60+ tests pass, no warnings.

- [ ] **Step 6: Commit**

```bash
git add src/ui/feed/filter/parser.rs \
        src/ui/feed/filter/search_input.rs \
        src/ui/feed/filter/mod.rs \
        src/ui/feed/mod.rs
git commit -m "refactor: typed Filter/Preset replace string-parsed FilterExpr"
```

---

## Task 2: Sidebar FILTERS panel + wiring

**Files:**
- Create: `src/ui/sidebar/filters.rs`
- Modify: `src/ui/sidebar/mod.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create `src/ui/sidebar/filters.rs`**

```rust
use crate::{theme::Palette, ui::feed::filter::parser::Preset};
use iced::{
    widget::{button, column, row, text},
    Border, Element, Length, Padding,
};

#[derive(Debug, Clone)]
pub enum FilterPanelMsg {
    Toggle(Preset),
}

pub fn filters_panel<Msg: Clone + 'static>(
    active: Option<Preset>,
    on_msg: impl Fn(FilterPanelMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0 = palette.bg;
    let bg_hover = palette.bg_hover;
    let fg = palette.fg;
    let fg_dim2 = palette.fg_dim2;
    let accent = palette.accent;

    let rows: Vec<Element<Msg>> = Preset::all()
        .iter()
        .map(|&preset| {
            let is_active = active == Some(preset);
            let star_color = if is_active { accent } else { fg_dim2 };
            let label_color = if is_active { fg } else { fg_dim2 };

            button(
                row![
                    text("★")
                        .size(11)
                        .color(star_color)
                        .font(iced::Font::MONOSPACE),
                    text(preset.label())
                        .size(11)
                        .color(label_color)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(5)
                .align_y(iced::Alignment::Center),
            )
            .padding(Padding {
                top: 5.0,
                bottom: 5.0,
                left: 8.0,
                right: 8.0,
            })
            .width(Length::Fill)
            .on_press(on_msg(FilterPanelMsg::Toggle(preset)))
            .style(move |_, status| button::Style {
                background: Some(iced::Background::Color(match status {
                    iced::widget::button::Status::Hovered => bg_hover,
                    _ => bg0,
                })),
                border: Border::default(),
                ..Default::default()
            })
            .into()
        })
        .collect();

    column(rows)
        .spacing(2)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 0.0,
            right: 0.0,
        })
        .into()
}
```

- [ ] **Step 2: Update `sidebar/mod.rs`** — swap out saved_views, add Filters

Replace the `pub mod saved_views;` line and its exports:
```rust
// Remove:
pub mod saved_views;
pub use saved_views::{saved_views_panel, SavedView, SavedViewsMsg};

// Add:
pub mod filters;
pub use filters::{filters_panel, FilterPanelMsg};
```

In the `SidebarMsg` enum, replace:
```rust
SavedViews(SavedViewsMsg),
```
with:
```rust
Filters(FilterPanelMsg),
```

In `SidebarState`, remove the `saved_views` field and its initialization:
```rust
// Remove from struct:
pub saved_views: Vec<SavedView>,

// Remove from SidebarState::new():
saved_views: vec![
    SavedView { id: 0, label: "slow queries (>500ms)".into() },
    SavedView { id: 1, label: "COLLSCANs only".into() },
    SavedView { id: 2, label: "writes to orders".into() },
],
```

In `SidebarState::update`, replace the `SavedViews` arm:
```rust
// Remove:
SidebarMsg::SavedViews(m) => match m {
    SavedViewsMsg::Delete(id) => self.saved_views.retain(|v| v.id != id),
    SavedViewsMsg::Load(_) | SavedViewsMsg::Save => {}
},

// Add:
SidebarMsg::Filters(_) => {
    // Preset toggle handled in App::update before reaching here.
}
```

Update `SidebarState::view` signature to accept `active_preset`:
```rust
pub fn view<Msg: Clone + 'static>(
    &self,
    on_msg: impl Fn(SidebarMsg) -> Msg + 'static + Copy,
    palette: &Palette,
    width: f32,
    active_preset: Option<crate::ui::feed::filter::parser::Preset>,
) -> Element<'static, Msg>
```

In the view body, replace the SAVED VIEWS section:
```rust
// Remove:
section_header("SAVED VIEWS".into(), None),
saved_views_panel(
    &self.saved_views,
    move |m| on_msg(SidebarMsg::SavedViews(m)),
    palette,
),

// Add:
section_header("FILTERS".into(), None),
filters_panel(
    active_preset,
    move |m| on_msg(SidebarMsg::Filters(m)),
    palette,
),
```

- [ ] **Step 3: Delete `saved_views.rs`**

```bash
rm src/ui/sidebar/saved_views.rs
```

- [ ] **Step 4: Update `main.rs`**

Update the `sidebar.view()` call (around line 385). First compute `active_preset`, then pass it:

```rust
// Before:
.view(Message::Sidebar, &palette, self.sidebar_width);

// After:
.view(
    Message::Sidebar,
    &palette,
    self.sidebar_width,
    self.sidebar.active().and_then(|c| c.feed.filter.filter.preset),
);
```

Add handling for `SidebarMsg::Filters` in the `Message::Sidebar` arm. After the existing `SidebarMsg::Clients` block (around line 237), add:

```rust
if let SidebarMsg::Filters(FilterPanelMsg::Toggle(preset)) = &m {
    let preset = *preset;
    if let Some(conn) = self.sidebar.active_mut() {
        let current = conn.feed.filter.filter.preset;
        let next = if current == Some(preset) { None } else { Some(preset) };
        conn.feed.filter.set_preset(next);
    }
}
```

Add the import at the top of `main.rs` where other sidebar imports are:
```rust
use crate::ui::sidebar::filters::FilterPanelMsg;
```

- [ ] **Step 5: Run tests**

```bash
cargo fmt && cargo build && cargo test && cargo clippy
```

Expected: all tests pass, no warnings.

- [ ] **Step 6: Commit**

```bash
git add src/ui/sidebar/filters.rs \
        src/ui/sidebar/mod.rs \
        src/main.rs
git rm src/ui/sidebar/saved_views.rs
git commit -m "feat: sidebar FILTERS panel with slow queries and COLLSCANs only presets"
```
