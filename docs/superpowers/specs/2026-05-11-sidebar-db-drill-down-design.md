# Sidebar DB Drill-Down Design

**Date:** 2026-05-11  
**Status:** Approved

## Problem

`QueryEntry` has no `db` field. Sidebar shows one hardcoded database. Collection selection in sidebar has no effect on the feed. Multi-database traffic cannot be inspected.

## Goal

Hierarchical sidebar drill-down: select database → filter feed to all events in that database. Select collection within it → narrow to that collection. Click again → deselect (show all).

## Data Layer

### New type: `DatabaseName`

Add `DatabaseName` newtype to `src/data/types.rs` (same pattern as `CollectionName`).

### `QueryEntry` gains `db: DatabaseName`

All existing fields stay. Mock templates updated to assign db per collection (e.g. `orders`, `products`, `users`, `carts`, `sessions`, `reviews`, `inventory`, `events` → `shop`; add `analytics` db with 2 collections and `auth` db with 1 collection for variety).

### Mock data: 3 databases

| Database    | Collections                           |
|-------------|---------------------------------------|
| `shop`      | orders, products, users, carts, sessions, reviews, inventory, events |
| `analytics` | pageviews, funnels                    |
| `auth`      | tokens                                |

## Sidebar

### State restructure

Replace `db_name: String` + `collections: Vec<CollectionItem>` with:

```rust
pub databases: Vec<DatabaseItem>
```

```rust
pub struct DatabaseItem {
    pub name: String,
    pub expanded: bool,
    pub active: bool,        // true = this db is the active filter
    pub collections: Vec<CollectionItem>,
}
```

`CollectionItem` gains no new fields. Its `active` flag means "this collection is the active filter".

### Selection rules

- Click inactive database → set `active = true`, `expanded = true`, clear any active collection within it; deactivate all other databases and their collections.
- Click active database → set `active = false`; clear all active collections. Feed shows all events.
- Click inactive collection → set parent db `active = true` (if not already), set collection `active = true`; deactivate all other dbs/collections.
- Click active collection → set collection `active = false`; parent db remains active (feed shows full db).
- Clicking expand/collapse chevron (if added) is independent of selection.

### Messages

```rust
pub enum DatabasesMsg {
    ToggleDb(String),           // db name
    ToggleCollection(String, String),  // (db name, coll name)
}
```

`SidebarMsg::Databases(DatabasesMsg)` replaces `SidebarMsg::Collections(CollectionsMsg)`.

### `register_entries`

On each batch, for any `(db, coll)` pair not yet in `databases`, insert it (new `DatabaseItem` or append `CollectionItem` to existing one). Databases keep insertion order.

## Filter Bar: Chip-Input

The filter bar text input becomes a chip-input. Valid filter tokens crystallize into removable chips; partial/unrecognized text stays as raw text being typed.

### Chip types

| Token pattern | Chip label |
|---------------|------------|
| `db:shop`     | `db:shop ×` |
| `coll:orders` | `coll:orders ×` |
| `app:api`     | `app:api ×` |
| `slow` / `slow:true` | `slow ×` |
| `warn` / `warn:true` | `warn ×` |

Chips render left of the cursor in the same bar. Clicking `×` removes that chip and its token from `FilterState.text`, re-parses, re-filters.

### Sidebar → filter bar

When sidebar selection changes, `App::update` injects tokens into `FilterState.text`:
- Selecting `shop` db → inserts `db:shop` token (replaces any existing `db:*` token).
- Selecting `orders` collection → inserts `coll:orders` token (replaces any existing `coll:*` token).
- Deselecting db → removes `db:*` token (and `coll:*` token if present).
- Deselecting collection → removes `coll:*` token only.

`FilterExpr::parse` already handles `coll:` and `app:`. It gains `db:` support.

### `FilterExpr` addition

```rust
pub db: Option<String>,
```

Parse rule: `db:val` → `expr.db = Some(val.to_lowercase())`.

Match rule:
```rust
if let Some(db) = &self.db {
    if !entry.db.as_str().to_lowercase().contains(db.as_str()) { return false; }
}
```

`FeedState` no longer needs separate `sidebar_db`/`sidebar_coll` fields — the shared `FilterState` carries the full filter state. Sidebar selection = inject tokens = re-parse = re-filter.

## App::update wire-up

```
SidebarMsg::Databases(DatabasesMsg::ToggleDb(db_name)) =>
    self.sidebar.update(msg);
    // derive new active state
    let active_db = self.sidebar.active_db();   // Option<String>
    let active_coll = self.sidebar.active_coll(); // Option<String>
    self.feed.filter.set_scope(active_db, active_coll);
    // set_scope replaces db:/coll: tokens in filter text and re-parses

SidebarMsg::Databases(DatabasesMsg::ToggleCollection(db_name, coll_name)) =>
    self.sidebar.update(msg);
    let active_db = self.sidebar.active_db();
    let active_coll = self.sidebar.active_coll();
    self.feed.filter.set_scope(active_db, active_coll);
```

`FilterState::set_scope(db: Option<String>, coll: Option<String>)` replaces `db:*` and `coll:*` tokens in `self.text`, then calls `self.expr = FilterExpr::parse(&self.text)`.

## Sidebar UI

Each `DatabaseItem` renders as:

```
▾ shop  [active highlight if selected]
    ◧ orders   2.4M docs · 8.4 GB   7i
    ◧ products  ...
▸ analytics
▸ auth
```

- `▾` / `▸` chevron indicates expanded/collapsed (toggle on click of chevron or db label).
- Clicking db label = toggle selection (not just expand).
- Collections only visible when expanded.
- Active db: label uses `fg` color + `bg_sel` background (same as current active collection style).
- Active collection: same `bg_sel` highlight.

## Invariants

- At most one database active at a time.
- At most one collection active at a time; active collection's parent db must also be active.
- Deselecting db clears collection selection.
- Deselecting collection keeps db selected.
- `sidebar_coll = Some(_)` implies `sidebar_db = Some(_)`.

## Out of Scope

- Multi-select (select multiple dbs or collections simultaneously).
- Collapsing databases separately from selecting them (collapse/expand is a stretch goal, selection always works).
- Persisting selection across restarts.
