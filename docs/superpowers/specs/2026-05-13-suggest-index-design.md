# Suggest Index — Design Spec

**Date:** 2026-05-13  
**Branch:** `feature/suggest-index`  
**Designs:** `_designs/suggest-index/`

---

## Overview

When the inspector shows a COLLSCAN query, the Overview tab's warn banner displays a "Suggest index" button. Clicking it navigates to the Explain tab, which gains a new **suggested index** section showing a before/after plan comparison and a one-click index creation action.

All wiring is mock (no real MongoDB connection). Matches the mock-first pattern used throughout the app.

---

## Interaction Flow

1. User selects a COLLSCAN query in the feed → inspector opens on Overview tab.
2. Warn banner shows: `◆ <warn message>  [Suggest index]`
3. User clicks **Suggest index** → `InspectorMsg::SuggestIndex` fires → tab switches to `InspectorTab::Explain`.
4. Explain tab renders the new suggested-index section at the bottom of the suggestions card.

The button navigates only. No modal, no side panel.

---

## Explain Tab Changes

### Suggestions card — new section (COLLSCAN only)

Replaces the existing static "create index" text row with two sub-components:

#### 1. Before / after plan split

Two columns, same four stages in both:

| Stage | Before | After (est.) |
|-------|--------|-------------|
| COLLSCAN → IXSCAN | ~92% of total ms | ~1ms |
| FETCH | — | ~2ms |
| SORT | ~6% of total ms | ~1ms (fewer docs) |
| LIMIT | ~1ms | ~0ms |

- **Before** column: dimmed to 40% opacity after Run, label "before"
- **After** column: green border + subtle green background after Run, label "✓ index applied [EST.]"
  - Amber `EST.` badge inline in the label
  - Italic note below: *"actual speedup depends on data distribution"*
- Bar widths are proportional to ms within each column independently (each column's widest bar = 100%). The two columns do not share a scale — they show plan shape, not absolute comparison.
- Stage ms values are derived from existing `explain_stages()` mock data — no new mock data needed.

#### 2. Code pill

Single-row control:

```
[ index ] [ db.{coll}.createIndex({ {first_key}: 1 }) ] [ Copy ] [ ▶ Run ]
```

- Left badge: `index` label, green-tinted background.
- Middle: syntax-highlighted command — `db` dim, `.createIndex` in `tok_fn` colour, key in `tok_str`, `1` in `tok_num`.
- **Copy** button: writes the command string to clipboard via `iced::clipboard::write`.
- **▶ Run** button: sets `index_applied = true` immediately (mock). Becomes `✓ Created` (green-tinted) after click.

---

## State

New struct in `src/ui/inspector/tabs/explain.rs`:

```rust
#[derive(Debug, Clone, Default)]
pub struct ExplainState {
    pub index_applied: bool,
}
```

New message variants in `src/ui/inspector/tabs/mod.rs`:

```rust
pub enum ExplainMsg {
    CopyIndex,
    RunIndex,
}
```

`ExplainMsg` bubbles up through `InspectorMsg::Explain(ExplainMsg)` → `Message::Inspector`.

`InspectorState` gains:
- `pub explain: ExplainState`
- `pub enum InspectorMsg { ..., Explain(ExplainMsg), SuggestIndex }`

`InspectorState::update` handles:
- `SuggestIndex` → `self.tab = InspectorTab::Explain`
- `Explain(ExplainMsg::RunIndex)` → `self.explain.index_applied = true`
- `Explain(ExplainMsg::CopyIndex)` → return `Task` with clipboard write (handled in `main.rs`)

---

## View signature change

`explain_tab` currently takes no state. New signature:

```rust
pub fn explain_tab<Msg: Clone + 'static>(
    entry: &QueryEntry,
    state: &ExplainState,
    on_msg: impl Fn(ExplainMsg) -> Msg + 'static + Copy,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg>
```

The Overview tab's `overview_tab` function gains an `on_suggest` callback:

```rust
pub fn overview_tab<'a, Msg: Clone + 'static>(
    entry: &'a QueryEntry,
    on_suggest: impl Fn() -> Msg + 'static + Copy,
    palette: &Palette,
    fs: f32,
) -> Element<'a, Msg>
```

The existing inert `container("Suggest index")` in `overview.rs` becomes a `button` with `on_press(on_suggest())`.

---

## Clipboard Task

`main.rs` `update` match arm for `Message::Inspector(InspectorMsg::Explain(ExplainMsg::CopyIndex))`:

```rust
Message::Inspector(InspectorMsg::Explain(ExplainMsg::CopyIndex)) => {
    let cmd = /* build createIndex string from selected entry */;
    return iced::clipboard::write::<Message>(cmd);
}
```

---

## Files Changed

| File | Change |
|------|--------|
| `src/ui/inspector/tabs/explain.rs` | Add `ExplainState`, `ExplainMsg`, new suggested-index section in view |
| `src/ui/inspector/tabs/mod.rs` | Export `ExplainMsg`, `ExplainState` |
| `src/ui/inspector/mod.rs` | Add `InspectorMsg::Explain`, `InspectorMsg::SuggestIndex`; add `explain: ExplainState`; update `update()` and `view()` |
| `src/ui/inspector/tabs/overview.rs` | Wire "Suggest index" button with `on_suggest` callback |
| `src/main.rs` | Handle `ExplainMsg::CopyIndex` clipboard task |

---

## Designs

All mockup HTML and PNG screenshots are in `_designs/suggest-index/`:

- `approach-options.html/.png` — three interaction pattern options (A modal, B inline, C tab-nav)
- `explain-improvements.html/.png` — three Explain tab improvement directions
- `code-block-variants.html/.png` — three code presentation styles (terminal, pill, card)
- `final-design.html/.png` — approved final state: before/after + code pill + applied state with EST. badge
