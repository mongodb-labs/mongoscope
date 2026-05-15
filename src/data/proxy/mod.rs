pub mod intercept;

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use intercept::{
    new_app_name_store, new_direct_channel, new_pending_map, AppNameStore, DirectEntry,
    InterceptLayer, MongoCommand, PendingMap,
};
use mongod_proxy::{serve, Command, ExplainEvent, Proxy, Stage};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use super::{
    model::{IndexSuggestion, Op, Plan, QueryEntry, Suggestion},
    source::{DataSource, EntryStore},
    types::*,
};

pub struct ProxySource {
    pub upstream_host: String,
    pub upstream_port: u16,
    pub proxy_port: u16,
    pub next_id: Arc<AtomicU64>,
}

impl ProxySource {
    pub fn new(
        upstream_host: String,
        upstream_port: u16,
        proxy_port: u16,
        next_id: Arc<AtomicU64>,
    ) -> Self {
        Self {
            upstream_host,
            upstream_port,
            proxy_port,
            next_id,
        }
    }
}

impl DataSource for ProxySource {
    fn start(self: Box<Self>, tx: mpsc::Sender<QueryEntry>, store: EntryStore) {
        let upstream_host = self.upstream_host.clone();
        let upstream_port = self.upstream_port;
        let proxy_port = self.proxy_port;
        let next_id = self.next_id.clone();

        tokio::spawn(async move {
            let listener = match TcpListener::bind(format!("127.0.0.1:{proxy_port}")).await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("mongoscope: failed to bind proxy port {proxy_port}: {e}");
                    return;
                }
            };

            let (explain_tx, mut explain_rx) = mpsc::channel::<ExplainEvent>(1024);
            let (direct_tx, mut direct_rx) = new_direct_channel();

            let pending = new_pending_map();
            let app_name = new_app_name_store();
            let intercept = InterceptLayer::new(pending.clone(), app_name.clone(), Some(direct_tx));

            let proxy = Proxy::new(upstream_host, upstream_port, false)
                .enable_explain_with_sink(explain_tx)
                .layer(intercept);

            let convert_task = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(event) = explain_rx.recv() => {
                            let id = next_id.fetch_add(1, Ordering::Relaxed);
                            let entry = explain_to_entry(event, id, &pending, &app_name);
                            store_upsert(&store, entry.clone());
                            let _ = tx.send(entry).await;
                        }
                        Some(direct) = direct_rx.recv() => {
                            let id = next_id.fetch_add(1, Ordering::Relaxed);
                            let entry = direct_to_entry(direct, id);
                            store_upsert(&store, entry.clone());
                            let _ = tx.send(entry).await;
                        }
                        else => break,
                    }
                }
            });

            let _ = serve(listener, proxy).await;
            convert_task.abort();
        });
    }
}

fn explain_to_entry(
    event: ExplainEvent,
    id: u64,
    pending: &PendingMap,
    app_name: &AppNameStore,
) -> QueryEntry {
    let t_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let latency_ms = std::time::Duration::from(event.total.execution_time).as_millis() as u32;
    let slow = latency_ms >= 1000;

    let op = command_to_op(&event.command);

    let db = DatabaseName::try_new(event.namespace.database().as_ref().to_string())
        .unwrap_or_else(|_| DatabaseName::try_new("unknown".to_string()).unwrap());

    let coll = CollectionName::try_new(event.namespace.collection().as_ref().to_string())
        .unwrap_or_else(|_| CollectionName::try_new("unknown".to_string()).unwrap());

    let (plan, index) = classify_plan(&event.plan);

    let docs_examined = DocsExamined::new(i64::from(event.total.docs_examined).max(0) as u64);
    let docs_returned = DocsReturned::new(i64::from(event.total.n_returned).max(0) as u64);

    let conn_id = {
        let raw: i32 = event.client_request_id.into_inner();
        ConnId::new(raw.unsigned_abs())
    };

    let request_id: i32 = event.client_request_id.into_inner();
    let pending_entry = pending.lock().unwrap().remove(&request_id);

    let (filter, pipeline, update, doc, response_docs) = match pending_entry {
        Some(p) => (p.filter, p.pipeline, p.update, p.doc, p.response_docs),
        None => (None, None, None, None, vec![]),
    };

    let app = {
        let stored = app_name.lock().unwrap().clone();
        let name = stored.unwrap_or_else(|| "unknown".to_string());
        AppName::try_new(name).unwrap_or_else(|_| AppName::try_new("unknown".to_string()).unwrap())
    };

    let suggestions = build_suggestions(&plan, &filter, latency_ms);

    QueryEntry {
        id: QueryId::try_new(id).unwrap(),
        t_ms: TimestampMs::new(t_ms),
        latency_ms: LatencyMs::new(latency_ms),
        op,
        db,
        coll,
        app,
        plan,
        index,
        docs_examined: Some(docs_examined),
        docs_returned: Some(docs_returned),
        filter,
        pipeline,
        update,
        doc,
        warn: None,
        slow,
        is_system: false,
        conn_id,
        lsid: None,
        cluster_time: None,
        response_docs,
        rejected_plan_count: 0,
        suggestions,
    }
}

