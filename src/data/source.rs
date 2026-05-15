use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;

use super::model::QueryEntry;

/// Shared entry buffer — both the GUI feed and the MCP server read from this.
pub type EntryStore = Arc<Mutex<VecDeque<QueryEntry>>>;

pub fn new_entry_store() -> EntryStore {
    Arc::new(Mutex::new(VecDeque::new()))
}

pub trait DataSource: Send + 'static {
    /// Called once per connection. Implementor spawns its own task and:
    ///   - sends every entry through `tx` (for GUI batching subscription)
    ///   - appends every entry to `store` (shared with MCP server)
    fn start(self: Box<Self>, tx: mpsc::Sender<QueryEntry>, store: EntryStore);
}
