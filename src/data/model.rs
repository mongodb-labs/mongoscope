use super::types::*;
use indexmap::IndexMap;

// Used by the hidden Schema tab — https://github.com/mongodb-labs/mongoscope/issues/28
#[allow(dead_code)]
pub struct SchemaField {
    pub name: &'static str,
    pub type_str: &'static str,
    pub samples: &'static [&'static str],
    pub coverage_pct: u8,
}

// Used by the hidden Schema tab — https://github.com/mongodb-labs/mongoscope/issues/28
#[allow(dead_code)]
pub struct CollectionSchema {
    pub coll: CollectionName,
    pub fields: Vec<SchemaField>,
    pub sampled_docs: u32,
}

pub type BsonDoc = IndexMap<String, BsonVal>;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum BsonVal {
    Doc(BsonDoc),
    Array(Vec<BsonVal>),
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    ObjectId(String),
    IsoDate(String),
    Timestamp(String),
    NumberLong(i64),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Find,
    Aggregate,
    CountDocuments,
    InsertOne,
    UpdateOne,
    UpdateMany,
    DeleteMany,
    /// Raw command name from wire protocol.
    Unknown(String),
}

impl Op {
    pub fn label(&self) -> String {
        match self {
            Op::Find => "FIND".into(),
            Op::Aggregate => "AGG".into(),
            Op::CountDocuments => "CNT".into(),
            Op::InsertOne => "INS".into(),
            Op::UpdateOne => "UPD".into(),
            Op::UpdateMany => "UPD×".into(),
            Op::DeleteMany => "DEL".into(),
            Op::Unknown(s) => s.to_uppercase(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    CollScan,
    IxScan(IndexName),
    IdHack,
}

impl Plan {
    pub fn label(&self) -> String {
        match self {
            Plan::CollScan => "COLLSCAN".into(),
            Plan::IxScan(_) => "IXSCAN".into(),
            Plan::IdHack => "IDHACK".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexSuggestion {
    pub ixscan_ms: u32,
    pub fetch_ms: u32,
    pub sort_ms: u32,
    pub limit_ms: u32,
}

#[derive(Debug, Clone)]
pub enum Suggestion {
    CreateIndex(IndexSuggestion),
}

#[derive(Debug, Clone)]
pub struct QueryEntry {
    pub id: QueryId,
    pub t_ms: TimestampMs,
    pub latency_ms: LatencyMs,
    pub op: Op,
    pub db: DatabaseName,
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
    pub slow: bool,
    pub is_system: bool,
    pub conn_id: ConnId,
    pub lsid: Option<String>,
    pub cluster_time: Option<String>,
    pub response_docs: Vec<BsonDoc>,
    pub rejected_plan_count: u8,
    pub suggestions: Vec<Suggestion>,
}
