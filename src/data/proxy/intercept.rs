use std::{
    collections::HashMap,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{SystemTime, UNIX_EPOCH},
};

use futures::Stream;
use mongod_proxy::{
    message::Message,
    operation::{
        op_msg::{OpMsgSection, OperationMessage},
        Operation,
    },
};
use tokio::sync::mpsc as tokio_mpsc;
use tower_layer::Layer;
use tower_service::Service;

use crate::data::model::{BsonDoc, BsonVal, Op};

pub type PendingMap = Arc<Mutex<HashMap<i32, PendingEntry>>>;
pub type AppNameStore = Arc<Mutex<Option<String>>>;
pub type DirectSender = tokio_mpsc::UnboundedSender<DirectEntry>;
pub type DirectReceiver = tokio_mpsc::UnboundedReceiver<DirectEntry>;

// ── Command classification ────────────────────────────────────────────────────

/// Every MongoDB wire-protocol command seen by the intercept layer, classified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MongoCommand {
    // ── User query commands ──────────────────────────────────────────────────
    Find,
    Aggregate,
    Insert,
    Update,
    Delete,
    FindAndModify,
    Count,
    Distinct,

    // ── User DDL commands ────────────────────────────────────────────────────
    CreateCollection,
    DropCollection,
    CreateIndexes,
    DropIndexes,
    CreateSearchIndexes,
    DropSearchIndex,
    UpdateSearchIndex,
    RenameCollection,
    DropDatabase,

    // ── System/driver commands — shown only when "sys ops" toggle is on ──────
    Explain,
    ListDatabases,
    ListCollections,
    ListIndexes,
    GetParameter,
    GetMore,
    KillCursors,
    ConnectionStatus,
    ServerStatus,
    CurrentOp,
    ReplSetGetStatus,

    // ── Connection/auth handshake — always skipped, never shown ─────────────
    Hello,
    Ping,
    WhatsMyUri,
    BuildInfo,
    EndSessions,
    SaslStart,
    SaslContinue,
    Logout,
    GetNonce,
    GetLastError,
    Authenticate,

    // ── Anything else ────────────────────────────────────────────────────────
    Other(String),
}

impl MongoCommand {
    /// Parse from the raw (already-lowercased) command name on the wire.
    pub fn from_name(name: &str) -> Self {
        match name {
            "find" => Self::Find,
            "aggregate" => Self::Aggregate,
            "insert" => Self::Insert,
            "update" => Self::Update,
            "delete" => Self::Delete,
            "findandmodify" => Self::FindAndModify,
            "count" => Self::Count,
            "distinct" => Self::Distinct,

            "create" => Self::CreateCollection,
            "drop" => Self::DropCollection,
            "createindexes" => Self::CreateIndexes,
            "dropindexes" => Self::DropIndexes,
            "createsearchindexes" => Self::CreateSearchIndexes,
            "dropsearchindex" => Self::DropSearchIndex,
            "updatesearchindex" => Self::UpdateSearchIndex,
            "renamecollection" => Self::RenameCollection,
            "dropdatabase" => Self::DropDatabase,

            "explain" => Self::Explain,
            "listdatabases" => Self::ListDatabases,
            "listcollections" => Self::ListCollections,
            "listindexes" | "listindexesnonblocking" => Self::ListIndexes,
            "getparameter" => Self::GetParameter,
            "getmore" => Self::GetMore,
            "killcursors" => Self::KillCursors,
            "connectionstatus" => Self::ConnectionStatus,
            "serverstatus" => Self::ServerStatus,
            "currentop" => Self::CurrentOp,
            "replsetgetstatus" => Self::ReplSetGetStatus,

            "hello" | "ismaster" | "isnew" => Self::Hello,
            "whatsmyuri" => Self::WhatsMyUri,
            "buildinfo" | "buildinfowithversion" => Self::BuildInfo,
            "ping" => Self::Ping,
            "endsessions" => Self::EndSessions,
            "saslstart" => Self::SaslStart,
            "saslcontinue" => Self::SaslContinue,
            "logout" => Self::Logout,
            "getnonce" => Self::GetNonce,
            "getlasterror" => Self::GetLastError,
            "authenticate" => Self::Authenticate,

            other => Self::Other(other.to_string()),
        }
    }

