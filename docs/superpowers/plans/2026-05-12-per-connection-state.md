# Per-Connection State Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move feed, databases, and client state inside each `ConnectionState` so the UI reflects whichever connection is selected, and mock data only starts after a connection is established via the dialog.

**Architecture:** Add a `ConnectionState` struct that owns `FeedState`, `databases`, `clients`, and `capturing`. `SidebarState` holds `Vec<ConnectionState>` and an `active_id`. `App` removes its global `feed`/`capturing`/`tx` fields and delegates to `sidebar.active()`. Each live connection runs its own keyed subscription; no connections means no data flows.

**Tech Stack:** Rust, iced 0.13 (`Subscription::run_with_id`, `iced::stream::channel`), tokio

---

## File Map

| File | Change |
|------|--------|
| `src/ui/sidebar/connection_state.rs` | **Create** — `ConnectionState` struct + `register_entries` |
| `src/ui/sidebar/mod.rs` | **Modify** — restructure `SidebarState`, add `active()`/`active_mut()`, update `update()` and `view()` |
| `src/main.rs` | **Modify** — remove `feed`/`capturing`/`tx` from `App`, update `Message`, `update()`, `subscription()`, `view()` |

No other files change.

---

### Task 1: Create `ConnectionState`

**Files:**
- Create: `src/ui/sidebar/connection_state.rs`

- [ ] **Step 1: Write the failing tests**

Create `src/ui/sidebar/connection_state.rs` with only the test module (will fail to compile — that's the signal):

```rust
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
```

- [ ] **Step 2: Add `pub mod connection_state;` to `src/ui/sidebar/mod.rs`**

At the top of `src/ui/sidebar/mod.rs`, after the existing `pub mod` lines, add:

```rust
pub mod connection_state;
pub use connection_state::ConnectionState;
```

- [ ] **Step 3: Run to confirm compile error**

```bash
cargo check 2>&1 | head -20
```

Expected: error about `ConnectionState` not being defined.

- [ ] **Step 4: Implement `ConnectionState`**

Write the full struct in `src/ui/sidebar/connection_state.rs`:

```rust
use crate::data::model::QueryEntry;
use crate::ui::feed::FeedState;
use crate::ui::sidebar::clients::{app_color_for, ClientItem};
use crate::ui::sidebar::connections::ConnectionItem;
use crate::ui::sidebar::databases::{CollectionItem, DatabaseItem};

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
```

- [ ] **Step 5: Run tests**

```bash
cargo test ui::sidebar::connection_state 2>&1
```

Expected: 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/ui/sidebar/connection_state.rs src/ui/sidebar/mod.rs
git commit -m "feat: add ConnectionState with feed, databases, clients, capturing"
```

---

### Task 2: Restructure `SidebarState`

**Files:**
- Modify: `src/ui/sidebar/mod.rs`

This task changes `SidebarState` fields and all its methods. `main.rs` will fail to compile until Task 3 — that's expected. Do not commit until Task 3 restores compilation.

- [ ] **Step 1: Replace `SidebarState` fields**

In `src/ui/sidebar/mod.rs`, replace the `SidebarState` struct definition and `SidebarState::new()`:

```rust
pub struct SidebarState {
    pub connections: Vec<ConnectionState>,
    pub active_id: Option<usize>,
    pub dialog: Option<ConnectionDialogState>,
    pub saved_views: Vec<SavedView>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            connections: vec![],
            active_id: None,
            dialog: None,
            saved_views: vec![
                SavedView { id: 0, label: "slow queries (>500ms)".into() },
                SavedView { id: 1, label: "COLLSCANs only".into() },
                SavedView { id: 2, label: "writes to orders".into() },
            ],
        }
    }

    pub fn active(&self) -> Option<&ConnectionState> {
        let id = self.active_id?;
        self.connections.iter().find(|c| c.item.id == id)
    }

    pub fn active_mut(&mut self) -> Option<&mut ConnectionState> {
        let id = self.active_id?;
        self.connections.iter_mut().find(|c| c.item.id == id)
    }

    pub fn active_db(&self) -> Option<String> {
        self.active()?.databases.iter().find(|d| d.active).map(|d| d.name.clone())
    }

    pub fn active_coll(&self) -> Option<String> {
        let conn = self.active()?;
        conn.databases
            .iter()
            .find(|d| d.active)
            .and_then(|d| d.collections.iter().find(|c| c.active))
            .map(|c| c.name.clone())
    }
