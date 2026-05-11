use nutype::nutype;

#[nutype(validate(greater = 0), derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord))]
pub struct QueryId(u64);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct LatencyMs(u32);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct TimestampMs(u64);

#[nutype(sanitize(trim), validate(not_empty), derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct CollectionName(String);

#[nutype(sanitize(trim), validate(not_empty), derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct AppName(String);

#[nutype(sanitize(trim), validate(not_empty), derive(Debug, Clone, PartialEq, Eq, Hash))]
pub struct IndexName(String);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct DocsExamined(u64);

#[nutype(derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord))]
pub struct DocsReturned(u64);

#[nutype(sanitize(trim), derive(Debug, Clone, PartialEq, Eq))]
pub struct FilterText(String);

#[nutype(sanitize(trim), derive(Debug, Clone, PartialEq, Eq))]
pub struct ComposeText(String);