    /// The BSON key whose value is the target collection name (e.g. `"find"` → `"orders"`).
    /// Returns `None` for commands that don't target a single collection.
    pub fn collection_key(&self) -> Option<&str> {
        match self {
            Self::Find => Some("find"),
            Self::Aggregate => Some("aggregate"),
            Self::Insert => Some("insert"),
            Self::Update => Some("update"),
            Self::Delete => Some("delete"),
            Self::FindAndModify => Some("findandmodify"),
            Self::Count => Some("count"),
            Self::Distinct => Some("distinct"),
            Self::Explain => Some("explain"),
            Self::GetMore => Some("getmore"),
            Self::Other(name) => Some(name.as_str()),
            _ => None,
        }
    }

    /// True for commands that should never appear in the feed regardless of filters.
    pub fn is_handshake(&self) -> bool {
        matches!(
            self,
            Self::Hello
                | Self::Ping
                | Self::WhatsMyUri
                | Self::BuildInfo
                | Self::EndSessions
                | Self::SaslStart
                | Self::SaslContinue
                | Self::Logout
                | Self::GetNonce
                | Self::GetLastError
                | Self::Authenticate
        )
    }

    /// True for driver/admin commands hidden by default; revealed by "sys ops" toggle.
    /// User query commands (find/aggregate/insert/update/delete/count/distinct/findAndModify)
    /// are NOT system. Everything else — including unknown commands — is system by default.
    pub fn is_system(&self) -> bool {
        !matches!(
            self,
            Self::Find
                | Self::Aggregate
                | Self::Insert
                | Self::Update
                | Self::Delete
                | Self::FindAndModify
                | Self::Count
                | Self::Distinct
                | Self::CreateCollection
                | Self::DropCollection
                | Self::CreateIndexes
                | Self::DropIndexes
                | Self::CreateSearchIndexes
                | Self::DropSearchIndex
                | Self::UpdateSearchIndex
                | Self::RenameCollection
                | Self::DropDatabase
        )
    }

    /// Convert to the `Op` type used in `QueryEntry`.
    pub fn to_op(&self) -> Op {
        match self {
            Self::Find => Op::Find,
            Self::Aggregate => Op::Aggregate,
            Self::Insert => Op::InsertOne,
            Self::Update => Op::UpdateMany,
            Self::Delete => Op::DeleteMany,
            Self::FindAndModify => Op::UpdateOne,
            Self::Count => Op::CountDocuments,
            Self::Distinct => Op::Unknown("DISTINCT".into()),
            Self::CreateCollection => Op::Unknown("CREATE-COLL".into()),
            Self::DropCollection => Op::Unknown("DROP-COLL".into()),
            Self::CreateIndexes => Op::Unknown("CREATE-IX".into()),
            Self::DropIndexes => Op::Unknown("DROP-IX".into()),
            Self::CreateSearchIndexes => Op::Unknown("CREATE-SRCH-IX".into()),
            Self::DropSearchIndex => Op::Unknown("DROP-SRCH-IX".into()),
            Self::UpdateSearchIndex => Op::Unknown("UPD-SRCH-IX".into()),
            Self::RenameCollection => Op::Unknown("RENAME-COLL".into()),
            Self::DropDatabase => Op::Unknown("DROP-DB".into()),
            Self::Explain => Op::Unknown("EXPLAIN".into()),
            Self::ListDatabases => Op::Unknown("LIST-DBS".into()),
            Self::ListCollections => Op::Unknown("LIST-COLLS".into()),
            Self::ListIndexes => Op::Unknown("LIST-IX".into()),
            Self::GetParameter => Op::Unknown("GET-PARAM".into()),
            Self::GetMore => Op::Unknown("GETMORE".into()),
            Self::KillCursors => Op::Unknown("KILL-CURS".into()),
            Self::ConnectionStatus => Op::Unknown("CONN-STATUS".into()),
            Self::ServerStatus => Op::Unknown("SRV-STATUS".into()),
            Self::CurrentOp => Op::Unknown("CURRENT-OP".into()),
            Self::ReplSetGetStatus => Op::Unknown("REPL-STATUS".into()),
            Self::Hello
            | Self::Ping
            | Self::WhatsMyUri
            | Self::BuildInfo
            | Self::EndSessions
            | Self::SaslStart
            | Self::SaslContinue
            | Self::Logout
            | Self::GetNonce
            | Self::GetLastError
            | Self::Authenticate => Op::Unknown("HANDSHAKE".into()),
            Self::Other(name) => Op::Unknown(name.to_uppercase()),
        }
    }
}

