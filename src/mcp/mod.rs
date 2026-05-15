use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use indexmap::IndexMap;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};

use crate::data::{
    model::{Op, QueryEntry, Suggestion},
    proxy::spawn_proxy,
    source::EntryStore,
};

// ── Connection store ──────────────────────────────────────────────────────────

pub struct ConnectionRecord {
    pub id: u64,
    pub name: String,
    pub upstream_uri: String,
    pub proxy_port: u16,
    pub entries: EntryStore,
    /// None for GUI-managed connections (lifecycle controlled by subscription).
    pub abort_handle: Option<tokio::task::AbortHandle>,
}

pub type ConnectionStore = Arc<Mutex<IndexMap<u64, ConnectionRecord>>>;

pub fn new_connection_store() -> ConnectionStore {
    Arc::new(Mutex::new(IndexMap::new()))
}

// ── Parameter types ───────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddConnectionParams {
    /// MongoDB URI to intercept (e.g. mongodb://localhost:27017)
    pub upstream_uri: String,
    /// Display name for this connection (defaults to the URI)
    pub name: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ConnectionIdParams {
    /// Connection ID returned by add_connection
    pub connection_id: u64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListRequestsParams {
    /// Connection ID to filter (omit to search all connections)
    pub connection_id: Option<u64>,
    /// Maximum entries to return (default: 50, max: 500)
    pub limit: Option<u32>,
    /// Only return slow queries (latency > 1000ms)
    pub slow_only: Option<bool>,
    /// Filter by database name
    pub db: Option<String>,
    /// Filter by collection name
    pub coll: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetRequestParams {
    /// Numeric query ID
    pub id: u64,
    /// Connection ID to search (omit to search all connections)
    pub connection_id: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct OptionalConnectionParams {
    /// Connection ID to scope this operation (omit to include all connections)
    pub connection_id: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ApplySuggestionParams {
    /// Numeric query ID (from get_request or list_requests)
    pub request_id: u64,
    /// Suggestion index shown in get_request output (0-based)
    pub suggestion_id: u32,
    /// Connection ID to search (omit to search all connections)
    pub connection_id: Option<u64>,
}

// ── Handler ───────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct MongoscopeMcp {
    connections: ConnectionStore,
    next_id: Arc<AtomicU64>,
    #[allow(dead_code)] // used by #[tool_router] macro expansion
    tool_router: ToolRouter<MongoscopeMcp>,
}

#[tool_router]
impl MongoscopeMcp {
    pub fn new(connections: ConnectionStore, next_id: Arc<AtomicU64>) -> Self {
        Self {
            connections,
            next_id,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Add a MongoDB connection and immediately start intercepting its traffic. Returns the proxy connection string your application should connect to instead of the real MongoDB URI."
    )]
    async fn add_connection(
        &self,
        Parameters(p): Parameters<AddConnectionParams>,
    ) -> Result<CallToolResult, McpError> {
        use crate::data::proxy::parse_mongo_uri;

        let (upstream_host, upstream_port) = match parse_mongo_uri(&p.upstream_uri) {
            Ok(v) => v,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error: {}",
                    e
                ))]));
            }
        };

        let name = p.name.clone().unwrap_or_else(|| p.upstream_uri.clone());
        let entries: EntryStore = Arc::new(Mutex::new(VecDeque::new()));

        let (proxy_port, abort_handle) = match spawn_proxy(
            upstream_host,
            upstream_port,
            0,
            entries.clone(),
            self.next_id.clone(),
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error starting proxy: {}",
                    e
                ))]));
            }
        };

        let conn_id = {
            let mut guard = self.connections.lock().unwrap();
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            guard.insert(
                id,
                ConnectionRecord {
                    id,
                    name: name.clone(),
                    upstream_uri: p.upstream_uri.clone(),
                    proxy_port,
                    entries,
                    abort_handle: Some(abort_handle),
                },
            );
            id
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Connection added (ID: {conn_id}).\nName: {name}\nProxy connection string: mongodb://127.0.0.1:{proxy_port}/\n\nPoint your application at the proxy URI instead of {} to start capturing traffic.",
            p.upstream_uri
        ))]))
    }

    #[tool(description = "List all active intercepted MongoDB connections.")]
    async fn list_connections(&self) -> Result<CallToolResult, McpError> {
        let conn_data: Vec<(u64, String, String, u16, EntryStore)> = {
            let guard = self.connections.lock().unwrap();
            guard
                .values()
                .map(|c| {
                    (
                        c.id,
                        c.name.clone(),
                        c.upstream_uri.clone(),
                        c.proxy_port,
                        c.entries.clone(),
                    )
                })
                .collect()
        };

        if conn_data.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No connections. Use add_connection to start intercepting traffic.",
            )]));
        }

        let mut lines = vec![format!("{} connection(s):\n", conn_data.len())];
        for (id, name, uri, port, entries) in conn_data {
            let count = entries.lock().unwrap().len();
            lines.push(format!(
                "[{id}] {name} → {uri} | proxy: mongodb://127.0.0.1:{port}/ | {count} requests captured",
            ));
        }
        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(description = "Remove an intercepted connection and stop its proxy.")]
    async fn remove_connection(
        &self,
        Parameters(p): Parameters<ConnectionIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let removed = {
            let mut guard = self.connections.lock().unwrap();
            guard.shift_remove(&p.connection_id)
        };
        match removed {
            Some(c) => {
                if let Some(h) = c.abort_handle {
                    h.abort();
                }
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Connection {} ({}) removed.",
                    c.id, c.name
                ))]))
            }
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "No connection with ID {}. Use list_connections to see available connections.",
                p.connection_id
            ))])),
        }
    }

    #[tool(
        description = "Get the proxy connection string for a specific connection. Point your application at this URI to have its traffic intercepted."
    )]
    async fn get_connection_string(
        &self,
        Parameters(p): Parameters<ConnectionIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let data: Option<(String, u16)> = {
            let guard = self.connections.lock().unwrap();
            guard
                .get(&p.connection_id)
                .map(|c| (c.upstream_uri.clone(), c.proxy_port))
        };
        match data {
            Some((uri, port)) => Ok(CallToolResult::success(vec![Content::text(format!(
                "mongodb://127.0.0.1:{port}/\n\nUse this instead of {uri} to have traffic captured by Mongoscope.",
            ))])),
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "No connection with ID {}. Use list_connections to see available connections.",
                p.connection_id
            ))])),
        }
    }

    #[tool(
        description = "List recently captured MongoDB requests. Supports filtering by connection, latency, database, and collection."
    )]
    async fn list_requests(
        &self,
        Parameters(p): Parameters<ListRequestsParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = p.limit.unwrap_or(50).min(500) as usize;
        let slow_only = p.slow_only.unwrap_or(false);
        let entries = self.collect_entries(p.connection_id).await;

        let filtered: Vec<&QueryEntry> = entries
            .iter()
            .rev()
            .filter(|e| {
                if slow_only && !e.slow {
                    return false;
                }
                if let Some(db) = &p.db {
                    if e.db.as_str() != db {
                        return false;
                    }
                }
                if let Some(coll) = &p.coll {
                    if e.coll.as_str() != coll {
                        return false;
                    }
                }
                true
            })
            .take(limit)
            .collect();

        if filtered.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No requests captured yet. Ensure your application is connected to the proxy connection string.",
            )]));
        }

        let mut lines = vec![format!(
            "Showing {} of {} total captured (newest first):\n",
            filtered.len(),
            entries.len()
        )];
        for e in filtered {
            lines.push(format_entry_summary(e));
        }
        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(
        description = "Get full details of a specific MongoDB request by its numeric ID, including index suggestions with their suggestion IDs."
    )]
    async fn get_request(
        &self,
        Parameters(p): Parameters<GetRequestParams>,
    ) -> Result<CallToolResult, McpError> {
        let entries = self.collect_entries(p.connection_id).await;
        match entries.iter().find(|e| e.id.into_inner() == p.id) {
            Some(e) => Ok(CallToolResult::success(vec![Content::text(
                format_entry_detail(e),
            )])),
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "No request found with ID {}. Use list_requests to see available IDs.",
                p.id
            ))])),
        }
    }

    #[tool(
        description = "Get all index optimization recommendations. Shows COLLSCAN queries with missing indexes and suggested index keys."
    )]
    async fn get_recommendations(
        &self,
        Parameters(p): Parameters<OptionalConnectionParams>,
    ) -> Result<CallToolResult, McpError> {
        let entries = self.collect_entries(p.connection_id).await;
        let with_suggestions: Vec<&QueryEntry> = entries
            .iter()
            .filter(|e| !e.suggestions.is_empty())
            .collect();

        if with_suggestions.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No index recommendations found. Either no COLLSCAN queries captured yet, or all queries are using indexes.",
            )]));
        }

        let mut lines = vec![format!(
            "{} request(s) have index recommendations:\n",
            with_suggestions.len()
        )];
        for e in &with_suggestions {
            lines.push(format_entry_summary(e));
            for (i, s) in e.suggestions.iter().enumerate() {
                match s {
                    Suggestion::CreateIndex(idx) => {
                        let keys = index_spec_from_entry(e)
                            .unwrap_or_else(|| "(no filter captured)".to_string());
                        lines.push(format!(
                            "  [suggestion {}] createIndex({}) on {}.{} — estimated: IXSCAN {}ms + FETCH {}ms + SORT {}ms (currently {}ms)",
                            i, keys, e.db, e.coll,
                            idx.ixscan_ms, idx.fetch_ms, idx.sort_ms,
                            e.latency_ms.into_inner(),
                        ));
                    }
                }
            }
        }
        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(
        description = "Clear all captured requests for one or all connections. Useful to reset state before a test run."
    )]
    async fn clear_requests(
        &self,
        Parameters(p): Parameters<OptionalConnectionParams>,
    ) -> Result<CallToolResult, McpError> {
        let stores: Vec<(u64, EntryStore)> = {
            let guard = self.connections.lock().unwrap();
            match p.connection_id {
                Some(id) => match guard.get(&id) {
                    Some(c) => vec![(id, c.entries.clone())],
                    None => {
                        return Ok(CallToolResult::success(vec![Content::text(format!(
                            "No connection with ID {id}.",
                        ))]));
                    }
                },
                None => guard.values().map(|c| (c.id, c.entries.clone())).collect(),
            }
        };

        for (_, store) in &stores {
            store.lock().unwrap().clear();
        }

        Ok(CallToolResult::success(vec![Content::text(
            match p.connection_id {
                Some(id) => format!("Cleared requests for connection {id}."),
                None => format!("Cleared requests for all {} connection(s).", stores.len()),
            },
        )]))
    }

    #[tool(
        description = "Get aggregate statistics for captured requests: total count, slow queries, and breakdown by operation type."
    )]
    async fn get_stats(
        &self,
        Parameters(p): Parameters<OptionalConnectionParams>,
    ) -> Result<CallToolResult, McpError> {
        let entries = self.collect_entries(p.connection_id).await;

        if entries.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No requests captured yet.",
            )]));
        }

        let total = entries.len();
        let slow = entries.iter().filter(|e| e.slow).count();
        let with_suggestions = entries.iter().filter(|e| !e.suggestions.is_empty()).count();

        let mut op_counts: IndexMap<String, usize> = IndexMap::new();
        for e in &entries {
            *op_counts.entry(e.op.label().to_string()).or_insert(0) += 1;
        }

        let mut lines = vec![
            format!("Total captured   : {total}"),
            format!("Slow (>1000ms)   : {slow}"),
            format!("With suggestions : {with_suggestions}"),
            String::new(),
            "By operation type:".to_string(),
        ];
        for (op, count) in &op_counts {
            lines.push(format!("  {op}: {count}"));
        }

        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(
        description = "List all unique database.collection namespaces seen in captured traffic, with query counts."
    )]
    async fn list_namespaces(
        &self,
        Parameters(p): Parameters<OptionalConnectionParams>,
    ) -> Result<CallToolResult, McpError> {
        let entries = self.collect_entries(p.connection_id).await;

        let mut namespaces: IndexMap<String, usize> = IndexMap::new();
        for e in &entries {
            *namespaces
                .entry(format!("{}.{}", e.db, e.coll))
                .or_insert(0) += 1;
        }

        if namespaces.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No namespaces seen yet.",
            )]));
        }

        let mut lines = vec![format!("{} namespace(s):\n", namespaces.len())];
        for (ns, count) in &namespaces {
            lines.push(format!("  {ns} — {count} queries"));
        }
        Ok(CallToolResult::success(vec![Content::text(
            lines.join("\n"),
        )]))
    }

    #[tool(
        description = "Get the mongosh createIndex command for a specific index suggestion. Use get_request to find request and suggestion IDs."
    )]
    async fn apply_suggestion(
        &self,
        Parameters(p): Parameters<ApplySuggestionParams>,
    ) -> Result<CallToolResult, McpError> {
        let entries = self.collect_entries(p.connection_id).await;

        let entry = match entries.iter().find(|e| e.id.into_inner() == p.request_id) {
            Some(e) => e,
            None => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "No request found with ID {}. Use list_requests to see available IDs.",
                    p.request_id
                ))]));
            }
        };

        match entry.suggestions.get(p.suggestion_id as usize) {
            Some(Suggestion::CreateIndex(_)) => {
                match index_spec_from_entry(entry) {
                    Some(keys) => {
                        let db = entry.db.as_str();
                        let coll = entry.coll.as_str();
                        Ok(CallToolResult::success(vec![Content::text(format!(
                            "Run in mongosh against the `{db}` database:\n\ndb.{coll}.createIndex({keys})\n\nOr with explicit database:\n\ndb.getSiblingDB(\"{db}\").{coll}.createIndex({keys})"
                        ))]))
                    }
                    None => Ok(CallToolResult::success(vec![Content::text(
                        "Cannot generate createIndex command: no filter data was captured for this request. \
                         Ensure the proxy intercepted the request before generating suggestions."
                            .to_string(),
                    )])),
                }
            }
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "No suggestion with index {} for request {}. Use get_request to see available suggestions.",
                p.suggestion_id, p.request_id
            ))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for MongoscopeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("mongoscope", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Mongoscope captures and analyzes MongoDB wire-protocol traffic via a transparent proxy. \
                 Workflow: (1) add_connection with your MongoDB URI to start intercepting, \
                 (2) use get_connection_string to get the proxy URI for your application, \
                 (3) run your workload, \
                 (4) analyze traffic with list_requests, get_request, get_recommendations, get_stats, list_namespaces, \
                 (5) use apply_suggestion to get the ready-to-run createIndex command for any recommendation. \
                 Multiple connections can be managed simultaneously."
                    .to_string(),
            )
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

