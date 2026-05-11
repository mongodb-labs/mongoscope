# Mongoscope

Rust desktop MongoDB query debugger and traffic inspector. Built with iced 0.13 (Elm-style, `iced::application()`).

## Design reference

HTML/JSX design files live in `~/temp/mongoscope/` (Mongoscope.html, sidebar.jsx, feed.jsx, inspector.jsx, etc.).

**If that directory is missing or empty**, ask the user for the Anthropic design viewer link so you can re-fetch and extract it there.

Always follow the design. Do not deviate without explicit user permission. When permission is granted, log it here under "Approved deviations" so you don't need to ask again.

## Approved deviations

_(none yet)_

## Stack

- `iced 0.13` — features: canvas, tokio, lazy
- `nutype 0.5` — newtypes for QueryId, Timestamp, etc.
- `tokio` runtime via iced's built-in integration
- Mock data source at 2–3 entries/sec (`src/data/mock/`)

## Key patterns

- `Palette: Copy` — always pass by value, extract `Color` fields before `'static` closures
- `lazy(dep, |_| ...)` — memoize expensive subtrees; dep must change when content changes
- Views return `Element<'a, Msg>` when borrowing entry data, `Element<'static, Msg>` when using only owned/Copy data
- Style closures must be `'static` — never capture `&Palette` directly
