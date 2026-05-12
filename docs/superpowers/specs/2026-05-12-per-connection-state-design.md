# Per-Connection State Design

**Date:** 2026-05-12
**Branch:** feat/per-connection-state

## Problem

Mock data starts recording the moment the app opens, before any connection is established. The connection dialog is still open, yet the feed fills, databases populate, and clients appear. This is unrealistic and will break when real connections are added.

The root cause: `FeedState`, `databases`, and `clients` are global on `App`/`SidebarState`. They are not owned by any connection, so they can't be scoped to one.

## Goal

- App opens → dialog shows, feed empty, sidebar has no databases/clients/connections
- `DialogDone` → connection added, capturing starts, data begins flowing into that connection's feed
- Selecting a different connection → UI switches to that connection's data (feed, databases, clients)
- Each connection is independent; its data is preserved when you switch away

## Data Model

### `ConnectionState` (new)

Owns everything scoped to one connection.

```rust
pub struct ConnectionState {
    pub item: ConnectionItem,          // label, uri, proxy_port, color, live
    pub feed: FeedState,
    pub databases: Vec<DatabaseItem>,
    pub clients: Vec<ClientItem>,
    pub capturing: bool,               // starts true; user can pause/resume
}
```

`ConnectionState::new(item: ConnectionItem) -> Self` — initializes with empty feed, empty databases, empty clients, `capturing: true`.

### `SidebarState` changes

```rust
pub struct SidebarState {
    pub connections: Vec<ConnectionState>,   // was: Vec<ConnectionItem> + separate db/client vecs
    pub active_id: Option<usize>,
    pub dialog: Option<ConnectionDialogState>,
    pub saved_views: Vec<SavedView>,         // unchanged: global, application setting
}
```

Remove from `SidebarState`:
- `databases: Vec<DatabaseItem>` — now on `ConnectionState`
- `clients: Vec<ClientItem>` — now on `ConnectionState`

Add helpers:
- `fn active(&self) -> Option<&ConnectionState>`
- `fn active_mut(&mut self) -> Option<&mut ConnectionState>`

### `App` changes

Remove from `App`:
- `capturing: bool` — now on `ConnectionState`
- `tx: Option<tokio::sync::mpsc::Sender<QueryEntry>>` — no longer needed (subscription uses `Subscription::run` keyed per connection)

`FeedState` moves to `ConnectionState`. `InspectorState` stays on `App` (it is pure UI tab/selection state, not connection data).

## Message Changes

### `QueriesReceived`

```rust
// before
Message::QueriesReceived(Vec<QueryEntry>)

// after
Message::QueriesReceived(usize, Vec<QueryEntry>)   // (connection_id, entries)
```

Routes incoming entries to the correct `ConnectionState` by id.

### `ToggleCapture`

Now targets the active connection:

```rust
Message::ToggleCapture
// handler: toggle self.sidebar.active_mut().map(|c| c.capturing = !c.capturing)
```

## Subscription

Each live connection runs its own stream, keyed by connection id so iced doesn't restart streams when other state changes.

```rust
fn subscription(&self) -> Subscription<Message> {
    let data_subs: Vec<Subscription<Message>> = self.sidebar.connections
        .iter()
        .filter(|c| c.item.live)
        .map(|c| {
            let id = c.item.id;
            Subscription::run_with_id(id, move || {
                iced::stream::channel(256, move |mut output| async move {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
                    Box::new(MockSource).start(tx);
                    loop {
                        // batch logic unchanged
                        // ...
                        let _ = output.send(Message::QueriesReceived(id, batch)).await;
                    }
                })
            })
        })
        .collect();

    // sidebar drag sub unchanged
    // ...

    Subscription::batch(data_subs)
}
```

When `live` becomes `false` (connection removed or disconnected), that stream's subscription is dropped automatically by iced on the next render cycle.

## Startup Behavior

`SidebarState::new()`:
- `connections: vec![]`
- `active_id: None`
- `dialog: None` — `App::new()` opens it because connections is empty
- `databases` and `clients` fields removed

`App::new()`:
- Remove `capturing: bool`
- Remove `tx`
- Open dialog because `sidebar.connections.is_empty()`

Result: app opens, dialog shows, feed is empty, subscription produces no streams (no live connections).

## DialogDone Flow

1. `ConnectionsMsg::DialogDone` fires in `SidebarState::update`
2. Build `ConnectionItem { live: true, active: true, ... }` from dialog state
3. Push `ConnectionState::new(item)` onto `connections` — `capturing` defaults to `true`
4. Set `active_id` to new connection's id
5. Clear `dialog`

No change needed in `App::update` for this path.

## View Wiring

The active connection's data feeds the existing view functions:

```
self.sidebar.active() -> Option<&ConnectionState>
  .feed        -> FeedState for feed view + inspector entry lookup
  .databases   -> databases panel
  .clients     -> clients panel
  .capturing   -> topbar capture indicator
```

When `active()` is `None` (no connections), feed and panels render empty/idle state. `App` passes `None` or a reference to the active connection into each view function.

`connections_panel` receives `items: Vec<&ConnectionItem>` extracted from `ConnectionState` slice — interface unchanged from the view's perspective.

## Sidebar View: Databases and Clients

`SidebarState::view` currently builds db/client panels from its own fields. After this change it pulls from `self.active()`:

```rust
let (dbs, clients, capturing) = self.active()
    .map(|c| (c.databases.as_slice(), c.clients.as_slice(), c.capturing))
    .unwrap_or((&[], &[], false));
```

## `register_entries`

Move from `SidebarState` to `ConnectionState`. Called in `App::update` on `QueriesReceived(id, entries)`:

```rust
if let Some(conn) = self.sidebar.connections.iter_mut().find(|c| c.item.id == id) {
    if conn.capturing {
        conn.register_entries(&entries);
        for entry in entries {
            conn.feed.push_entry(entry);
        }
    }
}
```

## What Stays the Same

- `FeedState` internals — no changes
- `InspectorState` — no changes
- `dialog.rs` — no changes
- `connections.rs` view (`connections_panel`) — receives `&[ConnectionItem]` slice, no changes
- `saved_views` — stays on `SidebarState`, unchanged
- Sidebar drag subscription — unchanged
- `DatabasesMsg`, `ClientsMsg` message handling — same logic, just on `ConnectionState` instead of `SidebarState`

## What Is Explicitly Out of Scope

- Real MongoDB connections (still mock)
- Per-connection filter state
- Disconnecting / removing a connection
- Connection color dot in the feed rows