impl MongoscopeMcp {
    async fn collect_entries(&self, connection_id: Option<u64>) -> Vec<QueryEntry> {
        let stores: Vec<EntryStore> = {
            let guard = self.connections.lock().unwrap();
            match connection_id {
                Some(id) => guard
                    .get(&id)
                    .map(|c| vec![c.entries.clone()])
                    .unwrap_or_default(),
                None => guard.values().map(|c| c.entries.clone()).collect(),
            }
        };

        let mut result = Vec::new();
        for store in stores {
            let entries = store.lock().unwrap();
            result.extend(entries.iter().cloned());
        }
        if connection_id.is_none() {
            result.sort_by_key(|e| e.id.into_inner());
        }
        result
    }
}

fn index_spec_from_entry(e: &QueryEntry) -> Option<String> {
    let filter = e.filter.as_ref()?;
    if filter.is_empty() {
        return None;
    }
    let pairs: Vec<String> = filter.keys().map(|k| format!("\"{k}\": 1")).collect();
    Some(format!("{{ {} }}", pairs.join(", ")))
}

fn format_entry_summary(e: &QueryEntry) -> String {
    let plan_label = match &e.plan {
        Some(p) => p.label(),
        None => "—".to_string(),
    };
    let slow_marker = if e.slow { " SLOW" } else { "" };
    let suggestion_marker = if !e.suggestions.is_empty() {
        " [index suggestion]"
    } else {
        ""
    };
    format!(
        "[{}] {} | {} | {}.{} | {}ms | {}{}{}",
        e.id.into_inner(),
        e.op.label(),
        e.app,
        e.db,
        e.coll,
        e.latency_ms.into_inner(),
        plan_label,
        slow_marker,
        suggestion_marker,
    )
}

