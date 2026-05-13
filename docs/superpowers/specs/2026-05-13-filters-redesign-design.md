# Filters Redesign — Design Spec

**Date:** 2026-05-13

---

## Overview

Two changes in one:

1. **Typed filter model** — replace the string-parsed `FilterExpr` with a proper `Filter` struct. Sidebar interactions set typed fields directly; no more string token manipulation (`set_scope`, `set_app`, `remove_token`).
2. **Preset filters panel** — replace the "SAVED VIEWS" sidebar section with a "FILTERS" section containing two hardcoded presets: *slow queries* and *COLLSCANs only*.

---

## Data Model

### `Filter` (replaces `FilterExpr`)

```rust
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub db: Option<String>,
    pub coll: Option<String>,
    pub app: Option<String>,
    pub kind: KindFilter,        // All / Find / Agg / Write / Count / Unknown
    pub preset: Option<Preset>,  // replaces slow + warn booleans
    pub text: Option<String>,    // free-form bare text (unrecognized tokens)
}
```

`Filter` implements `Display`, producing the canonical filter string shown in the text box:

```
db:shop coll:orders app:api slow
```

Token order: `db:` → `coll:` → `app:` → preset keyword → bare text.  
`kind` is not included in the display string (it's represented by the chips, not the text box).

### `Preset`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    SlowQueries,  // matches entry.slow == true
    CollScanOnly, // matches entry.plan == Some(Plan::CollScan)
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
}

impl Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token())
    }
}
```

### `Filter::matches`

```rust
impl Filter {
    pub fn matches(&self, entry: &QueryEntry) -> bool {
        if let Some(db) = &self.db { /* contains check */ }
        if let Some(coll) = &self.coll { /* contains check */ }
        if let Some(app) = &self.app { /* contains check */ }
        if !self.kind.matches(&entry.op) { return false; }
        match self.preset {
            Some(Preset::SlowQueries)  => { if !entry.slow { return false; } }
            Some(Preset::CollScanOnly) => { if entry.plan != Some(Plan::CollScan) { return false; } }
            None => {}
        }
        if let Some(text) = &self.text { /* haystack contains check */ }
        true
    }
}
```

### `FilterState`

```rust
pub struct FilterState {
    pub input: String,   // raw text box content (for widget)
    pub filter: Filter,  // typed filter (source of truth)
}
```

`input` is kept for the text box widget so partial/in-progress typing is preserved. On each keystroke `filter` is re-derived from `input` via `Filter::parse`. On sidebar/preset interactions, `filter` fields are set directly and `input` is regenerated via `filter.to_string()`.

---

## Parsing

`Filter::parse(input: &str) -> Filter` replaces `FilterExpr::parse`. Same token rules:

| Token | Field set |
|-------|-----------|
| `db:<val>` | `db = Some(val)` |
| `coll:<val>` | `coll = Some(val)` |
| `app:<val>` | `app = Some(val)` |
| `slow` / `slow:true` | `preset = Some(Preset::SlowQueries)` |
| `collscan` / `collscan:true` | `preset = Some(Preset::CollScanOnly)` |
| anything else | appended to `text` |

`kind` is not parsed from text (chip-only).  
`warn` token is dropped — no longer supported.

---

## Sidebar: FILTERS Panel

Replaces the "SAVED VIEWS" section. File: `src/ui/sidebar/filters.rs` (replaces `saved_views.rs`).

Two hardcoded presets rendered as toggle rows. Active state = `filter.preset == Some(variant)`.

```
FILTERS
  ★ slow queries          ← highlighted when active
  ★ COLLSCANs only
```

No delete button. No save button.

`FilterMsg` (formerly `SavedViewsMsg`):

```rust
pub enum FilterPanelMsg {
    Toggle(Preset),
}
```

Clicking an active preset sets `filter.preset = None`. Clicking an inactive preset sets `filter.preset = Some(variant)`. Both regenerate `input` from `filter.to_string()`.

The sidebar `view()` gains a `active_preset: Option<Preset>` parameter (derived from active feed's `filter.filter.preset`).

---

## Wiring

### `SidebarMsg`

```rust
SidebarMsg::Filters(FilterPanelMsg)  // replaces SavedViews
```

### Sidebar interactions → `FilterState`

| Interaction | Before | After |
|-------------|--------|-------|
| Click DB | `filter.set_scope(db, coll)` | `filter.filter.db = Some(db); filter.input = filter.filter.to_string()` |
| Click collection | `filter.set_scope(db, coll)` | `filter.filter.coll = Some(coll); ...` |
| Click client | `filter.set_app(app)` | `filter.filter.app = Some(app); ...` |
| Click preset | _(not wired)_ | `filter.filter.preset = Some/None; ...` |

`set_scope`, `set_app`, `remove_token` are removed.

---

## Files Changed

| File | Change |
|------|--------|
| `src/ui/feed/filter/parser.rs` | Rename/rewrite as `Filter` struct with `parse`, `matches`, `Display` |
| `src/ui/feed/filter/mod.rs` | Remove `FilterExpr`; use `Filter`. `FilterState.text` → `FilterState.input`. Remove `set_scope`, `set_app` |
| `src/ui/sidebar/saved_views.rs` → `filters.rs` | Rewrite as hardcoded presets panel, `FilterPanelMsg::Toggle(Preset)` |
| `src/ui/sidebar/mod.rs` | Section header "FILTERS"; `SidebarMsg::Filters`; `saved_views` field removed; `view()` gains `active_preset` param |
| `src/main.rs` | Update all `set_scope`/`set_app` call sites; handle `SidebarMsg::Filters`; pass `active_preset` to sidebar |
| `src/data/model.rs` | No change (Plan enum already correct) |
