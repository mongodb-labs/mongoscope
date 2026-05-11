use super::model::QueryEntry;

/// Swap point: replace MockSource with a real wire-protocol proxy by implementing this trait.
pub trait DataSource: Send + 'static {
    /// Called once at startup. Implementor spawns its own task and sends entries
    /// through `tx` until the channel closes.
    fn start(self: Box<Self>, tx: tokio::sync::mpsc::Sender<QueryEntry>);
}