fn build_suggestions(
    plan: &Option<Plan>,
    filter: &Option<crate::data::model::BsonDoc>,
    latency_ms: u32,
) -> Vec<Suggestion> {
    if !matches!(plan, Some(Plan::CollScan)) {
        return vec![];
    }
    let Some(filter) = filter else {
        return vec![];
    };
    if filter.is_empty() {
        return vec![];
    }
    vec![Suggestion::CreateIndex(IndexSuggestion {
        ixscan_ms: (latency_ms / 10).min(50),
        fetch_ms: (latency_ms / 20).min(25),
        sort_ms: 0,
        limit_ms: 0,
    })]
}

fn command_to_op(cmd: &Command) -> Op {
    match cmd {
        Command::Find => Op::Find,
        Command::Aggregate => Op::Aggregate,
        Command::Count => Op::CountDocuments,
        Command::Update => Op::UpdateMany,
        Command::Delete => Op::DeleteMany,
        Command::Distinct => Op::Unknown("DISTINCT".to_string()),
        Command::FindAndModify => Op::Unknown("FINDMOD".to_string()),
        Command::Other(name) => Op::Unknown(name.as_ref().to_uppercase()),
        _ => Op::Unknown("UNKNOWN".to_string()),
    }
}

fn classify_plan(node: &mongod_proxy::PlanNode) -> (Option<Plan>, Option<IndexName>) {
    match &node.stage {
        Stage::Collscan | Stage::SbeScan => (Some(Plan::CollScan), None),
        Stage::Ixscan | Stage::SbeIxscan | Stage::SbeIxseek | Stage::DistinctScan => {
            let idx = node
                .index_name
                .as_ref()
                .and_then(|n| IndexName::try_new(n.as_ref().to_string()).ok());
            let plan = idx.clone().map(Plan::IxScan).unwrap_or_else(|| {
                Plan::IxScan(IndexName::try_new("unknown".to_string()).unwrap())
            });
            (Some(plan), idx)
        }
        Stage::ExpressIxscan | Stage::ExpressClusteredIxscan => (Some(Plan::IdHack), None),
        _ => {
            for child in &node.children {
                let result = classify_plan(child);
                if result.0.is_some() {
                    return result;
                }
            }
            (None, None)
        }
    }
}

