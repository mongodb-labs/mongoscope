use indexmap::IndexMap;
use super::types::*;

pub type BsonDoc = IndexMap<String, BsonVal>;

#[derive(Debug, Clone, PartialEq)]
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
    /// Raw command name from wire protocol. Display uppercase, truncated to 6 chars.
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
            Op::Unknown(s) => s.to_uppercase().chars().take(6).collect(),
        }
    }

    pub fn is_read(&self) -> bool {
        matches!(self, Op::Find | Op::FindOne | Op::Aggregate | Op::CountDocuments)
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
    pub slow: bool,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub name: CollectionName,
    pub doc_count: u64,
    pub size_human: String,
    pub index_count: u8,
}

#[derive(Debug, Clone)]
pub struct ClientApp {
    pub name: AppName,
    pub color: [u8; 3],
}

/// Static catalog used by sidebar and mock data.
pub fn collections() -> Vec<Collection> {
    vec![
        Collection { name: CollectionName::try_new("orders").unwrap(),    doc_count: 2_413_882,  size_human: "8.4 GB".into(),  index_count: 7 },
        Collection { name: CollectionName::try_new("products").unwrap(),  doc_count: 184_302,    size_human: "412 MB".into(),  index_count: 5 },
        Collection { name: CollectionName::try_new("users").unwrap(),     doc_count: 892_014,    size_human: "1.8 GB".into(),  index_count: 6 },
        Collection { name: CollectionName::try_new("carts").unwrap(),     doc_count: 71_205,     size_human: "98 MB".into(),   index_count: 3 },
        Collection { name: CollectionName::try_new("sessions").unwrap(),  doc_count: 12_044_119, size_human: "4.2 GB".into(),  index_count: 4 },
        Collection { name: CollectionName::try_new("reviews").unwrap(),   doc_count: 3_201_885,  size_human: "2.1 GB".into(),  index_count: 5 },
        Collection { name: CollectionName::try_new("inventory").unwrap(), doc_count: 48_112,     size_human: "64 MB".into(),   index_count: 4 },
        Collection { name: CollectionName::try_new("events").unwrap(),    doc_count: 88_912_004, size_human: "41.2 GB".into(), index_count: 2 },
    ]
}

pub fn client_apps() -> Vec<ClientApp> {
    vec![
        ClientApp { name: AppName::try_new("checkout-svc").unwrap(),      color: [96,  165, 250] },
        ClientApp { name: AppName::try_new("catalog-api").unwrap(),       color: [167, 139, 250] },
        ClientApp { name: AppName::try_new("analytics-worker").unwrap(),  color: [244, 114, 182] },
        ClientApp { name: AppName::try_new("admin-portal").unwrap(),      color: [251, 191,  36] },
        ClientApp { name: AppName::try_new("mobile-bff").unwrap(),        color: [52,  211, 153] },
    ]
}
