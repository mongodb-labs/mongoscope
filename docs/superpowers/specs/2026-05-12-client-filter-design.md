# Client Filter Feature Design

**Date:** 2026-05-12  
**Status:** Approved

## Summary

Wire the client sidebar panel to the feed filter bar, mirroring the existing db/coll filter pattern. Clicking a client in the sidebar injects an `app:NAME` token into the filter. Clicking the pill's × removes it. Radio-style selection (one client at a time).

## Changes

### 1. `src/ui/sidebar/mod.rs` — `active_client()`
Add method alongside `active_db`/`active_coll`:
```rust
pub fn active_client(&self) -> Option<String> {
    self.active()?.clients.iter().find(|c| c.active).map(|c| c.name.clone())
}
```

### 2. `src/ui/sidebar/mod.rs` — radio toggle for clients
Change `ClientsMsg::Toggle` handler: set clicked client active, deactivate all others. If already active, deactivate (toggle off). Matches `apply_toggle_db` behavior.

### 3. `src/ui/feed/filter/mod.rs` — `set_app()`
New method alongside `set_scope()`:
- Strip any existing `app:` token from `self.text`
- Inject `app:NAME` if `Some(name)`, else leave removed
- Re-parse `self.expr`
- Include unit test

### 4. `src/main.rs` — wire `SidebarMsg::Clients`
After `self.sidebar.update(m.clone())`, add:
```rust
if let SidebarMsg::Clients(_) = m {
    let app = self.sidebar.active_client();
    if let Some(conn) = self.sidebar.active_mut() {
        conn.feed.filter.set_app(app);
    }
}
```

## What works for free (no changes needed)
- `app:` is already a chip token — pill renders and × removal works
- `FilterExpr::matches()` already filters on `app:`
- Filter placeholder already shows `app:api`
