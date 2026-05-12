# Connection Dialog Design

**Date:** 2026-05-12
**Status:** Approved

## Overview

Two-step modal wizard for adding a new MongoDB connection. Triggered by the "+" button in the sidebar CONNECTIONS panel. Mongoscope is a MITM proxy, so step 1 collects the target server, step 2 presents the proxy URI the user's app should use.

Visual reference: `_designs/connection-dialog/connection-wizard.html`  
Screenshot: `_designs/connection-dialog/screenshot-both-steps.png`

---

## Step 1 — Target server

**Trigger:** `ConnectionsMsg::Add` from sidebar "+" button.

**Fields:**

| Field | Type | Required | Notes |
|---|---|---|---|
| Target URI | textarea | yes | Pre-filled with `mongodb://localhost:27017/` |
| Name | text input | no | Displayed in sidebar connection list |
| Color | dropdown | no | Accent color for the connection item; default "No color" |

**Header:** "New Connection" title + two arrow-shaped step chips: `[1 · Target server →] [2 · Proxy string]`. Step 1 chip is green/active, step 2 is dim/inactive.

**Right panel (help):** Two static help cards:
- "Find connection string in Atlas" — brief instruction + "See example ↗" link
- "Connection string format" — "See example ↗" link

**Footer:** `Cancel` (left) | `Connect →` (right, primary green).

**On "Connect →":** Attempt TCP connection to the target. On success, advance to step 2. On failure, show an inline error below the URI field (no step change).

---

## Step 2 — Proxy string

**Shown after successful connection in step 1.**

**Header:** Same "New Connection" title. Step chips: `[✓ Target server] [2 · Proxy string →]`. Step 1 chip is dim-green (done), step 2 is active green.

**Success banner:** Green-tinted row showing "Connected to `<host>`" + "Mongoscope proxy is ready on port `<proxy_port>`".

**Proxy URI field:**
- Label: "Point your app to this URI instead"
- Value: `mongodb://localhost:<proxy_port>/?directConnection=true`
- Read-only display + **Copy** button (copies to clipboard)
- Sub-label: "Same credentials — swap the host:port and add `directConnection=true`."

**Routing summary table (three rows):**

| Label | Value |
|---|---|
| Your app connects to | `localhost:<proxy_port>` |
| Mongoscope proxies to | `<target_host>:<target_port>` |
| Traffic inspection | `active` (green) |

**Footer:** `← Back` (left, returns to step 1) | `Done` (right, primary green — saves connection and closes dialog).

---

## Proxy port assignment

Mongoscope picks a free local port when the connection is established (step 1 → step 2 transition). The port is bound to the connection and persists for the session. Port selection strategy is TBD at implementation time (e.g. start from 27117, increment until free).

---

## Data model additions

`ConnectionItem` (existing) needs:

```rust
pub proxy_port: u16,       // assigned at connect time
pub uri: String,           // raw target URI as entered
pub color: Option<Color>,  // accent color; None = no color
```

---

## Messages

```rust
pub enum ConnectionsMsg {
    Select(usize),          // existing
    Add,                    // existing — opens dialog step 1
    DialogUriChanged(String),
    DialogNameChanged(String),
    DialogColorChanged(Option<Color>),
    DialogConnect,          // "Connect →" pressed
    DialogConnectResult(Result<u16, String>), // proxy_port or error
    DialogBack,             // "← Back" from step 2
    DialogDone,             // "Done" — save & close
    DialogCancel,           // "Cancel" or ✕
}
```

---

## Dialog state

New `ConnectionDialogState` struct, held in `SidebarState`:

```rust
pub struct ConnectionDialogState {
    pub step: DialogStep,       // Step1 | Step2
    pub uri: String,
    pub name: String,
    pub color: Option<Color>,
    pub error: Option<String>,  // step 1 connection error
    pub proxy_port: u16,        // populated on success
}

pub enum DialogStep { Step1, Step2 }
```

Dialog is absent (`None`) when closed, `Some(ConnectionDialogState)` when open.

---

## Rendering

Dialog renders as a modal overlay on top of the full app. In iced 0.13, implement as a `Stack` or `container` overlay in `App::view` that conditionally renders the dialog widget when `sidebar.dialog.is_some()`. The scrim (dimmed background) is a full-screen semi-transparent container behind the dialog.

Dialog width: ~700px. Modal centered. No minimum height constraint — content drives height.

---

## Edge cases

- **Dialog already open:** The sidebar "+" button is disabled (or no-ops) when `sidebar.dialog.is_some()`. Only one dialog at a time.
- **Connect fails:** Error string shown inline below the URI field. Step does not advance. User can edit the URI and retry.
- **Back from step 2:** Returns to step 1 with URI/Name/Color fields preserved. Does not disconnect the already-established proxy (proxy is torn down only on Cancel or window close).

---

## Scope exclusions (not in this iteration)

- "Edit Connection String" toggle (form fields vs raw URI) — raw URI only
- Advanced connection options (TLS, auth, SSH tunnel)
- "Favorite this connection" / pin to top
- "Save" without connecting
- "Test Connection" button (Connect → is the test)
- Editing existing connections
- Deleting connections
