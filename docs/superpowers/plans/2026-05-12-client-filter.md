# Client Filter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the client sidebar panel to the feed filter bar so clicking a client injects `app:NAME` into the filter and clicking the pill removes it.

**Architecture:** Four targeted changes mirroring the existing db/coll pattern: make client toggle radio-style, add `active_client()` to `SidebarState`, add `set_app()` to `FilterState`, wire both in `main.rs`. Everything else (pill rendering, `×` removal, `FilterExpr::matches`) already works with `app:` tokens.

**Tech Stack:** Rust, iced 0.13

---

### Task 1: Make client toggle radio-style + add `active_client()`

**Files:**
- Modify: `src/ui/sidebar/mod.rs`

The `ClientsMsg::Toggle` handler currently does `c.active = !c.active` (multi-select). Change it to radio logic matching `apply_toggle_db`: clicked client toggles, all others deactivate. Then add `active_client()` alongside `active_db`/`active_coll`.

- [ ] **Step 1: Write failing test for radio toggle**

Add to the `#[cfg(test)]` block at the bottom of `src/ui/sidebar/mod.rs`:

```rust
#[test]
fn client_toggle_is_radio_style() {
    let mut s = SidebarState::default_for_test();
    // activate connection so active_mut() returns Some
    s.active_id = Some(0);
    let conn = s.connections[0].item.id; // just ensure there's a connection
    // set up two clients
    s.connections[0].clients = vec![
        crate::ui::sidebar::clients::ClientItem { name: "app1".into(), color: [0,0,0], active: false },
        crate::ui::sidebar::clients::ClientItem { name: "app2".into(), color: [0,0,0], active: false },
    ];
    s.update(SidebarMsg::Clients(ClientsMsg::Toggle("app1".into())));
    assert!(s.connections[0].clients[0].active);
    assert!(!s.connections[0].clients[1].active);

    s.update(SidebarMsg::Clients(ClientsMsg::Toggle("app2".into())));
    assert!(!s.connections[0].clients[0].active);
    assert!(s.connections[0].clients[1].active);

    // toggle active one off
    s.update(SidebarMsg::Clients(ClientsMsg::Toggle("app2".into())));
    assert!(!s.connections[0].clients[0].active);
    assert!(!s.connections[0].clients[1].active);
}

#[test]
fn active_client_returns_active_name() {
    let mut s = SidebarState::default_for_test();
    s.active_id = Some(0);
    s.connections[0].clients = vec![
        crate::ui::sidebar::clients::ClientItem { name: "myapp".into(), color: [0,0,0], active: false },
    ];
    assert_eq!(s.active_client(), None);
    s.connections[0].clients[0].active = true;
    assert_eq!(s.active_client(), Some("myapp".into()));
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -q -- client_toggle_is_radio_style active_client_returns_active_name 2>&1 | tail -20
```

Expected: FAIL — `active_client` doesn't exist yet, radio logic not yet implemented.

- [ ] **Step 3: Add `active_client()` method**

In `src/ui/sidebar/mod.rs`, after `active_coll()` (around line 83), add:

```rust
pub fn active_client(&self) -> Option<String> {
    self.active()?.clients.iter().find(|c| c.active).map(|c| c.name.clone())
}
```

- [ ] **Step 4: Change `ClientsMsg::Toggle` to radio logic**

In `src/ui/sidebar/mod.rs`, replace the `SidebarMsg::Clients(m)` arm (around lines 195–207):

```rust
SidebarMsg::Clients(m) => {
    if let Some(conn) = self.active_mut() {
        match m {
            ClientsMsg::Toggle(name) => {
                let was_active = conn.clients.iter().find(|c| c.name == name).map(|c| c.active).unwrap_or(false);
                for c in &mut conn.clients {
                    c.active = false;
                }
                if !was_active {
                    if let Some(c) = conn.clients.iter_mut().find(|c| c.name == name) {
                        c.active = true;
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 5: Check if `default_for_test` helper exists**

```bash
grep -n "default_for_test" /Users/jeroen.vervaeke/git/github.com/mongodb-labs/mongoscope/src/ui/sidebar/mod.rs
```

If it doesn't exist, add it inside the `#[cfg(test)]` block:

```rust
impl SidebarState {
    fn default_for_test() -> Self {
        use crate::ui::sidebar::connections::{ConnectionItem, ConnectionState};
        let item = ConnectionItem {
            id: 0,
            label: "test".into(),
            topology: "direct".into(),
            uri: "mongodb://localhost".into(),
            proxy_port: 27017,
            color: [100, 100, 200],
            active: true,
            live: true,
        };
        let mut s = SidebarState::new();
        s.connections.push(ConnectionState::new(item));
        s.active_id = Some(0);
        s
    }
}
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cargo test -q -- client_toggle_is_radio_style active_client_returns_active_name 2>&1 | tail -20
```

Expected: both PASS.

- [ ] **Step 7: Run full test suite + clippy**

```bash
cargo fmt && cargo build -q && cargo test -q 2>&1 | tail -20 && cargo clippy -- -D warnings 2>&1 | tail -20
```

Expected: no errors.

- [ ] **Step 8: Commit**