// ── Pending / Direct entries ──────────────────────────────────────────────────

pub struct PendingEntry {
    pub filter: Option<BsonDoc>,
    pub pipeline: Option<Vec<BsonDoc>>,
    pub update: Option<BsonDoc>,
    pub doc: Option<BsonDoc>,
    pub response_docs: Vec<BsonDoc>,
    pub start_ms: u64,
    pub db: String,
    pub coll: String,
    pub command: MongoCommand,
}

/// Entry emitted directly from the intercept layer when a response arrives.
/// Carries enough data to construct a QueryEntry without waiting for an ExplainEvent.
pub struct DirectEntry {
    pub request_id: i32,
    pub command: MongoCommand,
    pub db: String,
    pub coll: String,
    pub start_ms: u64,
    pub latency_ms: u32,
    pub filter: Option<BsonDoc>,
    pub pipeline: Option<Vec<BsonDoc>>,
    pub update: Option<BsonDoc>,
    pub doc: Option<BsonDoc>,
    pub app_name: Option<String>,
}

pub fn new_direct_channel() -> (DirectSender, DirectReceiver) {
    tokio_mpsc::unbounded_channel()
}

pub fn new_pending_map() -> PendingMap {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn new_app_name_store() -> AppNameStore {
    Arc::new(Mutex::new(None))
}

// ── Tower layer ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct InterceptLayer {
    pub pending: PendingMap,
    pub app_name: AppNameStore,
    pub direct_tx: Option<DirectSender>,
}

impl InterceptLayer {
    pub fn new(
        pending: PendingMap,
        app_name: AppNameStore,
        direct_tx: Option<DirectSender>,
    ) -> Self {
        Self {
            pending,
            app_name,
            direct_tx,
        }
    }
}

impl<S> Layer<S> for InterceptLayer {
    type Service = InterceptService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        InterceptService {
            inner,
            pending: self.pending.clone(),
            app_name: self.app_name.clone(),
            direct_tx: self.direct_tx.clone(),
        }
    }
}

pub struct InterceptService<S> {
    inner: S,
    pending: PendingMap,
    app_name: AppNameStore,
    direct_tx: Option<DirectSender>,
}

impl<S: Clone> Clone for InterceptService<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            pending: self.pending.clone(),
            app_name: self.app_name.clone(),
            direct_tx: self.direct_tx.clone(),
        }
    }
}

impl<S, St, E> Service<Message> for InterceptService<S>
where
    S: Service<Message, Response = St, Error = E>,
    S::Future: Send + 'static,
    St: Stream<Item = Result<Message, E>> + Unpin + Send + 'static,
    E: Send + 'static,
{
    type Response = InterceptStream<St>;
    type Error = E;
    type Future =
        Pin<Box<dyn std::future::Future<Output = Result<InterceptStream<St>, E>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), E>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Message) -> Self::Future {
        let request_id: i32 = req.request_id.into();

        let cmd = req
            .operation
            .command_name()
            .map(|s| MongoCommand::from_name(&s.to_ascii_lowercase()))
            .unwrap_or_else(|| MongoCommand::Other("unknown".into()));

        match &cmd {
            c if c.is_handshake() => {
                // Capture app name from hello/ismaster; otherwise skip entirely.
                if matches!(c, MongoCommand::Hello) {
                    if let Some(name) = extract_app_name(&req.operation) {
                        *self.app_name.lock().unwrap() = Some(name);
                    }
                }
            }
            _ => {
                let (filter, pipeline, update, doc) = match &cmd {
                    MongoCommand::Find | MongoCommand::Count | MongoCommand::Distinct => {
                        (extract_filter(&req.operation), None, None, None)
                    }
                    MongoCommand::Aggregate => (None, extract_pipeline(&req.operation), None, None),
                    MongoCommand::Update => {
                        let (f, u) = extract_update(&req.operation);
                        (f, None, u, None)
                    }
                    MongoCommand::Delete | MongoCommand::FindAndModify => {
                        (extract_delete_filter(&req.operation), None, None, None)
                    }
                    MongoCommand::Insert => (None, None, None, extract_insert_doc(&req.operation)),
                    _ => (None, None, None, None),
                };
                let coll_key = cmd.collection_key().unwrap_or("unknown");
                let (db, coll) = extract_db_coll(&req.operation, coll_key);
                self.pending.lock().unwrap().insert(
                    request_id,
                    PendingEntry {
                        filter,
                        pipeline,
                        update,
                        doc,
                        response_docs: vec![],
                        start_ms: current_ms(),
                        db,
                        coll,
                        command: cmd,
                    },
                );
            }
        }

        let app_name = self.app_name.clone();
        let direct_tx = self.direct_tx.clone();
        let fut = self.inner.call(req);
        let pending = self.pending.clone();
        Box::pin(async move {
            let inner = fut.await?;
            Ok(InterceptStream {
                inner,
                pending,
                request_id,
                app_name,
                direct_tx,
            })
        })
    }
}

