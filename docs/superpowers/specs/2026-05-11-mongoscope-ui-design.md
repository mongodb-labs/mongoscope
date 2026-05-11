# Mongoscope UI — Design Spec

_2026-05-11_

## ⚠ Design URL — HARD REQUIREMENT

The reference HTML prototype is accessed via a private URL provided by the user at the start of each session.

**Rules — no exceptions:**
- The URL **must never be committed, logged, or stored** in any file in this repo (`.env`, docs, code comments, history, CI config — anywhere).
- **Do NOT proceed with any implementation work** if the design URL has not been provided in the current session. Stop and ask the user for it first.
- Every implementation session starts with: fetch the URL → verify the HTML loads → then proceed. If fetch fails, stop and notify the user.

---

## Overview

Iced (Rust) implementation of the Mongoscope HTML prototype: a dense pro-dev-tool UI for inspecting MongoDB wire traffic (Fiddler for MongoDB). Phase 1 is UI-only with mock live data; real wire-protocol capture slots in later by swapping a single trait impl.

---

## Architecture: Elm-style pane composition (Approach A)

Each major pane owns `State + Message + update() + view()`. The root `App` composes them by lifting sub-messages. No shared mutable state across panes — panes communicate only through the root update loop.

```
App
├── sidebar::State  +  sidebar::Msg
├── feed::State     +  feed::Msg
├── inspector::State+  inspector::Msg
├── queries: Vec<Arc<QueryEntry>>   (shared read-only via Arc)
├── selected_id: Option<QueryId>    (lives at App level, read by both feed + inspector)
├── theme: Theme
├── density: Density
├── dock: Dock
└── show_density_lane: bool
```

---

## File Tree

Target: every file stays under ~80 lines. `widgets/` has zero state — pure `fn foo(...) -> Element<Msg>`. Only `mod.rs` files wire sub-components together.

```
src/
  main.rs                         — App struct, Message enum, update, view, subscription
  data/
    mod.rs                        — re-exports
    types.rs                      — all nutype domain primitives
    model.rs                      — QueryEntry, Collection, ClientApp, BsonDoc, Op, Plan
    source.rs                     — DataSource trait (swap point)
    mock/
      mod.rs                      — MockSource: async Stream<Item=QueryEntry>
      templates.rs                — the 15 query templates + buildFeed logic
  theme.rs                        — Theme, Density, Dock, Palette, all color tokens
  ui/
    mod.rs
    topbar.rs                     — view fn (no state)
    statusbar.rs                  — view fn (no state)
    widgets/                      — reusable primitives, zero state
      mod.rs
      op_badge.rs                 — OpBadge view fn
      plan_chip.rs                — PlanChip view fn
      latency_bar.rs              — LatencyBar + format_latency()
      bson_view.rs                — BsonView (recursive syntax-highlighted tree)
      mini_card.rs                — MiniCard (titled card shell)
      warn_banner.rs              — WarnBanner
      kv_grid.rs                  — KvGrid + KvRow
      ghost_btn.rs                — ghost_button() styled button helper
      icon_btn.rs                 — icon_button() helper
      section_header.rs           — uppercase label (sidebar + cards)
      toggle.rs                   — Toggle (rules on/off)
      gantt.rs                    — GanttRow + GanttTrack
      flame_row.rs                — FlameRow (explain plan stage)
      schema_row.rs               — SchemaRow
    sidebar/
      mod.rs                      — State, Msg, update, view (composes sections)
      connections.rs              — ConnectionSection + ConnectionItem
      collections.rs              — CollectionSection + CollectionItem
      clients.rs                  — ClientSection + ClientItem
      saved_views.rs              — SavedViewsSection
    feed/
      mod.rs                      — State, Msg, update, view (composes sub-views)
      filter/
        mod.rs                    — FilterBar (composes below)
        search_input.rs           — SearchInput
        kind_chips.rs             — KindChipGroup
      density_lane.rs             — Canvas Program (80-bucket ribbon)
      table/
        mod.rs                    — FeedTable (header + scrollable list)
        header.rs                 — FeedHeader (column labels)
        row.rs                    — FeedRow view fn
        cells.rs                  — per-column cell renderers
    inspector/
      mod.rs                      — State, Msg, update, view (tab shell + router only)
      header.rs                   — InspectorHeader (title + action buttons)
      tabs/
        overview/
          mod.rs                  — overview_tab()
          hero.rs                 — op badge + latency hero
          stats.rs                — KvGrid of query stats
          efficiency.rs           — EfficiencyCard
        request/
          mod.rs                  — request_tab()
          header.rs               — OP_MSG metadata bar + action buttons
        response/
          mod.rs
          header.rs
        explain/
          mod.rs                  — explain_tab()
          plan_flame.rs           — list of FlameRows
          suggestions.rs          — SuggestionList + SuggRow
        timeline/
          mod.rs
          gantt.rs                — GanttChart (phase breakdown)
          neighbours.rs           — NeighbourList
        compose/
          mod.rs
          editor.rs               — ComposeHeader + footer layout
          shell_gen.rs            — pure fn shell_gen(q: &QueryEntry) -> String, one branch per Op
        rules/
          mod.rs
          rule_list.rs            — RuleList + RuleItem
          interception.rs         — PendingInterception panel
        schema/
          mod.rs
          field_list.rs           — FieldList + FieldRow (with depth indent)
```

