# Mongoscope UI — Design Spec

_2026-05-11_

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

```
src/
  main.rs                    — App struct, Message enum, update, view, subscription
  data/
    mod.rs                   — re-exports
    types.rs                 — all nutype domain primitives
    model.rs                 — QueryEntry, Collection, ClientApp, BsonDoc, Op, Plan
    source.rs                — DataSource trait (swap point)
    mock.rs                  — MockSource: async Stream<Item=QueryEntry>
  theme.rs                   — Theme, Density, Dock, Palette, color tokens
  ui/
    mod.rs
    topbar.rs                — view fn (no state)
    statusbar.rs             — view fn (no state)
    sidebar/
      mod.rs                 — State, Msg, update, view
    feed/
      mod.rs                 — State, Msg, update, view
      density_lane.rs        — Canvas widget (80-bucket flame ribbon)
      row.rs                 — feed_row() view fn
      filter.rs              — filter_bar() view fn
    inspector/
      mod.rs                 — State, Msg, update, view + tab router
      overview.rs            — overview_tab() view fn
      request.rs             — request_tab() view fn
      response.rs            — response_tab() view fn
      explain.rs             — explain_tab() view fn
      timeline.rs            — timeline_tab() view fn
      compose.rs             — compose_tab() view fn
      rules.rs               — rules_tab() view fn
      schema.rs              — schema_tab() view fn
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
    Find, FindOne, Aggregate, CountDocuments,
    InsertOne, UpdateOne, UpdateMany, DeleteOne, DeleteMany,
}

pub enum Plan {
    CollScan,
    IxScan(IndexName),
    IdHack,
    IxScanLookup(IndexName),
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

`MockSource` (in `data/mock.rs`) implements this. Cycles through 15 query templates with ±20% latency jitter, emitting ~1 query per 80ms. To use real wire capture: implement `DataSource` on a proxy type and pass it to the subscription at startup.

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

### BsonView
Recursive view function rendering `BsonDoc` as syntax-highlighted rows. Colors from `tok_*` palette fields.

### Inspector tabs
Pure functions: `fn overview_tab(q: &QueryEntry, palette: &Palette) -> Element<inspector::Msg>`

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