```bash
git checkout -b feat/client-filter
git add src/ui/sidebar/mod.rs
git commit -m "feat: radio-style client toggle + active_client() accessor"
```

---

### Task 2: Add `set_app()` to `FilterState`

**Files:**
- Modify: `src/ui/feed/filter/mod.rs`

Add `set_app()` alongside `set_scope()`. Strips any existing `app:` token, optionally injects a new one.

- [ ] **Step 1: Write failing test**

Add to the `#[cfg(test)]` block in `src/ui/feed/filter/mod.rs`:

```rust
#[test]
fn set_app_injects_app_token() {
    let mut fs = FilterState::new();
    fs.set_app(Some("myapi".into()));
    assert_eq!(fs.text, "app:myapi");
    assert_eq!(fs.expr.app, Some("myapi".into()));
}

#[test]
fn set_app_replaces_existing_app_token() {
    let mut fs = FilterState::new();
    fs.text = "db:shop app:old slow".into();
    fs.expr = FilterExpr::parse(&fs.text);
    fs.set_app(Some("newapi".into()));
    assert_eq!(fs.text, "db:shop slow app:newapi");
    assert_eq!(fs.expr.app, Some("newapi".into()));
}

#[test]
fn set_app_none_removes_token() {
    let mut fs = FilterState::new();
    fs.text = "db:shop app:old slow".into();
    fs.expr = FilterExpr::parse(&fs.text);
    fs.set_app(None);
    assert_eq!(fs.text, "db:shop slow");
    assert_eq!(fs.expr.app, None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -q -- set_app 2>&1 | tail -20
```

Expected: FAIL — `set_app` doesn't exist.

- [ ] **Step 3: Implement `set_app()`**

In `src/ui/feed/filter/mod.rs`, after `set_scope()` (around line 84), add:

```rust
pub fn set_app(&mut self, app: Option<String>) {
    let rest: String = self
        .text
        .split_whitespace()
        .filter(|t| !t.starts_with("app:"))
        .collect::<Vec<_>>()
        .join(" ");

    let mut parts: Vec<String> = Vec::new();
    if !rest.is_empty() {
        parts.push(rest);
    }
    if let Some(a) = app {
        parts.push(format!("app:{}", a));
    }

    self.text = parts.join(" ");
    self.expr = FilterExpr::parse(&self.text);
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -q -- set_app 2>&1 | tail -20
```

Expected: all three PASS.

- [ ] **Step 5: Run full test suite + clippy**

```bash
cargo fmt && cargo build -q && cargo test -q 2>&1 | tail -20 && cargo clippy -- -D warnings 2>&1 | tail -20
```

Expected: no errors.

- [ ] **Step 6: Commit**

```bash
git add src/ui/feed/filter/mod.rs
git commit -m "feat: add set_app() to FilterState for client filter injection"
```

---

### Task 3: Wire `SidebarMsg::Clients` in `main.rs`

**Files:**
- Modify: `src/main.rs`

Add the same pattern used for `SidebarMsg::Databases`: after sidebar update, check if it was a Clients message, extract active client, call `set_app()`.

- [ ] **Step 1: Add the wiring**

In `src/main.rs`, find the block after `self.sidebar.update(m.clone());` (around line 149). It currently looks like:

```rust
self.sidebar.update(m.clone());
if let SidebarMsg::Databases(_) = m {
    let (db, coll) = (self.sidebar.active_db(), self.sidebar.active_coll());
    if let Some(conn) = self.sidebar.active_mut() {
        conn.feed.filter.set_scope(db, coll);
    }
}
Task::none()
```

Add the client block immediately after the Databases block:

```rust
self.sidebar.update(m.clone());
if let SidebarMsg::Databases(_) = m {
    let (db, coll) = (self.sidebar.active_db(), self.sidebar.active_coll());
    if let Some(conn) = self.sidebar.active_mut() {
        conn.feed.filter.set_scope(db, coll);
    }
}
if let SidebarMsg::Clients(_) = m {
    let app = self.sidebar.active_client();
    if let Some(conn) = self.sidebar.active_mut() {
        conn.feed.filter.set_app(app);
    }
}
Task::none()
```

- [ ] **Step 2: Build and verify it compiles**

```bash
cargo build 2>&1 | tail -20
```

Expected: no errors.

- [ ] **Step 3: Run full test suite + clippy**

```bash
cargo fmt && cargo test -q 2>&1 | tail -20 && cargo clippy -- -D warnings 2>&1 | tail -20
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire client sidebar toggle to app: filter token"
```

---

### Task 4: Final verification

- [ ] **Step 1: Run full quality checks**

```bash
cargo fmt && cargo build && cargo test && cargo clippy -- -D warnings
```

Expected: all pass, zero warnings.

- [ ] **Step 2: Manual smoke test**

Run the app:
```bash
cargo run
```

Verify:
1. Click a client in sidebar → `app:NAME` pill appears in filter bar, feed filters to that client
2. Click same client again → pill disappears, feed unfiltered  
3. Click different client → pill switches to new client name
4. Click `×` on pill → pill disappears, client deselects in sidebar
5. Client filter + db filter coexist (both pills visible, both active)

- [ ] **Step 3: Done**

Feature complete. All four tasks merged on `feat/client-filter` branch.