---

## Domain Types (nutype)

All primitives in `data/types.rs` via the `nutype` crate:

| Type | Inner | Constraints |
|------|-------|-------------|
| `QueryId` | `u64` | `greater = 0` |
| `LatencyMs` | `u32` | — |
| `TimestampMs` | `u64` | — |
| `CollectionName` | `String` | `sanitize(trim)`, `validate(not_empty)` |
| `AppName` | `String` | `sanitize(trim)`, `validate(not_empty)` |
| `IndexName` | `String` | `sanitize(trim)`, `validate(not_empty)` |
| `DocsExamined` | `u64` | — |
| `DocsReturned` | `u64` | — |
| `FilterText` | `String` | `sanitize(trim)`, `validate(not_empty)` |
| `ComposeText` | `String` | `sanitize(trim)`, `validate(not_empty)` |

`QueryEntry` uses all of these. Mock generator constructs via `::try_new().expect(...)` — fail-fast on bad test data.

---

## Data Model

```rust
pub struct QueryEntry {
    pub id: QueryId,
    pub t_ms: TimestampMs,
    pub latency_ms: LatencyMs,
    pub op: Op,
    pub coll: CollectionName,
    pub app: AppName,
    pub plan: Option<Plan>,
    pub index: Option<IndexName>,
    pub docs_examined: Option<DocsExamined>,
    pub docs_returned: Option<DocsReturned>,
    pub filter: Option<BsonDoc>,
    pub pipeline: Option<Vec<BsonDoc>>,
    pub update: Option<BsonDoc>,
    pub doc: Option<BsonDoc>,
    pub warn: Option<String>,
}

pub enum Op {
    // Reads
    Find,
    FindOne,
    Aggregate,
    CountDocuments,
    // Writes
    InsertOne,
    UpdateOne,
    UpdateMany,
    DeleteOne,
    DeleteMany,
    // Catch-all — raw command name from wire protocol.
    // All mock data starts here; specific variants are promoted as support is added.
    // Display as uppercase raw string in OpBadge. KindFilter treats as "other".
    Unknown(String),
}

pub enum Plan {
    CollScan,
    IxScan(IndexName),
    IdHack,
    IxScanLookup(IndexName),
    // Unrecognized plan stage name — display as-is, no color coding.
    Unknown(String),
}

pub type BsonDoc = IndexMap<String, BsonVal>;

pub enum BsonVal {
    Doc(BsonDoc),
    Array(Vec<BsonVal>),
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    ObjectId(String),
    IsoDate(String),
    Null,
}

pub struct Collection {
    pub name: CollectionName,
    pub doc_count: u64,
    pub size_human: String,
    pub index_count: u8,
}

pub struct ClientApp {
    pub name: AppName,
    pub color: [u8; 3],
}
```

