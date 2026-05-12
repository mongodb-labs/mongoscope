# MCP Panel — Design Spec

**Date:** 2026-05-12  
**Status:** Approved

## Summary

Add an MCP (Model Context Protocol) button to the topbar that opens a right-side overlay drawer. The drawer lets users start/stop a mock MCP server and shows the tools available plus the `mcp.json` config snippet needed to connect an AI agent.

The feature is mock-only: no real network server is started. The UI simulates state transitions with a short delay to mimic real startup.

---

## Architecture

### New state on `App`

```rust
mcp_panel: McpPanelState
```

```rust
pub struct McpPanelState {
    pub open: bool,
    pub server: McpServerState,
}

pub enum McpServerState {
    Stopped,
    Starting,
    Running { port: u16 },  // mock port: 3717
}
```

### New messages

```rust
Message::McpToggle           // open/close drawer
Message::McpStartStop        // Stopped→Starting, Running→Stopped
Message::McpStarted          // fired after mock delay: Starting→Running
```

`McpStartStop` when `Starting` does nothing (button disabled).  
`McpStarted` is produced by `Task::perform` after ~800ms delay (same pattern as connection dialog).

### New files

| File | Purpose |
|---|---|
| `src/ui/topbar/mcp_button.rs` | Topbar button widget |
| `src/ui/mcp_panel.rs` | Drawer view + `McpPanelState` |

`McpPanelState` lives in `mcp_panel.rs` and is owned by `App`.

### `highlight_request` wiring

The MCP tool `highlight_request` maps directly to `Message::Feed(FeedMsg::Select(id))` on the active connection. No new infrastructure needed — the existing inspector selection mechanism handles it.

---

## Topbar Button (`mcp_button.rs`)

Small button matching the style of `capture_indicator`. Placed just left of the capture button.

| State | Dot colour | Label colour | Border |
|---|---|---|---|
| Stopped | Grey `#555` | `fg_dim` | `border` |
| Starting | Orange `palette.warn` | Orange | Orange |
| Running | Green `palette.ok` | Green | Green |

Clicking sends `Message::McpToggle`.

---

## Drawer (`mcp_panel.rs`)

Overlay on top of the full app body (sidebar + feed + inspector). Feed stays full width underneath.

**Layout:** absolute-positioned, right edge, full height of app body. Width: 300px.

**Backdrop:** `rgba(0,0,0,0.65)` — same alpha as connection dialog scrim. Clicking backdrop sends `Message::McpToggle` (closes).

### Header

- Title: `"MCP Server"`
- Status row: dot + label + port chip (port chip only visible when `Running`)
- ✕ close button (top-right), sends `Message::McpToggle`

### Body

**Tools section:**

| Tool | Description |
|---|---|
| `list_requests` | Get all captured requests & responses |
| `get_request` | Fetch full details of a request by ID |
| `highlight_request` | Select + highlight a row in the feed UI |

**Configure section** (`mcp.json`):

- Shown only when `Running`
- Code block with config snippet
- Copy button (top-right of code block) copies to clipboard
- When `Stopped` or `Starting`: placeholder text `"Port assigned on start"`

### Footer

Start/Stop button, full width:

| Server state | Button text | Style | Enabled |
|---|---|---|---|
| Stopped | `Start server` | Green fill | Yes |
| Starting | `Starting…` | Orange border, no fill | No |
| Running | `Stop server` | Neutral border | Yes |

---

## Connection Dialog — Backdrop Consistency

The existing `dialog_view` scrim uses `rgba(0,0,0,0.65)`. No change needed — it already matches the MCP drawer backdrop alpha. Both use `0.65`.

---

## Mock Behaviour

- **Start:** `McpStartStop` when `Stopped` → sets `Starting`, fires `Task::perform` with 800ms async sleep → produces `McpStarted` → sets `Running { port: 3717 }`
- **Stop:** `McpStartStop` when `Running` → sets `Stopped` immediately
- Port is always `3717` in mock mode

---

## Out of Scope

- Real MCP server (HTTP, SSE, tool dispatch)
- Actual `list_requests` / `get_request` / `highlight_request` tool handlers over the wire
- Port conflict detection
- Error state
