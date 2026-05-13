# Inspector Panel — Design Spec

**Date:** 2026-05-13
**Branch:** `feature/inspector-panel`

---

## Overview

Four improvements to the inspector panel:

1. Hide inspector until an entry is selected
2. Wire the close (✕) button
3. Remove the `"◉"` button; wire `"↗"/"↙"` as maximize/restore
4. Add a vertical drag handle to resize the inspector (like the sidebar)

All wiring is mock-compatible — no real backend changes needed.

---

## State

Replace the four flat fields (`inspector_open`, `inspector_height`, `inspector_maximized`, `inspector_dragging`) with a single enum in `App`:

```rust
pub enum InspectorPanel {
    Closed,
    Open { height: f32, dragging: bool },
    Maximized { prev_height: f32 },
}

impl Default for InspectorPanel {
    fn default() -> Self {
        InspectorPanel::Closed
    }
}
```

`App` gains `inspector_panel: InspectorPanel` (replaces nothing — this is new state).

---

## State Transitions

| From | Event | To |
|------|-------|----|
| `Closed` | entry selected in feed | `Open { height: 300.0, dragging: false }` |
| `Open` | ✕ pressed | `Closed` |
| `Maximized` | ✕ pressed | `Closed` |
| `Open { height }` | ↗ pressed | `Maximized { prev_height: height }` |
| `Maximized { prev_height }` | ↙ pressed | `Open { height: prev_height, dragging: false }` |
| `Open` | resize drag start | `Open { dragging: true }` |
| `Open { dragging: true }` | cursor moved | `Open { height: clamped(new_height) }` |
| `Open { dragging: true }` | mouse released | `Open { dragging: false }` |

Height clamped to `120.0..=600.0`.

---

## Messages

New top-level `Message` variants (mirror sidebar resize pattern):

```rust
InspectorResizeStart,
InspectorResizeMove(f32),   // raw cursor Y from window top
InspectorResizeEnd,
```

New `InspectorMsg` variants (routed through existing `Message::Inspector`):

```rust
InspectorMsg::Close,
InspectorMsg::ToggleMaximize,
```

`App::update` intercepts these before the catch-all `Message::Inspector(m)` arm.

---

## Layout (`App::view`)

```
match inspector_panel {
    Closed =>
        main_pane = feed only (Length::Fill), no resize handle

    Open { height, .. } =>
        main_pane = feed (Fill) + resize_handle (4px tall) + inspector (Fixed(height))

    Maximized { .. } =>
        main_pane = inspector only (Fill), feed hidden
}
```

The resize handle is a 4px-tall `mouse_area` container with `ResizingVertically` cursor, `on_press(InspectorResizeStart)`, `on_release(InspectorResizeEnd)`. Matches sidebar handle style (border color background).

---

## Resize Subscription

When `inspector_panel` is `Open { dragging: true }`, extend the existing subscription batch with a mouse listener:

```rust
CursorMoved { position } => Some(Message::InspectorResizeMove(position.y))
ButtonReleased(Left)     => Some(Message::InspectorResizeEnd)
```

`InspectorResizeMove(y)` handler computes new height as the distance from cursor to the bottom of the topbar (approximate): the layout height available is not directly known, so use the heuristic `new_height = last_known_window_height - y`. In practice, drag feels correct because as cursor moves up, `y` decreases and height increases.

Simpler: store nothing extra. Compute `new_height` by tracking the delta: on `ResizeStart`, record the cursor Y and current height; on `ResizeMove`, compute `new_height = start_height + (start_y - current_y)`. Clamp to `120..=600`.

Add to `App`:
```rust
inspector_resize_start: Option<(f32, f32)>,  // (cursor_y, height) at drag start
```

Or fold into the enum:
```rust
Open { height: f32, drag_start: Option<(f32, f32)> }
```

Using `drag_start: Option<(f32, f32)>` in the `Open` variant eliminates the separate field.

---

## Inspector Header Changes

File: `src/ui/inspector/header.rs`

Remove `"◉"`. Wire the remaining two buttons:

```rust
pub fn inspector_header<Msg: Clone + 'static>(
    entry: &QueryEntry,
    maximized: bool,
    on_close: Msg,
    on_maximize: Msg,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg>
```

- `"↗"` shown when `!maximized`, fires `on_maximize`
- `"↙"` shown when `maximized`, fires `on_maximize` (same message, toggle)
- `"✕"` fires `on_close`

---

## Auto-Open Behavior

In `Message::Feed` handler in `App::update`:

```rust
let new_selected = ...;
if new_selected != prev_selected && new_selected.is_some() {
    if matches!(self.inspector_panel, InspectorPanel::Closed) {
        self.inspector_panel = InspectorPanel::Open { height: 300.0, drag_start: None };
    }
}
```

Selecting a different entry while inspector is already open does not change its state (user may have resized or maximized it).

---

## Files Changed

| File | Change |
|------|--------|
| `src/main.rs` | Add `InspectorPanel` enum; add `inspector_panel` field; handle new messages; update layout logic; extend subscription |
| `src/ui/inspector/mod.rs` | Pass `maximized`, `on_close`, `on_maximize` to `inspector_header`; handle `InspectorMsg::Close` / `ToggleMaximize` |
| `src/ui/inspector/header.rs` | Remove `"◉"`; wire `"↗"/"↙"` and `"✕"` via callbacks |