---

## DataSource Trait (swap point)

```rust
// data/source.rs
pub trait DataSource: Send + 'static {
    fn subscribe(self) -> impl Stream<Item = QueryEntry> + Send;
}
```

`MockSource` (in `data/mock/`) implements this. Cycles through 15 query templates with ±20% latency jitter, emitting ~1 query per 80ms. To use real wire capture: implement `DataSource` on a proxy type and pass it to the subscription at startup.

**Mock data bootstrapping strategy:** Initially all templates emit `Op::Unknown("find")`, `Op::Unknown("aggregate")`, etc. Specific `Op` variants are promoted one at a time as their display/filter support is implemented and verified. This prevents incomplete `match` arms from blocking compilation during development — add a variant, handle it everywhere, promote the mock template.

---

## Message Inventory

### App-level
```rust
pub enum Message {
    Sidebar(sidebar::Msg),
    Feed(feed::Msg),
    Inspector(inspector::Msg),
    QueryReceived(Arc<QueryEntry>),
    ThemeToggled,
    DensityChanged(Density),
    DockChanged(Dock),
    DensityLaneToggled,
}
```

### sidebar::Msg
```rust
pub enum Msg {
    ConnectionSelected(usize),
    ConnectionAdded,
    CollectionSelected(Option<usize>),
    ClientFilterSelected(Option<usize>),
    SavedViewActivated(usize),
}
```

### feed::Msg
```rust
pub enum Msg {
    FilterTextChanged(String),
    FilterTextCleared,
    KindFilterChanged(KindFilter),
    CaptureToggled,
    FeedCleared,
}
```
Note: `RowSelected` lives at App level (`Message::RowSelected(QueryId)`) so both Feed and Inspector react.

`KindFilter` variants:
```rust
pub enum KindFilter {
    All,
    Reads,   // Find | FindOne | Aggregate | CountDocuments
    Writes,  // InsertOne | UpdateOne | UpdateMany | DeleteOne | DeleteMany
    Slow,    // latency_ms >= 1000
    Scans,   // Plan::CollScan
    Unknown, // Op::Unknown(_)
}
```

### inspector::Msg
```rust
pub enum Msg {
    TabChanged(Tab),
    Closed,
    Pinned,
    Shared,
    // Overview
    SuggestIndex,
    // Request
    CopyRequest,
    CopyAsShell,
    CopyRawBytes,
    // Response
    ExportResponse,
    CopyAsJson,
    DiffResponse,
    // Explain
    ExplainViewChanged(ExplainView),
    ShowRejectedPlans,
    ApplySuggestion(usize),
    TrySuggestion(usize),
    // Compose
    ComposeTextChanged(String),
    ReplayQuery,
    DryRunQuery,
    RunQuery,
    // Rules
    RuleToggled(usize),
    RuleMenuOpened(usize),
    NewRuleCreated,
    InterceptionEditRequest,
    InterceptionStepOver,
    InterceptionContinue,
    InterceptionAbort,
}
```

---

## Component States

```rust
// sidebar::State
pub struct State {
    pub selected_connection: usize,
    pub selected_collection: Option<usize>,
    pub selected_client: Option<usize>,
}

// feed::State
pub struct State {
    pub filter_text: String,
    pub kind_filter: KindFilter,  // All | Reads | Writes | Slow | Scans
    pub capture_running: bool,
}

// inspector::State
pub struct State {
    pub active_tab: Tab,           // Overview | Request | Response | Explain | Timeline | Compose | Rules | Schema
    pub compose_text: String,
    pub explain_view: ExplainView, // Tree | Flame | Raw
    pub pinned: bool,
}
```

---

## Live Data Flow

```
MockSource::subscribe()
  → Stream<QueryEntry>
  → iced::subscription::run()
  → Message::QueryReceived(Arc<QueryEntry>)
  → App::update() → queries.push(entry), cap at 5000
  → feed view re-renders filtered slice
  → inspector view re-renders selected entry
```

