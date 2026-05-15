# Mongoscope

Rust desktop MongoDB query debugger and traffic inspector. Built with iced 0.13 (Elm-style, `iced::application()`).

## Design reference

HTML/JSX design files live in `~/temp/mongoscope/` (Mongoscope.html, sidebar.jsx, feed.jsx, inspector.jsx, etc.).

**If that directory is missing or empty**, ask the user for the Anthropic design viewer link so you can re-fetch and extract it there.

Always follow the design. Do not deviate without explicit user permission. When permission is granted, log it here under "Approved deviations" so you don't need to ask again.

## Approved deviations

_(none yet)_

## Stack

- `iced 0.13` — features: canvas, tokio, lazy
- `nutype 0.5` — newtypes for QueryId, Timestamp, etc.
- `indexmap 2` — ordered maps
- `tokio 1` — features: time, rt-multi-thread, macros, sync
- `rand 0.8` — mock data generation (SmallRng, seed 42)
- `serde 1` — serialization
- Mock data source at 2–3 entries/sec (`src/data/mock/`)

## Keeping CLAUDE.md up to date

When adding new files, modules, data types, UI components, or changing key patterns: update the relevant section in this file. This is the primary context source for AI sessions — stale docs waste time.

## Before marking work done

Always run these in order and confirm all pass:

```
cargo fmt
cargo build
cargo test
cargo clippy
```

## File structure

```
src/
├── main.rs                        # App entry, layout composition, Msg enum, subscriptions (~1700 lines)
├── theme.rs                       # Palette (Copy), fonts, Density (Compact/Comfy), dark/light
├── data/
│   ├── mod.rs
│   ├── types.rs                   # Newtypes: QueryId, TimestampMs, LatencyMs, DatabaseName, etc.
│   ├── model.rs                   # QueryEntry, CollectionInfo, SchemaField, Suggestion
│   ├── source.rs                  # DataSource trait
│   └── mock/
│       ├── mod.rs                 # MockSource — streams entries via tokio channel
│       ├── templates.rs           # 12 query pattern templates
│       └── docs.rs                # Response document generation
└── ui/
    ├── mod.rs
    ├── dialog.rs                  # Generic dialog wrapper
    ├── statusbar.rs               # Bottom bar: ops/sec, total count, slow count
    ├── mcp_panel.rs               # MCP server toggle (port 3717), config copy
    ├── topbar/                    # Logo, menu bar, URI display, MCP btn, theme/density toggles
    ├── sidebar/
    │   ├── connections.rs         # Connection list + 2-step add-connection dialog
    │   ├── connection_state.rs    # Per-connection state (name, color, capturing)
    │   ├── databases.rs           # DB/collection tree with doc counts, sizes, index counts
    │   ├── clients.rs             # App name filter with color dots
    │   └── filters.rs             # Quick-toggle preset filters
    ├── feed/
    │   ├── mod.rs                 # Main query table, scroll state, 2000-entry ring
    │   ├── buckets.rs             # 80-bucket density histogram
    │   ├── density_lane.rs        # Visual density display
    │   ├── table/                 # Row rendering
    │   └── filter/                # Search bar + preset pill buttons
    ├── inspector/
    │   ├── mod.rs                 # Panel container, resize drag, maximize
    │   ├── header.rs              # Tab navigation
    │   └── tabs/
    │       ├── overview.rs        # Latency hero, namespace, op type, metrics
    │       ├── request.rs         # Filter/pipeline/update as BSON tree
    │       ├── response.rs        # Returned documents
    │       ├── explain.rs         # Plan analysis, index suggestion, before/after
    │       ├── compose.rs         # Query text editor
    │       ├── rules.rs           # Pattern rules (warn/block/highlight)
    │       └── schema.rs          # Field types, coverage %, sample values
    └── widgets/
        ├── op_badge.rs            # Operation type badge (colored)
        ├── plan_chip.rs           # IXSCAN / COLLSCAN / IDHACK chip
        ├── latency_bar.rs         # Visual latency indicator
        ├── appdot.rs              # Client app color dot
        ├── mini_card.rs           # Small labeled info card
        ├── kv_grid.rs             # Key-value pair layout
        ├── bson_view.rs           # Syntax-highlighted BSON tree
        ├── flame_row.rs           # Proportional bar visualization
        ├── schema_row.rs          # Field type + coverage display
        ├── icon_btn.rs            # Styled icon button
        ├── ghost_btn.rs           # Transparent button
        ├── section_header.rs      # Section titles
        ├── warn_banner.rs         # Warning message banner
        └── toggle.rs              # Pill toggle switch
```

