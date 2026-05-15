use super::types::*;
use indexmap::IndexMap;

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

/// All known MongoDB operation types.
/// Start all mock data as `Unknown` and promote to typed variants as support is added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Find,
    FindOne,
    Aggregate,
    CountDocuments,
    InsertOne,
    UpdateOne,
    UpdateMany,
    DeleteOne,
    DeleteMany,
    /// Raw command name from wire protocol.
    Unknown(String),
}

impl Op {
    pub fn label(&self) -> String {
        match self {
            Op::Find => "FIND".into(),
            Op::FindOne => "FIND¹".into(),
            Op::Aggregate => "AGG".into(),
            Op::CountDocuments => "CNT".into(),
            Op::InsertOne => "INS".into(),
            Op::UpdateOne => "UPD".into(),
            Op::UpdateMany => "UPD×".into(),
            Op::DeleteOne | Op::DeleteMany => "DEL".into(),
            Op::Unknown(s) => s.to_uppercase(),
        }
    }

    pub fn is_read(&self) -> bool {
        matches!(
            self,
            Op::Find | Op::FindOne | Op::Aggregate | Op::CountDocuments
        )
    }

    pub fn is_write(&self) -> bool {
        matches!(self, Op::InsertOne | Op::UpdateOne | Op::UpdateMany)
    }

    pub fn is_delete(&self) -> bool {
        matches!(self, Op::DeleteOne | Op::DeleteMany)
    }
}

/// Winning query plan. `Unknown` for unrecognized stage names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    CollScan,
    IxScan(IndexName),
    IdHack,
    IxScanLookup(IndexName),
    /// Unrecognized plan stage — display as-is, no color coding.
    Unknown(String),
}

impl Plan {
    pub fn label(&self) -> String {
        match self {
            Plan::CollScan => "COLLSCAN".into(),
            Plan::IxScan(_) => "IXSCAN".into(),
            Plan::IdHack => "IDHACK".into(),
            Plan::IxScanLookup(_) => "IXSCAN+LOOKUP".into(),
            Plan::Unknown(s) => s.clone(),
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