Filtering is pure: `feed_rows(queries, feed_state, sidebar_state)` returns `Vec<&Arc<QueryEntry>>` on each frame. No cached intermediate state — the feed is short enough that linear scan is fast.

---

## Visual Components

### DensityLane (Canvas)
80 equal-time buckets over the visible feed. Each bucket bar height = count / max_count. Color: `danger` if any slow, `warn` if max_lat > 100ms, `accent` otherwise. Implemented as `iced::widget::canvas::Canvas` with a stateless `Program`.

### FeedRow
Grid of 10 columns matching the HTML prototype:
`#` · `t+ms` · `op badge` · `namespace` · `filter/pipeline summary` · `plan chip` · `examined` · `returned` · `client` · `latency bar + value`

### OpBadge / PlanChip / LatencyBar
Reusable view functions. Color mapped from `theme::Palette` fields.

### BsonView (`widgets/bson_view.rs`)
Recursive syntax-highlighted tree. Rules:
- Keys: `tok_key`. Keys starting with `$` or containing `.` are quoted.
- Strings: `tok_str` (`"value"`). Strings matching `^ObjectId|^ISODate|^NumberDecimal|^\$\$NOW` render unquoted as `tok_call`.
- Numbers: `tok_num`. Booleans/null: `tok_lit` (italic).
- Brackets `{` `}` `[` `]`: `tok_br`. Colons/commas: `tok_colon`.
- Depth indented: `8 + depth * 14` px left padding per level.
- Short primitive arrays (≤4 items, no nested objects) inline on one line.
- Hover on any line highlights it and shows a left accent border.

---

## Inspector Tabs — Detailed

Inspector shell: fixed header (38px) + scrollable tab strip (30px compact / 36px comfy) + scrollable body. Docks right (520px wide) or bottom (360px tall). Empty state shows "Select a query to inspect" centered.

**Header:** left = `OpBadge` + `shop.{coll}` + `· #{id}`; right = pin `◉`, share `↗`, close `✕` icon buttons.

### Tab 1 — Overview

Layout (top → bottom, all in padded scroll body):

1. **Hero row** (`overview/hero.rs`): left = `OpBadge` + namespace text; right = large latency number (24px, colored by severity) + "total wall clock" dim label.
2. **WarnBanner** (`widgets/warn_banner.rs`, conditional on `q.warn`): amber background + border, `◆` icon, bold warn title, subtext "Tap Explain → Tree…", ghost "Suggest index" button → `Msg::SuggestIndex`.
3. **Stats grid** (`overview/stats.rs`): 2-column `KvGrid`, 11 rows: operation · namespace · plan · index · docs examined · docs returned · examined/returned ratio · latency · client · connection id · started timestamp.
4. **Efficiency card** (`overview/efficiency.rs`): `MiniCard` titled "efficiency". Gradient bar (ok→warn→danger). Needle position = `100 / max(1, log10(ratio+1) * 3)`%. Labels: "optimal (1×)" · "fair (50×)" · "poor (1000×+)".

### Tab 2 — Request (`request/`)

1. **ReqHeader** (`request/header.rs`): monospace metadata line "OP_MSG · msg_id={id} · {size} B"; right = ghost buttons: "copy" → `CopyRequest`, "as shell" → `CopyAsShell`, "raw bytes" → `CopyRawBytes`.
2. **BsonView** of reconstructed wire document: `{ $db, {op}: {coll}, filter?, projection?, sort?, limit?, pipeline?, cursor?, updates?, documents?, lsid, $clusterTime }`.

### Tab 3 — Response (`response/`)

1. **RespHeader** (`response/header.rs`): "OP_MSG · response · {n} docs · {bytes} B"; right = ghost buttons: "export" → `ExportResponse`, "as json" → `CopyAsJson`, "diff" → `DiffResponse`.
2. **BsonView** of response document:
   - Reads/aggregate: `{ cursor: { firstBatch: [...up to 3 docs...], id: NumberLong(0), ns }, ok: 1 }`. Docs shaped per collection (orders/products/generic).
   - Writes: `{ n, nModified?, ok: 1 }`.