## Data model

```rust
QueryEntry {
    id: QueryId,                    // u64 newtype
    t_ms: TimestampMs,              // u64
    latency_ms: LatencyMs,          // u32
    op: Op,                         // Find | FindOne | Aggregate | Count | Insert | Update | Delete | Unknown
    db: DatabaseName,
    coll: CollectionName,
    app: AppName,
    plan: Option<Plan>,             // CollScan | IxScan(IndexName) | IdHack | IxScanLookup
    index: Option<IndexName>,
    docs_examined: Option<DocsExamined>,
    docs_returned: Option<DocsReturned>,
    filter: Option<BsonDoc>,
    pipeline: Option<Vec<BsonDoc>>,
    update: Option<BsonDoc>,
    doc: Option<BsonDoc>,
    warn: Option<String>,
    slow: bool,                     // true if latency_ms > 1000
    response_docs: Vec<BsonDoc>,
    rejected_plan_count: u8,
    suggestions: Vec<Suggestion>,   // CreateIndex { keys, name, shell_cmd }
    conn_id, lsid, cluster_time,
}
```

**BSON types rendered**: Null, Bool, Int, Float, ObjectId, IsoDate, Timestamp, NumberLong, Array, Doc

## Mock data catalog

Database: `shop`. Collections:

| Collection | Docs | Size | Indexes |
|------------|------|------|---------|
| orders | 2.4M | 8.4GB | 7 |
| products | 184K | 412MB | 5 |
| users | 892K | 1.8GB | 6 |
| carts | 45K | 89MB | 3 |
| sessions | 12M | 4.2GB | 4 |
| reviews | 334K | 892MB | 4 |
| inventory | 28K | 67MB | 5 |
| events | 89M | 41.2GB | 2 |

Client apps: `checkout-svc`, `catalog-api`, `analytics-worker`, `admin-portal`, `mobile-bff`

12 query templates covering: find with IXSCAN, find with COLLSCAN (slow, for demo), aggregate COLLSCAN (slow), insert, update, delete, IDHACK patterns. RNG seed 42 for reproducibility. Latency jitter 0.8–1.2×.

## Feed filter syntax

Text search tokens parsed from the search bar:
- `db:<name>` — filter by database
- `coll:<name>` — filter by collection
- `app:<name>` — filter by client app
- `slow` — only slow queries
- `collscan` — only COLLSCANs
- `suggestions` — only entries with index suggestions

Preset pills (rendered as filter chips): Slow queries (>1000ms), COLLSCANs only, With suggestions.
Kind filter: All / Read / Write / Delete.

## Inspector tabs

Default height 475px. Drag divider to resize. Maximize button for fullscreen. Only visible when an entry is selected.

7 tabs: Overview, Request, Response, Explain, Compose, Rules, Schema.

Explain tab: shows plan type, docs examined/returned, index used. If COLLSCAN detected, shows `CreateIndex` suggestion with copyable shell command and before/after plan comparison.

Schema tab: 13 fields per collection with type labels, coverage percentages, sample values.

## MCP panel

Overlay modal. Port default 3717. States: Stopped → Starting (simulated delay) → Running. Shows config snippet to paste into Claude or other MCP clients.

## App-level state (App struct in main.rs)

- `theme: Theme` (Dark/Light)
- `density: Density` (Compact 28px / Comfy 34px)
- `sidebar_width: f32` (draggable)
- `inspector_height: f32` (draggable, default 475px)
- `sidebar: SidebarState`
- `feed: FeedState`
- `inspector: InspectorState`
- `mcp_panel: McpPanelState`
- `topbar: TopbarState`

Subscriptions: keyboard (`s` start, `d` stop mock), mock data stream, drag events for resize handles.

## Key patterns

- `Palette: Copy` — always pass by value, extract `Color` fields before `'static` closures
- `lazy(dep, |_| ...)` — memoize expensive subtrees; dep must change when content changes
- Views return `Element<'a, Msg>` when borrowing entry data, `Element<'static, Msg>` when using only owned/Copy data
- Style closures must be `'static` — never capture `&Palette` directly
- Feed stores max 2,000 entries; auto-scroll locks when user selects entry

## Design assets

`_designs/` contains HTML/PNG mockups:
- `suggest-index/` — index suggestion UI, explain improvements
- `connection-dialog/` — both steps of the add-connection flow
- `final-design.{html,png}`, `explain-improvements.{html,png}`, etc.