```

Remove `register_entries` from `SidebarState` entirely (it now lives on `ConnectionState`).

- [ ] **Step 2: Update `SidebarState::update()` — Connections messages**

Replace the `SidebarMsg::Connections` arm in `update()`:

```rust
SidebarMsg::Connections(m) => match m {
    ConnectionsMsg::Select(id) => {
        for c in &mut self.connections {
            c.item.active = c.item.id == id;
        }
        self.active_id = Some(id);
    }
    ConnectionsMsg::Add => {
        if self.dialog.is_none() {
            self.dialog = Some(ConnectionDialogState::new());
        }
    }
    ConnectionsMsg::DialogUriChanged(s) => {
        if let Some(d) = &mut self.dialog { d.uri = s; d.error = None; }
    }
    ConnectionsMsg::DialogNameChanged(s) => {
        if let Some(d) = &mut self.dialog { d.name = s; }
    }
    ConnectionsMsg::DialogColorChanged(c) => {
        if let Some(d) = &mut self.dialog { d.color = c; }
    }
    ConnectionsMsg::DialogConnect => {
        if let Some(d) = &mut self.dialog {
            d.step = crate::ui::dialog::DialogStep::Step1 { connecting: true };
            d.error = None;
        }
    }
    ConnectionsMsg::DialogConnectResult(Ok(port)) => {
        if let Some(d) = &mut self.dialog {
            d.proxy_port = port;
            d.step = crate::ui::dialog::DialogStep::Step2;
        }
    }
    ConnectionsMsg::DialogConnectResult(Err(e)) => {
        if let Some(d) = &mut self.dialog {
            d.step = crate::ui::dialog::DialogStep::Step1 { connecting: false };
            d.error = Some(e);
        }
    }
    ConnectionsMsg::DialogBack => {
        if let Some(d) = &mut self.dialog {
            d.step = crate::ui::dialog::DialogStep::Step1 { connecting: false };
        }
    }
    ConnectionsMsg::DialogDone => {
        if let Some(d) = &self.dialog {
            let next_id = self.connections.iter().map(|c| c.item.id).max().unwrap_or(0) + 1;
            let label = if d.name.is_empty() {
                d.uri
                    .trim_start_matches("mongodb://")
                    .trim_start_matches("mongodb+srv://")
                    .split('/')
                    .next()
                    .unwrap_or("connection")
                    .to_owned()
            } else {
                d.name.clone()
            };
            let topology = format!("direct · proxy :{}", d.proxy_port);
            let item = ConnectionItem {
                id: next_id,
                label,
                topology,
                uri: d.uri.clone(),
                proxy_port: d.proxy_port,
                color: d.color,
                active: true,
                live: true,
            };
            for c in &mut self.connections {
                c.item.active = false;
            }
            self.connections.push(ConnectionState::new(item));
            self.active_id = Some(next_id);
        }
        self.dialog = None;
    }
    ConnectionsMsg::DialogCancel => {
        self.dialog = None;
    }
    ConnectionsMsg::DialogCopyUri => {
        // handled in App::update to produce clipboard Task
    }
    ConnectionsMsg::DialogNoop => {}
},
```

- [ ] **Step 3: Update `SidebarState::update()` — Databases and Clients messages**

Replace the `SidebarMsg::Databases` and `SidebarMsg::Clients` arms:

```rust
SidebarMsg::Databases(m) => {
    if let Some(conn) = self.active_mut() {
        match m {
            DatabasesMsg::ToggleDb(name) => apply_toggle_db(&mut conn.databases, &name),
            DatabasesMsg::ToggleCollection(db, coll) => {
                apply_toggle_collection(&mut conn.databases, &db, &coll)
            }
        }
    }
}
SidebarMsg::Clients(m) => {
    if let Some(conn) = self.active_mut() {
        match m {
            ClientsMsg::Toggle(name) => {
                for c in &mut conn.clients {
                    if c.name == name { c.active = !c.active; }
                }
            }
        }
    }
}
```

- [ ] **Step 4: Update `SidebarState::view()` — extract active connection data**

In the `view()` method, replace the section that builds db/client panels. Find these lines:

```rust
let db_count = self.databases.len();
```

Replace everything from that line through the `databases_panel` and `clients_panel` calls with:

```rust
let (dbs, clients) = self.active()
    .map(|c| (c.databases.as_slice(), c.clients.as_slice()))
    .unwrap_or((&[], &[]));