/// Spawn a proxy for MCP use: binds port, starts proxy + drain task, returns bound port + abort handle.
pub async fn spawn_proxy(
    upstream_host: String,
    upstream_port: u16,
    proxy_port: u16,
    entries: EntryStore,
    next_id: Arc<AtomicU64>,
) -> Result<(u16, tokio::task::AbortHandle), String> {
    let listener = TcpListener::bind(format!("127.0.0.1:{proxy_port}"))
        .await
        .map_err(|e| format!("failed to bind proxy port {proxy_port}: {e}"))?;
    let bound_port = listener.local_addr().unwrap().port();

    let (explain_tx, mut explain_rx) = mpsc::channel::<ExplainEvent>(1024);
    let (direct_tx, mut direct_rx) = new_direct_channel();

    let pending = new_pending_map();
    let app_name = new_app_name_store();
    let intercept = InterceptLayer::new(pending.clone(), app_name.clone(), Some(direct_tx));

    let proxy = Proxy::new(upstream_host, upstream_port, false)
        .enable_explain_with_sink(explain_tx)
        .layer(intercept);

    let task = tokio::spawn(async move {
        let drain = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(event) = explain_rx.recv() => {
                        let id = next_id.fetch_add(1, Ordering::Relaxed);
                        let entry = explain_to_entry(event, id, &pending, &app_name);
                        store_upsert(&entries, entry);
                    }
                    Some(direct) = direct_rx.recv() => {
                        let id = next_id.fetch_add(1, Ordering::Relaxed);
                        let entry = direct_to_entry(direct, id);
                        store_upsert(&entries, entry);
                    }
                    else => break,
                }
            }
        });
        let _ = serve(listener, proxy).await;
        drain.abort();
    });
    let abort_handle = task.abort_handle();

    Ok((bound_port, abort_handle))
}

/// Upsert into EntryStore: replace existing entry with same conn_id (explain enrichment)
/// or append. Caps at 10,000 entries.
fn store_upsert(store: &EntryStore, entry: QueryEntry) {
    let conn_id = entry.conn_id;
    let mut guard = store.lock().unwrap();
    if let Some(pos) = guard.iter().position(|e| e.conn_id == conn_id) {
        guard[pos] = entry;
    } else {
        guard.push_back(entry);
        if guard.len() > 10_000 {
            guard.pop_front();
        }
    }
}

fn direct_to_entry(direct: DirectEntry, id: u64) -> QueryEntry {
    let db = DatabaseName::try_new(direct.db)
        .unwrap_or_else(|_| DatabaseName::try_new("unknown".to_string()).unwrap());
    let coll = CollectionName::try_new(direct.coll)
        .unwrap_or_else(|_| CollectionName::try_new("unknown".to_string()).unwrap());
    let app_str = direct.app_name.unwrap_or_else(|| "unknown".to_string());
    let app = AppName::try_new(app_str)
        .unwrap_or_else(|_| AppName::try_new("unknown".to_string()).unwrap());
    let is_system = direct.command.is_system();
    let op = direct.command.to_op();
    let slow = direct.latency_ms >= 1000;
    let conn_id = ConnId::new(direct.request_id.unsigned_abs());

    QueryEntry {
        id: QueryId::try_new(id).unwrap(),
        t_ms: TimestampMs::new(direct.start_ms),
        latency_ms: LatencyMs::new(direct.latency_ms),
        op,
        db,
        coll,
        app,
        plan: None,
        index: None,
        docs_examined: None,
        docs_returned: None,
        filter: direct.filter,
        pipeline: direct.pipeline,
        update: direct.update,
        doc: direct.doc,
        warn: None,
        slow,
        is_system,
        conn_id,
        lsid: None,
        cluster_time: None,
        response_docs: vec![],
        rejected_plan_count: 0,
        suggestions: vec![],
    }
}

/// Parse a MongoDB URI into (host, port).
pub fn parse_mongo_uri(uri: &str) -> Result<(String, u16), String> {
    let rest = uri
        .trim_start_matches("mongodb+srv://")
        .trim_start_matches("mongodb://");

    let rest = if let Some(at) = rest.rfind('@') {
        &rest[at + 1..]
    } else {
        rest
    };

    let hostport = rest.split('/').next().unwrap_or(rest);
    let hostport = hostport.split('?').next().unwrap_or(hostport);

    if let Some(colon) = hostport.rfind(':') {
        let host = hostport[..colon]
            .trim_matches('[')
            .trim_matches(']')
            .to_string();
        let port = hostport[colon + 1..]
            .parse::<u16>()
            .map_err(|_| format!("invalid port in URI: {uri}"))?;
        if host.is_empty() {
            return Err(format!("empty host in URI: {uri}"));
        }
        Ok((host, port))
    } else {
        let host = hostport.to_string();
        if host.is_empty() {
            return Err(format!("could not parse host from URI: {uri}"));
        }
        Ok((host, 27017))
    }
}
