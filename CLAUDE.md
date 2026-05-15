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
- `serde 1` / `serde_json 1` — serialization
- `bson 2` — BSON document parsing
- `mongod-proxy` (private git) — transparent TCP proxy with explain events and Tower layer interception
- `rmcp 1.7` — MCP server (stdio + streamable HTTP transports)
- `axum 0.8` — HTTP server for GUI MCP endpoint
- `clap 4` — CLI argument parsing (`--mcp` flag for headless stdio mode)

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
├── main.rs                        # App entry, layout composition, Msg enum, subscriptions, MCP HTTP server
├── theme.rs                       # Palette (Copy), fonts, Density (Compact/Comfy), dark/light
├── mcp/
│   └── mod.rs                     # MongoscopeMcp handler, ConnectionStore, tool implementations
├── data/
│   ├── mod.rs
│   ├── types.rs                   # Newtypes: QueryId, TimestampMs, LatencyMs, DatabaseName, etc.
│   ├── model.rs                   # QueryEntry, Op, Plan, Suggestion, BsonDoc/BsonVal
│   ├── source.rs                  # DataSource trait + EntryStore (Arc<Mutex<VecDeque<QueryEntry>>>)
│   └── proxy/
│       ├── mod.rs                 # ProxySource (DataSource impl), spawn_proxy, parse_mongo_uri
│       └── intercept.rs           # Tower Layer: captures filter/pipeline/update/response_docs/app_name
└── ui/
    ├── mod.rs
    ├── dialog.rs                  # Generic dialog wrapper
    ├── statusbar.rs               # Bottom bar: ops/sec, total count, slow count
    ├── mcp_panel.rs               # MCP server toggle (port 3717), config copy
    ├── topbar/                    # Logo, menu bar, URI display, MCP btn, theme/density toggles
    ├── sidebar/
    │   ├── connections.rs         # Connection list + 2-step add-connection dialog
    │   ├── connection_state.rs    # Per-connection state (name, color, capturing, entry_store)
    │   ├── databases.rs           # DB/collection tree built from captured traffic
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
    │       ├── compose.rs         # Query text editor (hidden, issue #33)
    │       ├── rules.rs           # Pattern rules (hidden, issue #32)
    │       └── schema.rs          # Field types, coverage %, sample values (hidden, issue #28)
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

## Data pipeline

No mock data. All entries come from real MongoDB wire protocol via the transparent proxy.

**Flow:**
1. User adds connection (URI → `DialogDone`)
2. `ConnectionState` created with its own `EntryStore`; `ConnectionRecord` registered in `ConnectionStore`
3. Subscription fires `ProxySource::start(tx, entry_store)` — binds proxy port, intercepts traffic
4. `InterceptLayer` (Tower) captures filter/pipeline/update/response_docs/app_name per request
5. `ExplainEvent` arrives → `explain_to_entry()` merges intercepted data → writes to both mpsc (GUI feed) and `EntryStore` (MCP)
6. MCP server reads same `EntryStore` — GUI and MCP see identical data

**EntryStore** = `Arc<Mutex<VecDeque<QueryEntry>>>` capped at 10,000 entries per connection.

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

4 visible tabs: Overview, Request, Response, Explain.
3 hidden tabs (infrastructure kept, not shown in tab bar): Compose (#33), Rules (#32), Schema (#28).

Explain tab: shows plan type, docs examined/returned, index used. If COLLSCAN + filter captured, shows `CreateIndex` suggestion with copyable shell command and before/after plan comparison.

## MCP panel

Overlay modal. States: Stopped → Starting → Running. Start button spawns a real axum HTTP server (`StreamableHttpService`) on a random port; port shown in config snippet once running. Shows ready-to-paste config for Claude/MCP clients. Stop button aborts the server task.

Headless mode: `mongoscope --mcp` runs a standalone stdio MCP server (no GUI).

## App-level state (App struct in main.rs)

- `theme: Theme` (Dark/Light)
- `density: Density` (Compact 28px / Comfy 34px)
- `sidebar_width: f32` (draggable)
- `inspector_panel: InspectorPanel` (Closed / Open { height } / Maximized)
- `sidebar: SidebarState`
- `inspector: InspectorState`
- `mcp_panel: McpPanelState`
- `connection_store: mcp::ConnectionStore` — shared with MCP server; keyed by connection ID
- `mcp_next_id: Arc<AtomicU64>` — global monotonic query ID across all connections
- `mcp_abort: Option<tokio::task::AbortHandle>` — abort handle for running MCP HTTP server

Subscriptions: per-connection proxy data stream (one subscription per live connection), drag events for sidebar/inspector resize handles.

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