let db_count = dbs.len();
```

Then pass `dbs` and `clients` instead of `&self.databases` / `&self.clients` in the panel calls:

```rust
section_header("DATABASES".into(), Some(format!("{} dbs", db_count))),
databases_panel(
    dbs,
    move |m| on_msg(SidebarMsg::Databases(m)),
    palette,
),
section_header("CLIENTS".into(), None),
clients_panel(
    clients,
    move |m| on_msg(SidebarMsg::Clients(m)),
    palette,
),
```

- [ ] **Step 5: Update `connections_panel` call in `view()`**

The `connections_panel` still takes `&[ConnectionItem]`. Extract items from `Vec<ConnectionState>`:

```rust
let conn_items: Vec<ConnectionItem> = self.connections.iter().map(|c| c.item.clone()).collect();
```

Pass `&conn_items` to `connections_panel`:

```rust
connections_panel(
    &conn_items,
    self.dialog.is_some(),
    move |m| on_msg(SidebarMsg::Connections(m)),
    palette,
),
```

- [ ] **Step 6: Add tests for `SidebarState` helpers**

At the bottom of `src/ui/sidebar/mod.rs`, add a test module:

```rust
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
            active: false,
            live: true,
        }
    }

    #[test]
    fn active_returns_none_when_empty() {
        let s = SidebarState::new();
        assert!(s.active().is_none());
    }

    #[test]
    fn active_returns_correct_connection() {
        let mut s = SidebarState::new();
        s.connections.push(ConnectionState::new(make_item(1)));
        s.connections.push(ConnectionState::new(make_item(2)));
        s.active_id = Some(2);
        assert_eq!(s.active().unwrap().item.id, 2);
    }

    #[test]
    fn dialog_done_creates_connection_state_with_capturing_true() {
        let mut s = SidebarState::new();
        s.dialog = Some(crate::ui::dialog::ConnectionDialogState::new());
        if let Some(d) = &mut s.dialog {
            d.proxy_port = 27117;
            d.step = crate::ui::dialog::DialogStep::Step2;
        }
        s.update(SidebarMsg::Connections(ConnectionsMsg::DialogDone));
        assert_eq!(s.connections.len(), 1);
        assert!(s.connections[0].capturing);
        assert!(s.dialog.is_none());
    }

    #[test]
    fn dialog_done_sets_active_id() {
        let mut s = SidebarState::new();
        s.dialog = Some(crate::ui::dialog::ConnectionDialogState::new());
        if let Some(d) = &mut s.dialog { d.proxy_port = 27117; }
        s.update(SidebarMsg::Connections(ConnectionsMsg::DialogDone));
        assert!(s.active_id.is_some());
        assert_eq!(s.active_id, Some(s.connections[0].item.id));
    }
}
```

---

### Task 3: Update `App` (restores compilation)

**Files:**
- Modify: `src/main.rs`

Do all steps in this task before running `cargo check` — they're interdependent.

- [ ] **Step 1: Update `Message` enum**

Change `QueriesReceived` to carry a connection id:

```rust
#[derive(Debug, Clone)]
enum Message {
    QueriesReceived(usize, Vec<QueryEntry>),
    Feed(FeedMsg),
    Inspector(InspectorMsg),
    Sidebar(SidebarMsg),
    ToggleTheme,
    ToggleDensity,
    ToggleCapture,
    Menu(MenuMsg),
    SidebarResizeStart,
    SidebarResizeMove(f32),
    SidebarResizeEnd,
}
```

- [ ] **Step 2: Update `App` struct**

Remove `capturing`, `tx`, and `feed` fields. The new struct:

```rust
struct App {
    inspector: InspectorState,
    sidebar: SidebarState,
    theme: Theme,
    density: Density,
    sidebar_width: f32,
    sidebar_dragging: bool,
}
```

- [ ] **Step 3: Update `App::new()`**

```rust
impl App {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self {
            inspector: InspectorState::new(),
            sidebar: SidebarState::new(),
            theme: Theme::Dark,
            density: Density::Compact,
            sidebar_width: 220.0,
            sidebar_dragging: false,
        };
        if app.sidebar.connections.is_empty() {
            app.sidebar.dialog = Some(ui::dialog::ConnectionDialogState::new());
        }
        (app, Task::none())
    }
