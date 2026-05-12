use crate::data::model::QueryEntry;
use crate::ui::feed::FeedState;
use crate::ui::sidebar::clients::{app_color_for, ClientItem};
use crate::ui::sidebar::connections::ConnectionItem;
use crate::ui::sidebar::collections::CollectionItem;
use crate::ui::sidebar::databases::DatabaseItem;

pub struct ConnectionState {
    pub item: ConnectionItem,
    pub feed: FeedState,
    pub databases: Vec<DatabaseItem>,
    pub clients: Vec<ClientItem>,
    pub capturing: bool,
}

impl ConnectionState {
    pub fn new(item: ConnectionItem) -> Self {
        Self {
            item,
            feed: FeedState::new(),
            databases: vec![],
            clients: vec![],
            capturing: true,
        }
    }

    pub fn register_entries(&mut self, entries: &[QueryEntry]) {
        for entry in entries {
            let app_name = entry.app.to_string();
            if !self.clients.iter().any(|c| c.name == app_name) {
                let color = app_color_for(&app_name);
                self.clients.push(ClientItem { name: app_name, color, active: false });
            }
            let db_name = entry.db.to_string();
            let coll_name = entry.coll.to_string();
            if let Some(db) = self.databases.iter_mut().find(|d| d.name == db_name) {
                if !db.collections.iter().any(|c| c.name == coll_name) {
                    db.collections.push(CollectionItem {
                        name: coll_name,
                        docs: 0,
                        size: "".into(),
                        idx: 0,
                        active: false,
                    });
                }
            } else {
                self.databases.push(DatabaseItem {
                    name: db_name,
                    expanded: true,
                    active: false,
                    collections: vec![CollectionItem {
                        name: coll_name,
                        docs: 0,
                        size: "".into(),
                        idx: 0,
                        active: false,
                    }],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::sidebar::connections::{ConnectionColor, ConnectionItem};

    fn make_item(id: usize) -> ConnectionItem {
        ConnectionItem {
            id,
            label: "test".into(),
            topology: "direct".into(),
            uri: "mongodb://localhost:27017/".into(),
            proxy_port: 27117,
            color: ConnectionColor::None,
            active: true,
            live: true,
        }
    }

    #[test]
    fn new_starts_capturing() {
        let s = ConnectionState::new(make_item(1));
        assert!(s.capturing);
    }

    #[test]
    fn new_has_empty_databases() {
        let s = ConnectionState::new(make_item(1));
        assert!(s.databases.is_empty());
    }

    #[test]
    fn new_has_empty_clients() {
        let s = ConnectionState::new(make_item(1));
        assert!(s.clients.is_empty());
    }

    #[test]
    fn new_has_empty_feed() {
        let s = ConnectionState::new(make_item(1));
        assert!(s.feed.entries.is_empty());
    }
}