### Tab 4 — Explain (`explain/`)

1. **ExplainHeader**: left = "winning plan · {plan}" (plan colored `danger` if COLLSCAN); right = 4 view toggle ghost buttons — "Tree" · "Flame" [active] · "Raw" · "Rejected plans (3)" → `ExplainViewChanged` / `ShowRejectedPlans`.
2. **PlanFlame** (`explain/plan_flame.rs`): vertical list of `FlameRow`s. Each row is a 4-column grid:
   - Col 1 (160px): stage name, colored by severity (`danger` = COLLSCAN, `warn` = in-memory SORT, neutral otherwise).
   - Col 2: horizontal track; filled bar width = `max(3%, ms/total_ms * 100%)`; bar color matches severity; ms value shown inside bar in `accent_fg`.
   - Col 3 (90px): docs count right-aligned.
   - Col 4: optional note text dimmed (e.g. "no index usable", "spill risk").
   - Stage sets per plan type:
     - COLLSCAN: COLLSCAN (92% time, bad) + SORT in memory (6%, warn) + LIMIT (1ms)
     - IDHACK: single IDHACK row
     - IXSCAN: IXSCAN·{index} (35%) + FETCH (45%) + optional SORT MERGE (10%) + optional LIMIT
3. **Suggestions card** (`explain/suggestions.rs`): `MiniCard` titled "suggestions".
   - COLLSCAN: two `SuggRow`s — (1) "create index" + `db.{coll}.createIndex(...)` code chip + "est. ~4ms · 99.9% faster" + "Apply" button → `ApplySuggestion(0)`; (2) "add covered projection" + `.project({...})` + "avoid FETCH stage" + "Try" → `TrySuggestion(1)`.
   - Indexed: single "looks healthy" row with green label + description of index used.

### Tab 5 — Timeline (`timeline/`)

1. **TimelineHeader**: dim "t+0" left, "t+{total}ms" right.
2. **GanttChart** (`timeline/gantt.rs`): 6 phase rows. Each row = label (90px) + relative-positioned track. Bar absolute-positioned: `left = acc/total * 100%`, `width = max(1%, phase_ms/total * 100%)`. Bar background = phase color from palette. ms shown inside bar. Phases: parse (1ms, `t_parse`) · auth (1ms, `t_auth`) · plan (3%, `t_plan`) · exec (85%, `t_exec`) · serialize (8%, `t_ser`) · network↑ (1ms, `t_net`).
3. **NeighbourList** (`timeline/neighbours.rs`): `MiniCard` titled "neighbours on connection {conn_id}". 7 rows: the 3 queries before, the selected query (highlighted with `bg_sel`), the 3 after. Each row = 4 cols: `+{t_ms}` dim · latency bar (width = `max(8, min(280, log10(lat+1) * 60))px`, colored by severity) · `{op} · {coll}` · `{latency}` dim.

### Tab 6 — Compose (`compose/`)

1. **ComposeHeader** (`compose/editor.rs`): left = "replay on {connection} · or switch →" (connection name accented); right = ghost "↻ Replay" → `ReplayQuery`, ghost "◇ Dry-run" → `DryRunQuery`, solid "▶ Run (⌘↵)" → `RunQuery`.
2. **Editor**: `text_input` (multiline) pre-populated with `shell_gen(query)` → `ComposeTextChanged`. Shell gen (`compose/shell_gen.rs`) formats per op: `db.coll.find({...}).sort({...}).limit(N)`, `db.coll.aggregate([...])`, `db.coll.updateOne({filter},{update})`, etc. `Op::Unknown(cmd)` → `db.coll.runCommand({cmd: ...})`.
3. **ComposeFooter**: left = "shell · mongosh 2.4.0" dim; right = "↑↓ history · ⌘K palette · ⌘↵ run" dim.