fn format_entry_detail(e: &QueryEntry) -> String {
    let mut lines = vec![
        format!("=== Request {} ===", e.id.into_inner()),
        format!("Operation : {}", e.op.label()),
        format!("Namespace : {}.{}", e.db, e.coll),
        format!("App       : {}", e.app),
        format!(
            "Latency   : {}ms{}",
            e.latency_ms.into_inner(),
            if e.slow { " (SLOW)" } else { "" }
        ),
    ];

    if let Some(plan) = &e.plan {
        lines.push(format!("Plan      : {}", plan.label()));
    }
    if let Some(idx) = &e.index {
        lines.push(format!("Index     : {idx}"));
    }
    if let Some(dr) = &e.docs_returned {
        lines.push(format!("Returned  : {} docs", dr.into_inner()));
    }
    if let Some(de) = &e.docs_examined {
        lines.push(format!("Examined  : {} docs", de.into_inner()));
    }

    let filter_label = match &e.op {
        Op::Aggregate => "Pipeline",
        Op::InsertOne => "Document",
        Op::UpdateOne | Op::UpdateMany => "Update",
        _ => "Filter",
    };
    if let Some(f) = &e.filter {
        lines.push(format!(
            "{filter_label}   : {}",
            serde_json::to_string(f).unwrap_or_default()
        ));
    }
    if let Some(p) = &e.pipeline {
        lines.push(format!(
            "Pipeline  : {}",
            serde_json::to_string(p).unwrap_or_default()
        ));
    }
    if let Some(w) = &e.warn {
        lines.push(format!("Warning   : {w}"));
    }

    if !e.suggestions.is_empty() {
        lines.push("Suggestions:".to_string());
        for (i, s) in e.suggestions.iter().enumerate() {
            match s {
                Suggestion::CreateIndex(idx) => {
                    let keys = index_spec_from_entry(e)
                        .unwrap_or_else(|| "(no filter captured)".to_string());
                    lines.push(format!(
                        "  [{}] createIndex({keys}) on {}.{} — estimated gain: ~{}ms (IXSCAN {}ms + FETCH {}ms + SORT {}ms)",
                        i, e.db, e.coll,
                        idx.ixscan_ms + idx.fetch_ms + idx.sort_ms,
                        idx.ixscan_ms, idx.fetch_ms, idx.sort_ms,
                    ));
                    lines.push(format!(
                        "       → apply_suggestion request_id={} suggestion_id={i}",
                        e.id.into_inner()
                    ));
                }
            }
        }
    }

    lines.join("\n")
}