```

- [ ] **Step 4: Update `App::update()` — `QueriesReceived`**

Replace the `Message::QueriesReceived` arm:

```rust
Message::QueriesReceived(conn_id, entries) => {
    let active_id = self.sidebar.active_id;
    if let Some(conn) = self.sidebar.connections.iter_mut().find(|c| c.item.id == conn_id) {
        if !conn.capturing { return Task::none(); }
        conn.register_entries(&entries);
        let mut added = 0usize;
        for entry in entries {
            if conn.feed.push_entry(entry) { added += 1; }
        }
        let is_active = active_id == Some(conn_id);
        if added == 0 || !is_active { return Task::none(); }
        if conn.feed.scroll_locked {
            let scrolling_up = conn.feed.scroll_y < conn.feed.prev_scroll_y;
            if !scrolling_up {
                let dy = added as f32 * self.density.row_height();
                conn.feed.pending_scroll_by += 1;
                return scrollable::scroll_by(
                    scrollable::Id::new(FEED_SCROLL_ID),
                    scrollable::AbsoluteOffset { x: 0.0, y: dy },
                );
            }
        } else if !conn.feed.paused {
            conn.feed.pending_scroll_to += 1;
            return scrollable::scroll_to(
                scrollable::Id::new(FEED_SCROLL_ID),
                scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
            );
        }
    }
    Task::none()
}
```

- [ ] **Step 5: Update `App::update()` — `Feed` message**

```rust
Message::Feed(m) => {
    if let Some(conn) = self.sidebar.active_mut() {
        conn.feed.update(m);
    }
}
```

- [ ] **Step 6: Update `App::update()` — `ToggleCapture`**

```rust
Message::ToggleCapture => {
    if let Some(conn) = self.sidebar.active_mut() {
        conn.capturing = !conn.capturing;
    }
}
```

- [ ] **Step 7: Update `App::update()` — `Sidebar` / `DatabasesMsg` scope**

The `feed.filter.set_scope` call needs to reach the active connection's feed. Replace:

```rust
Message::Sidebar(ref m) => {
    match m {
        SidebarMsg::Connections(ConnectionsMsg::DialogConnect) => {
            self.sidebar.update(m.clone());
            return Task::perform(
                async {
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    Result::<u16, String>::Ok(27117)
                },
                |r| Message::Sidebar(SidebarMsg::Connections(
                    ConnectionsMsg::DialogConnectResult(r)
                )),
            );
        }
        SidebarMsg::Connections(ConnectionsMsg::DialogCopyUri) => {
            if let Some(d) = &self.sidebar.dialog {
                let uri = format!(
                    "mongodb://localhost:{}/?directConnection=true",
                    d.proxy_port
                );
                return iced::clipboard::write::<Message>(uri);
            }
        }
        _ => {}
    }
    self.sidebar.update(m.clone());
    if let SidebarMsg::Databases(_) = m {
        let (db, coll) = (self.sidebar.active_db(), self.sidebar.active_coll());
        if let Some(conn) = self.sidebar.active_mut() {
            conn.feed.filter.set_scope(db, coll);
        }
    }
}
```

- [ ] **Step 8: Update `App::subscription()`**

Replace the entire `subscription` method:

```rust
fn subscription(&self) -> Subscription<Message> {
    let data_subs: Vec<Subscription<Message>> = self.sidebar.connections
        .iter()
        .filter(|c| c.item.live)
        .map(|c| {
            let id = c.item.id;
            Subscription::run_with_id(id, move || {
                iced::stream::channel(256, move |mut output| async move {
                    use iced::futures::SinkExt;
                    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
                    Box::new(MockSource).start(tx);
                    loop {
                        let first = rx.recv().await;
                        let Some(first) = first else { break };
                        let mut batch = vec![first];
                        let deadline = tokio::time::Instant::now()
                            + std::time::Duration::from_millis(300);
                        loop {
                            match tokio::time::timeout_at(deadline, rx.recv()).await {
                                Ok(Some(e)) => {
                                    batch.push(e);
                                    if batch.len() >= 8 { break; }
                                }
                                _ => break,
                            }
                        }
                        let _ = output.send(Message::QueriesReceived(id, batch)).await;
                    }
                })
            })
        })
        .collect();

    if self.sidebar_dragging {
        let drag_sub = iced::event::listen_with(|event, _status, _window| {
            match event {
                iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    Some(Message::SidebarResizeMove(position.x))
                }
                iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Message::SidebarResizeEnd)
                }
                _ => None,
            }
        });
        let mut all = data_subs;
        all.push(drag_sub);
        Subscription::batch(all)
    } else {
        Subscription::batch(data_subs)
    }
}
```

- [ ] **Step 9: Update `App::view()`**

Replace the section that reads from `self.feed` and `self.capturing`. The full `view()` method:

```rust
fn view(&self) -> Element<Message> {
    let palette = self.theme.palette();
    let fs = self.density.fs_base();
    let bg = palette.bg;
    let border_color = palette.border;

    let capturing = self.sidebar.active().map(|c| c.capturing).unwrap_or(false);

    let conn = ConnInfo {
        host: "localhost:27017".into(),
        uri: "mongoscope://localhost".into(),
        rs_name: Some("rs0".into()),
        connected: true,
    };

    let top = topbar(
        &conn,
        capturing,
        |m| Message::Menu(m),
        Message::ToggleCapture,
        &palette,
    );

    let (ops_per_sec, query_count, slow_count) = self.sidebar.active()
        .map(|c| {
            let ops = if c.feed.entries.len() >= 10 { 12.5f32 } else { 0.0 };
            let slow = c.feed.entries.iter().filter(|e| e.slow).count();
            (ops, c.feed.entries.len(), slow)
        })
        .unwrap_or((0.0, 0, 0));

    let status = statusbar(
        &StatusInfo {
            ops_per_sec,
            query_count,
            slow_count,
            theme_label: self.theme.label(),
            density_label: self.density.label(),
        },
        Message::ToggleTheme,
        Message::ToggleDensity,
        &palette,
    );

    let sidebar_el = self.sidebar.view(|m| Message::Sidebar(m), &palette, self.sidebar_width);

    let resize_handle = mouse_area(
        container(iced::widget::Space::new(4.0, Length::Fill))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(border_color)),
                ..Default::default()
            })
    )
    .on_press(Message::SidebarResizeStart)
    .on_release(Message::SidebarResizeEnd)
    .interaction(mouse::Interaction::ResizingHorizontally);

    let feed_el: Element<Message> = if let Some(conn) = self.sidebar.active() {
        conn.feed.view(|m| Message::Feed(m), palette, self.density)
    } else {
        iced::widget::Space::new(Length::Fill, Length::Fill).into()
    };

    let selected_entry = self.sidebar.active().and_then(|c| {
        c.feed.selected.and_then(|id| c.feed.entries.iter().find(|e| e.id == id))
    });

    let inspector_el = self.inspector.view(
        selected_entry,
        |m| Message::Inspector(m),
        palette,
        fs,
    );

    let main_pane = column![
        container(feed_el)
            .width(Length::Fill)
            .height(Length::Fill),
        container(inspector_el)
            .width(Length::Fill)
            .height(340),
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill);

    let body = row![
        sidebar_el,
        resize_handle,
        main_pane,
    ]
    .spacing(0)
    .height(Length::Fill);

    let base: Element<Message> = container(
        column![top, body, status].spacing(0)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| iced::widget::container::Style {
        background: Some(iced::Background::Color(bg)),
        border: iced::Border { color: border_color, width: 0.0, radius: 0.0.into() },
        ..Default::default()
    })
    .into();

    if let Some(dialog_state) = &self.sidebar.dialog {
        let dialog_palette = self.theme.palette();
        stack![
            base,
            dialog_view(
                dialog_state,
                |m| Message::Sidebar(SidebarMsg::Connections(m)),
                &dialog_palette,
            )
        ]
        .into()
    } else {
        base
    }
}
```

- [ ] **Step 10: Compile check**

```bash
cargo check 2>&1
```

Expected: no errors. Fix any remaining type errors before continuing (common: borrow checker on `conn` after `iter_mut`, missing `use` imports).

- [ ] **Step 11: Run all tests**

```bash
cargo test 2>&1
```

Expected: all tests pass including the new `connection_state` and `sidebar` tests.

- [ ] **Step 12: Commit**

```bash
git add src/ui/sidebar/mod.rs src/main.rs
git commit -m "feat: move feed/databases/clients into ConnectionState, gate subscription per connection"
```

---

### Task 4: Manual smoke test

- [ ] **Step 1: Run the app**

```bash
cargo run 2>&1
```

- [ ] **Step 2: Verify startup behavior**

On launch:
- Dialog is visible
- Feed is empty (no rows)
- Sidebar DATABASES section shows "0 dbs"
- Sidebar CLIENTS section is empty
- Sidebar CONNECTIONS section is empty

- [ ] **Step 3: Verify connect flow**

Click "Connect →" in the dialog, wait for the spinner, then click "Done":
- Dialog closes
- Connection appears in CONNECTIONS section (live dot, LIVE badge)
- Feed rows begin appearing within ~1 second
- DATABASES section populates as entries arrive
- CLIENTS section populates as entries arrive

- [ ] **Step 4: Verify capture toggle**

Click the capture indicator in the topbar to pause:
- Feed stops receiving new rows
- Existing rows remain

Click again to resume:
- Feed starts receiving rows again

- [ ] **Step 5: Commit if smoke test passes**

No new files to add — this step is validation only.

---

## Spec Coverage Check

| Spec requirement | Covered in |
|-----------------|-----------|
| `ConnectionState` owns `feed`, `databases`, `clients`, `capturing` | Task 1 |
| `SidebarState` holds `Vec<ConnectionState>` + `active_id` | Task 2 |
| `DialogDone` creates `ConnectionState` with `capturing: true` | Task 2 Step 2 |
| `App` removes global `feed`/`capturing`/`tx` | Task 3 Steps 2–3 |
| `QueriesReceived(usize, Vec<QueryEntry>)` routes by id | Task 3 Step 1, Step 4 |
| Subscription keyed per live connection | Task 3 Step 8 |
| Startup: empty connections, dialog opens, no data | Task 3 Step 3 + Task 4 Step 2 |
| `register_entries` on `ConnectionState` | Task 1 Step 4 |
| `saved_views` stays global | Task 2 Step 1 (unchanged field) |
| `InspectorState` stays on `App` | Task 3 Step 2 |