Note: `compose/shell_gen.rs` is a separate pure function file — complex enough (one branch per Op variant) to warrant isolation.

### Tab 7 — Rules (`rules/`)

1. **RuleHeader**: left = "4 active rules · matched 102× this session" dim; right = solid "+" New rule" button → `NewRuleCreated`.
2. **RuleList** (`rules/rule_list.rs`): 4 hardcoded rules. Each `RuleItem` = 4-col grid:
   - Col 1: `Toggle` widget (pill, green when on, grey when off) → `RuleToggled(i)`.
   - Col 2: two-line body — `WHEN {condition}` (mono) + `DO {action}` (accent colored).
   - Col 3: `{hits}×` dim.
   - Col 4: `⋯` icon button → `RuleMenuOpened(i)`.
   - Disabled rule (off): opacity 0.55.
3. **PendingInterception** (`rules/interception.rs`): `MiniCard` titled "pending interception". Body: "paused → {op}.{coll} · {app}" + dim "Rule matched: {condition}". Action buttons row: ghost "Edit request" → `InterceptionEditRequest`, ghost "Step over" → `InterceptionStepOver`, solid "▶ Continue" → `InterceptionContinue`, danger "✕ Abort" → `InterceptionAbort`.

### Tab 8 — Schema (`schema/`)

1. **SchemaHeader**: "schema of shop.{coll}" (coll accented) + "· inferred from 2,000 sampled docs" dim.
2. **FieldList** (`schema/field_list.rs`): one `FieldRow` per field. 4-column grid:
   - Col 1 (180px): field name mono, indented `4 + depth * 14` px. Nested fields prefixed with dim "└─ " and show only the last path segment.
   - Col 2 (100px): type string dim (ObjectId / String / Decimal / enum / Array\<Doc\> / etc.)
   - Col 3 (120px): coverage — thin bar (`schcov-bar`, accent fill) + `{pct}%` dim.
   - Col 4: sample values joined with " · ", dim, truncated with ellipsis.
   - Collection-specific field sets: orders (13 fields with nested shipping.country, items.sku/qty) vs products (8 fields) vs generic fallback.

---

## Theme

```rust
pub enum Theme { Dark, Light }
pub enum Density { Compact, Comfy }
pub enum Dock { Right, Bottom }

pub struct Palette {
    // backgrounds
    pub bg: Color, pub bg1: Color, pub bg2: Color,
    pub bg_sel: Color, pub bg_hover: Color,
    // foregrounds
    pub fg: Color, pub fg_dim: Color, pub fg_dim2: Color,
    // borders
    pub border: Color, pub border2: Color,
    // semantic
    pub accent: Color, pub accent_fg: Color,
    pub warn: Color, pub danger: Color, pub ok: Color,
    // op colors
    pub op_read: Color, pub op_write: Color, pub op_agg: Color, pub op_delete: Color,
    // timeline phase colors
    pub t_parse: Color, pub t_auth: Color, pub t_plan: Color,
    pub t_exec: Color, pub t_ser: Color, pub t_net: Color,
    // bson token colors
    pub tok_key: Color, pub tok_str: Color, pub tok_num: Color,
    pub tok_lit: Color, pub tok_call: Color, pub tok_br: Color,
    pub tok_p: Color, pub tok_colon: Color,
}
```

All values sourced from `mongoscope.css` oklch tokens, converted to sRGB.

---

## Non-goals (mock phase)

- No real MongoDB connection or wire protocol parsing
- No disk persistence (saved views, rules are hardcoded)
- All "Copy / Export / Run / Apply / Suggest" messages log to `eprintln!` (ready to wire)
- No window resizing/drag for inspector split (fixed proportions)

---

## Dependencies (Cargo.toml)

- `iced` 0.13 (features: `canvas`, `tokio`)
- `nutype` 0.5 (features: `serde`)
- `indexmap` 2 (ordered BsonDoc)
- `tokio` 1 (features: `time`, `rt-multi-thread`)
- `rand` 0.8 (mock jitter)
