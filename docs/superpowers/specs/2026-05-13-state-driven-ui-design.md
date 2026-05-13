# State-Driven UI: Remove All Hardcoded Data

**Goal:** Every value shown in the UI flows from a model field. No hardcoded database names, connection IDs, document content, schemas, or counts in view code.

---

## 1. Model Changes (`src/data/model.rs` + `src/data/types.rs`)

### New newtype in `types.rs`
```rust
#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct ConnId(u32);
```

### New fields on `QueryEntry`
```rust
pub conn_id: ConnId,
pub lsid: Option<String>,
pub cluster_time: Option<String>,
pub response_docs: Vec<BsonDoc>,
pub rejected_plan_count: u8,
```

- `conn_id` — replaces `1420 + (entry.id % 40)` formula in views
- `lsid` — replaces hardcoded `"lsid: { id: UUID(\"a4b...\") }"` in request tab
- `cluster_time` — replaces hardcoded `"$clusterTime: { clusterTime: Timestamp(...) }"` in request tab
- `response_docs` — replaces inline `mock_doc()` factory in response tab
- `rejected_plan_count` — replaces hardcoded `"Rejected plans (3)"` in explain tab

### New types: `CollectionSchema`
```rust
pub struct SchemaField {
    pub name: &'static str,
    pub type_str: &'static str,
    pub samples: &'static [&'static str],
    pub coverage_pct: u8,
}

pub struct CollectionSchema {
    pub coll: CollectionName,
    pub fields: Vec<SchemaField>,
    pub sampled_docs: u32,
}
```

Add `pub fn mock_schemas() -> Vec<CollectionSchema>` — static data for orders, products, users, carts, sessions, reviews, inventory, events. Move hardcoded field/type/sample data out of `schema.rs` into here.

---

## 2. Mock Changes (`src/data/mock/`)

### New file: `mock/docs.rs`
Move `mock_doc()` logic here as:
```rust
pub fn gen_response_docs(coll: &str, op: &Op, n: usize) -> Vec<BsonDoc>
```
Returns up to `n` realistic documents based on collection name. Covers all collections in `model::collections()`, with a fallback generic doc for unknown collections.

### Update `mock/mod.rs`
Generate new fields per entry:
- `conn_id`: cycle through a pool of 20 IDs (e.g. 10001–10020) based on `id % 20`
- `lsid`: ~70% of entries get a realistic session ID string; 30% `None`
- `cluster_time`: all entries get a timestamp string derived from `t_ms`
- `response_docs`: call `gen_response_docs(coll, op, docs_returned.unwrap_or(1))`
- `rejected_plan_count`: `CollScan` → 1, `IxScan` → 2, `IdHack` → 0, others → 0

---

## 3. View Changes

### `header.rs`
Replace `text("shop.")` with `text(format!("{}.", entry.db))`.

### `overview.rs`
- Replace both `format!("shop.{}", coll)` with `format!("{}.{}", entry.db, coll)`
- Replace `1420 + (entry.id.into_inner() % 40)` with `entry.conn_id.into_inner()`

### `request.rs`
- Replace `text("$db: \"shop\"")` with `text(format!("$db: {:?}", entry.db.as_str()))`
- Replace hardcoded `lsid` line with conditional block: show only if `entry.lsid.is_some()`
- Replace hardcoded `$clusterTime` line with `entry.cluster_time` value (shown if Some)

### `response.rs`
- Remove `mock_doc()` and `build_response()` functions
- Use `entry.response_docs` directly — pass to `bson_view`
- Derive `byte_est` from `entry.response_docs` actual content size

### `explain.rs`
- Replace `"Rejected plans (3)"` with `format!("Rejected plans ({})", entry.rejected_plan_count)` — hide tab if count is 0
- Fix `ixscan_ms: u32 = 1`, `fetch_ms: u32 = 2`, `sort_after_ms: u32 = 1` → derive as fractions of `total_ms` (matching the ratios already used for before-plan stages)

### `schema.rs`
- Change signature to accept `schema: Option<&CollectionSchema>`
- Remove hardcoded field/type/sample blocks
- Render from `schema.fields`; show placeholder if `None`
- Replace `"inferred from 2,000 sampled docs"` with `format!("inferred from {} sampled docs", schema.sampled_docs)`
- Use `entry.db` for namespace display

### `compose.rs`
- Add `cluster_label: &str` parameter
- Replace `text("prod-cluster-0")` with `text(cluster_label)`
- `"shell · mongosh 2.4.0"` stays as-is — this is a UI constant (shell version doesn't vary per query)

### `timeline.rs` — DELETE
Remove file. Remove from `mod.rs` imports, `InspectorTab` enum, `InspectorTab::all()`, and tab routing match.

---

## 4. Inspector Wiring (`src/ui/inspector/mod.rs`)

- Remove `Timeline` from `InspectorTab` enum and `all()`
- Schema tab call: look up schema from `model::mock_schemas()` by `entry.coll`, pass `Option<&CollectionSchema>`
- Compose tab call: pass `cluster_label` from active `ConnectionItem.label`

The schema lookup and cluster label retrieval happen at the call site in `mod.rs`, keeping tab functions pure (no app state access inside tabs).

---

## 5. What stays hardcoded (intentionally)

| Value | Reason |
|---|---|
| `"OP_MSG"` in request/response headers | MongoDB wire protocol name — factually correct for all modern queries, not per-query data |
| Button labels (`"copy"`, `"export"`, `"↻ Replay"`, etc.) | UI chrome |
| `"shell · mongosh 2.4.0"` in compose footer | UI constant; shell version doesn't come from a query |
| Efficiency threshold labels (`"optimal (1×)"`, etc.) | UI copy |
| Phase colors in explain flame | Palette-derived, not data |

---

## Out of scope

- Real MongoDB backend wiring
- Schema inference from live data
- Rules tab interception example (cosmetic, no data model needed yet)