pub struct InterceptStream<St> {
    inner: St,
    pending: PendingMap,
    request_id: i32,
    app_name: AppNameStore,
    direct_tx: Option<DirectSender>,
}

impl<St, E> Stream for InterceptStream<St>
where
    St: Stream<Item = Result<Message, E>> + Unpin,
{
    type Item = Result<Message, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Ok(msg))) => {
                let docs = extract_response_docs(&msg.operation);
                let mut pending = self.pending.lock().unwrap();
                if let Some(entry) = pending.get_mut(&self.request_id) {
                    if !docs.is_empty() {
                        entry.response_docs.extend(docs);
                    }
                    // Emit DirectEntry immediately — do NOT remove from PendingMap so
                    // ExplainEvent (if it fires) can still consume filter/pipeline/etc.
                    let now = current_ms();
                    let latency_ms = now.saturating_sub(entry.start_ms) as u32;
                    let app_name = self.app_name.lock().unwrap().clone();
                    let direct = DirectEntry {
                        request_id: self.request_id,
                        command: entry.command.clone(),
                        db: entry.db.clone(),
                        coll: entry.coll.clone(),
                        start_ms: entry.start_ms,
                        latency_ms,
                        filter: entry.filter.clone(),
                        pipeline: entry.pipeline.clone(),
                        update: entry.update.clone(),
                        doc: entry.doc.clone(),
                        app_name,
                    };
                    if let Some(tx) = &self.direct_tx {
                        let _ = tx.send(direct);
                    }
                }
                Poll::Ready(Some(Ok(msg)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn current_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn extract_db_coll(operation: &Operation, coll_key: &str) -> (String, String) {
    let body = match op_msg_body(operation) {
        Some(b) => b,
        None => return ("unknown".to_string(), "unknown".to_string()),
    };
    let db = body.get_str("$db").unwrap_or("unknown").to_string();
    let coll = body.get_str(coll_key).unwrap_or("unknown").to_string();
    (db, coll)
}

// ── BSON conversion ───────────────────────────────────────────────────────────

fn bson_doc_to_model(doc: &bson::Document) -> BsonDoc {
    doc.iter()
        .map(|(k, v)| (k.clone(), bson_val_to_model(v)))
        .collect()
}

fn bson_val_to_model(val: &bson::Bson) -> BsonVal {
    match val {
        bson::Bson::Null => BsonVal::Null,
        bson::Bson::Boolean(b) => BsonVal::Bool(*b),
        bson::Bson::Int32(i) => BsonVal::Int(*i as i64),
        bson::Bson::Int64(i) => BsonVal::NumberLong(*i),
        bson::Bson::Double(f) => BsonVal::Float(*f),
        bson::Bson::String(s) => BsonVal::Str(s.clone()),
        bson::Bson::ObjectId(id) => BsonVal::ObjectId(id.to_string()),
        bson::Bson::DateTime(dt) => BsonVal::IsoDate(dt.to_string()),
        bson::Bson::Timestamp(ts) => BsonVal::Timestamp(format!("{}:{}", ts.time, ts.increment)),
        bson::Bson::Array(arr) => BsonVal::Array(arr.iter().map(bson_val_to_model).collect()),
        bson::Bson::Document(doc) => BsonVal::Doc(bson_doc_to_model(doc)),
        other => BsonVal::Str(other.to_string()),
    }
}

// ── Request extraction helpers ────────────────────────────────────────────────

fn op_msg_body(operation: &Operation) -> Option<&bson::Document> {
    match operation {
        Operation::Message(OperationMessage { sections, .. }) => {
            sections.iter().find_map(|s| match s {
                OpMsgSection::Body(doc) => Some(doc),
                _ => None,
            })
        }
        Operation::Query(q) => Some(&q.query),
        _ => None,
    }
}

fn extract_app_name(operation: &Operation) -> Option<String> {
    let body = op_msg_body(operation)?;
    let client = body.get_document("client").ok()?;
    let application = client.get_document("application").ok()?;
    application.get_str("name").ok().map(str::to_owned)
}

fn extract_filter(operation: &Operation) -> Option<BsonDoc> {
    let body = op_msg_body(operation)?;
    body.get_document("filter").ok().map(bson_doc_to_model)
}

fn extract_pipeline(operation: &Operation) -> Option<Vec<BsonDoc>> {
    let body = op_msg_body(operation)?;
    let arr = body.get_array("pipeline").ok()?;
    let stages: Vec<BsonDoc> = arr
        .iter()
        .filter_map(|v| {
            if let bson::Bson::Document(doc) = v {
                Some(bson_doc_to_model(doc))
            } else {
                None
            }
        })
        .collect();
    if stages.is_empty() {
        None
    } else {
        Some(stages)
    }
}

fn extract_update(operation: &Operation) -> (Option<BsonDoc>, Option<BsonDoc>) {
    let body = match op_msg_body(operation) {
        Some(b) => b,
        None => return (None, None),
    };
    let updates = match body.get_array("updates").ok() {
        Some(u) => u,
        None => return (None, None),
    };
    let first = match updates.first() {
        Some(bson::Bson::Document(d)) => d,
        _ => return (None, None),
    };
    let filter = first.get_document("q").ok().map(bson_doc_to_model);
    let update = first.get_document("u").ok().map(bson_doc_to_model);
    (filter, update)
}

fn extract_delete_filter(operation: &Operation) -> Option<BsonDoc> {
    let body = op_msg_body(operation)?;
    if let Ok(deletes) = body.get_array("deletes") {
        if let Some(bson::Bson::Document(d)) = deletes.first() {
            if let Ok(q) = d.get_document("q") {
                return Some(bson_doc_to_model(q));
            }
        }
    }
    body.get_document("query").ok().map(bson_doc_to_model)
}

fn extract_insert_doc(operation: &Operation) -> Option<BsonDoc> {
    match operation {
        Operation::Message(OperationMessage { sections, .. }) => {
            for section in sections {
                if let OpMsgSection::DocumentSequence {
                    identifier,
                    documents,
                } = section
                {
                    if identifier == "documents" {
                        return documents.first().map(bson_doc_to_model);
                    }
                }
            }
            for section in sections {
                if let OpMsgSection::Body(body) = section {
                    if let Ok(arr) = body.get_array("documents") {
                        if let Some(bson::Bson::Document(d)) = arr.first() {
                            return Some(bson_doc_to_model(d));
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

// ── Response extraction helpers ───────────────────────────────────────────────

fn extract_response_docs(operation: &Operation) -> Vec<BsonDoc> {
    match operation {
        Operation::Message(OperationMessage { sections, .. }) => {
            for section in sections {
                if let OpMsgSection::Body(body) = section {
                    if let Ok(cursor) = body.get_document("cursor") {
                        for batch_key in &["firstBatch", "nextBatch"] {
                            if let Ok(arr) = cursor.get_array(batch_key) {
                                return arr
                                    .iter()
                                    .filter_map(|v| {
                                        if let bson::Bson::Document(d) = v {
                                            Some(bson_doc_to_model(d))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
            vec![]
        }
        Operation::Reply(r) => r.documents.iter().map(bson_doc_to_model).collect(),
        _ => vec![],
    }
}
