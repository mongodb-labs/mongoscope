# Connection Dialog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Two-step modal wizard for adding a MongoDB connection — step 1 collects target URI, step 2 displays the proxy URI to give the app. Dialog opens on launch (no connections) and via sidebar "+".

**Architecture:** Dialog state (`ConnectionDialogState`) lives on `SidebarState`. Dialog messages route through `SidebarMsg::Connections(ConnectionsMsg::Dialog*)`. `App::update` intercepts `DialogConnect` to spawn a stub async task (800 ms delay → `Ok(27117)`). Overlay renders via `stack![]` in `App::view`. No real TCP connection in this iteration.

**Tech Stack:** iced 0.13 (`stack`, `text_input`, `pick_list`, `container`, `column`, `row`), tokio async stub

---

## File Map

| Action | File | Responsibility |
|---|---|---|
| Modify | `src/ui/sidebar/connections.rs` | `ConnectionColor`, extended `ConnectionItem`, extended `ConnectionsMsg` |
| Create | `src/ui/dialog.rs` | `ConnectionDialogState`, `DialogStep`, all dialog view functions |
| Modify | `src/ui/mod.rs` | register `dialog` module |
| Modify | `src/ui/sidebar/mod.rs` | `SidebarState.dialog`, dialog message handling, startup auto-open |
| Modify | `src/main.rs` | `stack![]` overlay in `view()`, `DialogConnect` task, startup trigger |

---

## Task 1: `ConnectionColor`, extended `ConnectionItem`, extended `ConnectionsMsg`

**Files:**
- Modify: `src/ui/sidebar/connections.rs`

- [ ] **Step 1: Replace file contents**

Replace `src/ui/sidebar/connections.rs` entirely:

```rust
use iced::{
    widget::{button, column, container, pick_list, row, text},
    Border, Color, Element, Length, Padding,
};
use crate::theme::Palette;

// ── Color choice ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionColor {
    #[default]
    None,
    Red,
    Orange,
    Green,
    Blue,
    Purple,
}

impl ConnectionColor {
    pub const ALL: &'static [ConnectionColor] = &[
        ConnectionColor::None,
        ConnectionColor::Red,
        ConnectionColor::Orange,
        ConnectionColor::Green,
        ConnectionColor::Blue,
        ConnectionColor::Purple,
    ];

    pub fn to_iced(self) -> Option<Color> {
        match self {
            ConnectionColor::None   => None,
            ConnectionColor::Red    => Some(Color::from_rgb8(0xc0, 0x39, 0x2b)),
            ConnectionColor::Orange => Some(Color::from_rgb8(0xe6, 0x7e, 0x22)),
            ConnectionColor::Green  => Some(Color::from_rgb8(0x27, 0xae, 0x60)),
            ConnectionColor::Blue   => Some(Color::from_rgb8(0x29, 0x80, 0xb9)),
            ConnectionColor::Purple => Some(Color::from_rgb8(0x8e, 0x44, 0xad)),
        }
    }
}

impl std::fmt::Display for ConnectionColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionColor::None   => write!(f, "No color"),
            ConnectionColor::Red    => write!(f, "Red"),
            ConnectionColor::Orange => write!(f, "Orange"),
            ConnectionColor::Green  => write!(f, "Green"),
            ConnectionColor::Blue   => write!(f, "Blue"),
            ConnectionColor::Purple => write!(f, "Purple"),
        }
    }
}

// ── Data ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ConnectionItem {
    pub id: usize,
    pub label: String,
    pub topology: String,
    pub uri: String,
    pub proxy_port: u16,
    pub color: ConnectionColor,
    pub active: bool,
    pub live: bool,
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ConnectionsMsg {
    Select(usize),
    Add,
    // dialog
    DialogUriChanged(String),
    DialogNameChanged(String),
    DialogColorChanged(ConnectionColor),
    DialogConnect,
    DialogConnectResult(Result<u16, String>),
    DialogCopyUri,
    DialogBack,
    DialogDone,
    DialogCancel,
}

// ── Panel view ────────────────────────────────────────────────────────────────

pub fn connections_panel<Msg: Clone + 'static>(
    items: &[ConnectionItem],
    dialog_open: bool,
    on_msg: impl Fn(ConnectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0      = palette.bg;
    let bg_sel   = palette.bg_sel;
    let bg_hover = palette.bg_hover;
    let fg       = palette.fg;
    let fg_dim   = palette.fg_dim;
    let fg_dim2  = palette.fg_dim2;
    let ok       = palette.ok;
    let accent   = palette.accent;

    let rows: Vec<Element<Msg>> = items.iter().map(|item| {
        let is_active  = item.active;
        let bg         = if is_active { bg_sel } else { bg0 };
        let dot_color  = if item.live { ok } else { fg_dim2 };
        let id         = item.id;
        let label      = item.label.clone();
        let topo       = item.topology.clone();
        let live       = item.live;

        let dot = container(iced::widget::Space::new(7, 7))
            .width(7).height(7)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(dot_color)),
                border: Border { radius: 3.5.into(), ..Default::default() },
                ..Default::default()
            });

        let mut inner = row![
            dot,
            column![
                text(label).size(11).color(fg).font(iced::Font::MONOSPACE),
                text(topo).size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
            ].spacing(1).width(Length::Fill),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        if live {
            inner = inner.push(
                container(text("LIVE").size(9).color(accent).font(iced::Font::MONOSPACE))
                    .padding(Padding { top: 1.0, bottom: 1.0, left: 4.0, right: 4.0 })
                    .style(move |_| container::Style {
                        border: Border { color: accent, width: 1.0, radius: 2.0.into() },
                        ..Default::default()
                    })
            );
        }

        button(inner)
            .padding(Padding { top: 6.0, bottom: 6.0, left: 8.0, right: 8.0 })
            .width(Length::Fill)
            .on_press(on_msg(ConnectionsMsg::Select(id)))
            .style(move |_, status| button::Style {
                background: Some(iced::Background::Color(
                    match status {
                        iced::widget::button::Status::Hovered if !is_active => bg_hover,
                        _ => bg,
                    }
                )),
                border: Border::default(),
                ..Default::default()
            })
            .into()
    }).collect();

    let add_label = if dialog_open { fg_dim2 } else { fg_dim };
    let add_btn = {
        let mut b = button(
            row![
                text("+").size(12).color(add_label).font(iced::Font::MONOSPACE),
                text("Add connection").size(11).color(add_label).font(iced::Font::MONOSPACE),
            ].spacing(6).align_y(iced::Alignment::Center)
        )
        .padding(Padding { top: 5.0, bottom: 5.0, left: 8.0, right: 8.0 })
        .width(Length::Fill)
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        });
        if !dialog_open {
            b = b.on_press(on_msg(ConnectionsMsg::Add));
        }
        b
    };

    let mut col = column(rows).spacing(1);
    col = col.push(add_btn);
    col.padding(Padding { top: 4.0, bottom: 4.0, left: 0.0, right: 0.0 }).into()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_color_none_has_no_iced_color() {
        assert!(ConnectionColor::None.to_iced().is_none());
    }

    #[test]
    fn non_none_colors_have_iced_color() {
        for c in ConnectionColor::ALL.iter().skip(1) {
            assert!(c.to_iced().is_some(), "{c} should have a color");
        }
    }

    #[test]
    fn connection_color_display_none() {
        assert_eq!(ConnectionColor::None.to_string(), "No color");
    }

    #[test]
    fn connections_msg_add_is_clone() {
        let m = ConnectionsMsg::Add;
        let _ = m.clone();
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p mongoscope connections 2>&1 | tail -15
```

Expected: 4 tests pass, 0 failures. The crate will fail to compile until `connections_panel` callers are updated in Task 5 — that is fine, run with `-- --test-threads=1` if needed, or just proceed to Task 2.

Actually the compile will fail because `sidebar/mod.rs` calls `connections_panel` with the old signature. Accept that the project won't build until Task 5. Run only this module's tests:

```bash
cargo test -p mongoscope 'connections::tests' 2>&1 | tail -15
```

- [ ] **Step 3: Commit**

```bash
git add src/ui/sidebar/connections.rs
git commit -m "feat: ConnectionColor, extend ConnectionItem + ConnectionsMsg"
```

---

## Task 2: `ConnectionDialogState` + `DialogStep`

**Files:**
- Create: `src/ui/dialog.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create `src/ui/dialog.rs`** with state types only (view comes in Tasks 3–5):

```rust
use crate::ui::sidebar::connections::ConnectionColor;

#[derive(Debug, Clone, PartialEq)]
pub enum DialogStep {
    Step1 { connecting: bool },
    Step2,
}

#[derive(Debug, Clone)]
pub struct ConnectionDialogState {
    pub step: DialogStep,
    pub uri: String,
    pub name: String,
    pub color: ConnectionColor,
    pub error: Option<String>,
    pub proxy_port: u16,
}

impl ConnectionDialogState {
    pub fn new() -> Self {
        Self {
            step: DialogStep::Step1 { connecting: false },
            uri: "mongodb://localhost:27017/".into(),
            name: String::new(),
            color: ConnectionColor::None,
            error: None,
            proxy_port: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dialog_starts_step1_idle() {
        let d = ConnectionDialogState::new();
        assert_eq!(d.step, DialogStep::Step1 { connecting: false });
    }

    #[test]
    fn new_dialog_has_default_uri() {
        let d = ConnectionDialogState::new();
        assert_eq!(d.uri, "mongodb://localhost:27017/");
    }

    #[test]
    fn new_dialog_error_is_none() {
        let d = ConnectionDialogState::new();
        assert!(d.error.is_none());
    }

    #[test]
    fn new_dialog_proxy_port_zero() {
        let d = ConnectionDialogState::new();
        assert_eq!(d.proxy_port, 0);
    }
}
```

- [ ] **Step 2: Register the module**

In `src/ui/mod.rs`:

```rust
pub mod dialog;
pub mod feed;
pub mod inspector;
pub mod sidebar;
pub mod statusbar;
pub mod topbar;
pub mod widgets;
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p mongoscope 'dialog::tests' 2>&1 | tail -15
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/ui/dialog.rs src/ui/mod.rs
git commit -m "feat: ConnectionDialogState + DialogStep"
```

---

## Task 3: Step 1 view (form + help panel + connecting + error states)

**Files:**
- Modify: `src/ui/dialog.rs`

- [ ] **Step 1: Add imports and `step1_view` to `src/ui/dialog.rs`**

Add after the existing `impl ConnectionDialogState` block:

```rust
use iced::{
    widget::{button, column, container, pick_list, row, text, text_input},
    Alignment, Border, Color, Element, Length, Padding,
};
use crate::theme::Palette;
use crate::ui::sidebar::connections::ConnectionsMsg;

// ── Step 1 view ───────────────────────────────────────────────────────────────

pub fn step1_view<Msg: Clone + 'static>(
    state: &ConnectionDialogState,
    on_msg: impl Fn(ConnectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let connecting = matches!(state.step, DialogStep::Step1 { connecting: true });

    // palette values extracted before closures (Palette: Copy)
    let bg       = palette.bg;
    let bg1      = palette.bg1;
    let bg2      = palette.bg2;
    let fg       = palette.fg;
    let fg_dim   = palette.fg_dim;
    let fg_dim2  = palette.fg_dim2;
    let border   = palette.border;
    let border2  = palette.border2;
    let ok       = palette.ok;
    let danger   = palette.danger;
    let accent   = palette.accent;

    // field bg/border change when connecting
    let field_bg     = if connecting { Color { a: 0.5, ..bg2 } } else { bg1 };
    let field_border = if connecting { border } else { border2 };
    let field_fg     = if connecting { fg_dim2 } else { fg };

    // ── URI field ─────────────────────────────────────────────────────────────
    let uri_border_color = if state.error.is_some() { danger } else { field_border };
    let uri_val = state.uri.clone();

    let uri_input: Element<Msg> = if connecting {
        container(
            text(&uri_val).size(11).color(field_fg).font(iced::Font::MONOSPACE)
        )
        .width(Length::Fill)
        .padding(Padding::from([7, 8]))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(field_bg)),
            border: Border { color: field_border, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
    } else {
        text_input("mongodb://localhost:27017/", &uri_val)
            .on_input(move |s| on_msg(ConnectionsMsg::DialogUriChanged(s)))
            .padding(Padding::from([7, 8]))
            .size(11)
            .font(iced::Font::MONOSPACE)
            .style(move |_, _| text_input::Style {
                background: iced::Background::Color(bg1),
                border: Border { color: uri_border_color, width: 1.0, radius: 4.0.into() },
                icon: fg_dim2,
                placeholder: fg_dim2,
                value: fg,
                selection: ok,
            })
            .into()
    };

    let uri_label = row![
        text("Target URI").size(11).color(fg_dim).font(iced::Font::MONOSPACE),
        text(" ⓘ").size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
    ]
    .spacing(2)
    .align_y(Alignment::Center);

    let mut uri_col = column![uri_label, uri_input].spacing(5);

    // inline error message
    if let Some(err) = &state.error {
        let err_text = err.clone();
        uri_col = uri_col.push(
            row![
                text("✕").size(11).color(danger).font(iced::Font::MONOSPACE),
                text(err_text).size(10).color(danger).font(iced::Font::MONOSPACE),
            ]
            .spacing(5)
            .align_y(Alignment::Start)
        );
    }

    // ── Name field ────────────────────────────────────────────────────────────
    let name_val = state.name.clone();
    let name_input: Element<Msg> = if connecting {
        container(
            text(if name_val.is_empty() { "—".into() } else { name_val })
                .size(11).color(field_fg).font(iced::Font::MONOSPACE)
        )
        .width(Length::Fill)
        .padding(Padding::from([6, 8]))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(field_bg)),
            border: Border { color: field_border, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
    } else {
        text_input("My cluster", &name_val)
            .on_input(move |s| on_msg(ConnectionsMsg::DialogNameChanged(s)))
            .padding(Padding::from([6, 8]))
            .size(11)
            .font(iced::Font::MONOSPACE)
            .style(move |_, _| text_input::Style {
                background: iced::Background::Color(bg1),
                border: Border { color: border2, width: 1.0, radius: 4.0.into() },
                icon: fg_dim2,
                placeholder: fg_dim2,
                value: fg,
                selection: ok,
            })
            .into()
    };

    let name_col = column![
        text("Name").size(11).color(fg_dim).font(iced::Font::MONOSPACE),
        name_input,
    ].spacing(5).width(Length::Fill);

    // ── Color pick_list ───────────────────────────────────────────────────────
    let color_sel = state.color;
    let color_picker: Element<Msg> = if connecting {
        container(
            text(color_sel.to_string()).size(11).color(field_fg).font(iced::Font::MONOSPACE)
        )
        .width(150)
        .padding(Padding::from([6, 8]))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(field_bg)),
            border: Border { color: field_border, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
    } else {
        pick_list(
            ConnectionColor::ALL,
            Some(color_sel),
            move |c| on_msg(ConnectionsMsg::DialogColorChanged(c)),
        )
        .text_size(11)
        .text_font(iced::Font::MONOSPACE)
        .width(150)
        .style(move |_, _| pick_list::Style {
            text_color: fg,
            placeholder_color: fg_dim2,
            handle_color: fg_dim,
            background: iced::Background::Color(bg1),
            border: Border { color: border2, width: 1.0, radius: 4.0.into() },
        })
        .into()
    };

    let color_col = column![
        text("Color").size(11).color(fg_dim).font(iced::Font::MONOSPACE),
        color_picker,
    ].spacing(5);

    let name_color_row = row![name_col, color_col]
        .spacing(12)
        .align_y(Alignment::Start);

    // ── Connecting status ─────────────────────────────────────────────────────
    let mut form_col = column![uri_col, name_color_row].spacing(14);

    if connecting {
        let host = state.uri
            .trim_start_matches("mongodb://")
            .trim_start_matches("mongodb+srv://")
            .split('/').next()
            .unwrap_or(&state.uri)
            .to_owned();

        form_col = form_col.push(
            row![
                text("◌").size(13).color(ok).font(iced::Font::MONOSPACE),
                text(format!("Connecting to {}…", host))
                    .size(11).color(fg_dim).font(iced::Font::MONOSPACE),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
        );
    }

    // ── Help panel ────────────────────────────────────────────────────────────
    let help_panel = column![
        help_card(
            "Find connection string in Atlas",
            Some("Cluster view → Connect button → select driver"),
            palette,
        ),
        help_card(
            "Connection string format",
            None,
            palette,
        ),
    ]
    .spacing(12)
    .width(200);

    // ── Form + help layout ────────────────────────────────────────────────────
    let body = row![
        container(form_col)
            .width(Length::Fill)
            .padding(Padding { top: 18.0, bottom: 18.0, left: 24.0, right: 24.0 }),
        container(help_panel)
            .width(200)
            .height(Length::Fill)
            .padding(Padding { top: 18.0, bottom: 18.0, left: 16.0, right: 16.0 })
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg2)),
                border: Border { color: border, width: 1.0, radius: 0.0.into() },
                ..Default::default()
            }),
    ];

    body.into()
}
```

- [ ] **Step 2: Add `help_card` helper** (used by both step views) immediately before `step1_view`:

```rust
fn help_card<Msg: 'static>(
    title: &str,
    body: Option<&str>,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg2    = palette.bg2;
    let border = palette.border;
    let fg     = palette.fg;
    let fg_dim = palette.fg_dim;
    let ok     = palette.ok;

    let title_owned = title.to_owned();
    let body_owned  = body.map(str::to_owned);

    let mut col = column![
        text(title_owned).size(11).color(fg).font(iced::Font::MONOSPACE)
    ].spacing(4);

    if let Some(b) = body_owned {
        col = col.push(text(b).size(10).color(fg_dim).font(iced::Font::MONOSPACE));
    }

    col = col.push(text("See example ↗").size(10).color(ok).font(iced::Font::MONOSPACE));

    container(col)
        .padding(Padding::from([10, 12]))
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg2)),
            border: Border { color: border, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
}
```

- [ ] **Step 3: Check it compiles** (view functions can't be easily unit-tested; compilation is the check):

```bash
cargo check 2>&1 | head -30
```

Expected: errors only in `sidebar/mod.rs` (old `connections_panel` signature). No errors in `dialog.rs`.

- [ ] **Step 4: Commit**

```bash
git add src/ui/dialog.rs
git commit -m "feat: dialog step1_view with connecting and error states"
```

---

## Task 4: Step 2 view (proxy URI + routing table)

**Files:**
- Modify: `src/ui/dialog.rs`

- [ ] **Step 1: Add `step2_view` to `src/ui/dialog.rs`**

```rust
pub fn step2_view<Msg: Clone + 'static>(
    state: &ConnectionDialogState,
    on_msg: impl Fn(ConnectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg        = palette.bg;
    let bg1       = palette.bg1;
    let bg2       = palette.bg2;
    let fg        = palette.fg;
    let fg_dim    = palette.fg_dim;
    let fg_dim2   = palette.fg_dim2;
    let border    = palette.border;
    let border2   = palette.border2;
    let ok        = palette.ok;
    let ok_dim    = Color { a: 0.15, ..ok };
    let ok_border = Color { a: 0.4,  ..ok };

    // Extract host for display from original URI
    let target_host = state.uri
        .trim_start_matches("mongodb://")
        .trim_start_matches("mongodb+srv://")
        .split('/')
        .next()
        .unwrap_or(&state.uri)
        .to_owned();

    let proxy_port  = state.proxy_port;
    let proxy_uri   = format!("mongodb://localhost:{}/?directConnection=true", proxy_port);
    let proxy_uri2  = proxy_uri.clone();

    // ── Success banner ────────────────────────────────────────────────────────
    let banner = container(
        row![
            text("✓").size(14).color(ok).font(iced::Font::MONOSPACE),
            column![
                text(format!("Connected to {}", target_host))
                    .size(11).color(ok).font(iced::Font::MONOSPACE),
                text(format!("Mongoscope proxy is ready on port {}", proxy_port))
                    .size(10).color(fg_dim).font(iced::Font::MONOSPACE),
            ].spacing(2),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
    )
    .width(Length::Fill)
    .padding(Padding { top: 10.0, bottom: 10.0, left: 14.0, right: 14.0 })
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(ok_dim)),
        border: Border { color: ok_border, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    });

    // ── Proxy URI row ─────────────────────────────────────────────────────────
    let uri_display = container(
        text(proxy_uri).size(11).color(ok).font(iced::Font::MONOSPACE)
    )
    .width(Length::Fill)
    .padding(Padding::from([8, 12]))
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg1)),
        border: Border { color: border2, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    });

    let copy_btn = button(
        text("Copy").size(11).color(fg_dim).font(iced::Font::MONOSPACE)
    )
    .padding(Padding::from([6, 12]))
    .on_press(on_msg(ConnectionsMsg::DialogCopyUri))
    .style(move |_, _| button::Style {
        background: Some(iced::Background::Color(bg1)),
        border: Border { color: border2, width: 1.0, radius: 4.0.into() },
        text_color: fg_dim,
        ..Default::default()
    });

    let proxy_row = row![uri_display, copy_btn]
        .spacing(8)
        .align_y(Alignment::Center);

    let sub_label = text(
        "Same credentials — swap the host:port and add directConnection=true."
    )
    .size(10).color(fg_dim2).font(iced::Font::MONOSPACE);

    let proxy_section = column![
        text("Point your app to this URI instead")
            .size(11).color(fg_dim).font(iced::Font::MONOSPACE),
        proxy_row,
        sub_label,
    ].spacing(6);

    // ── Routing table ─────────────────────────────────────────────────────────
    let routing_table = container(
        column![
            routing_row("Your app connects to",
                        &format!("localhost:{}", proxy_port), fg_dim, fg, false),
            routing_row("Mongoscope proxies to",
                        &target_host, fg_dim, fg, false),
            routing_row("Traffic inspection",
                        "active", fg_dim, ok, true),
        ]
        .spacing(4)
    )
    .width(Length::Fill)
    .padding(Padding::from([14, 16]))
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg2)),
        border: Border { color: border, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    });

    container(
        column![banner, proxy_section, routing_table].spacing(18)
    )
    .width(Length::Fill)
    .padding(Padding { top: 20.0, bottom: 20.0, left: 24.0, right: 24.0 })
    .into()
}

fn routing_row<'a, Msg: 'static>(
    label: &str,
    value: &str,
    label_color: Color,
    value_color: Color,
    _bold: bool,
) -> Element<'static, Msg> {
    row![
        text(label.to_owned()).size(10).color(label_color).font(iced::Font::MONOSPACE)
            .width(Length::Fill),
        text(value.to_owned()).size(10).color(value_color).font(iced::Font::MONOSPACE),
    ]
    .align_y(Alignment::Center)
    .into()
}
```

- [ ] **Step 2: Compile check**

```bash
cargo check 2>&1 | grep 'dialog' | head -20
```

Expected: no errors in `dialog.rs`.

- [ ] **Step 3: Commit**

```bash
git add src/ui/dialog.rs
git commit -m "feat: dialog step2_view with proxy URI and routing table"
```

---

## Task 5: Dialog overlay (`dialog_view`) + step chip header

**Files:**
- Modify: `src/ui/dialog.rs`

- [ ] **Step 1: Add `dialog_view` to `src/ui/dialog.rs`**

This is the top-level function: scrim + modal shell + step chips + footer.

```rust
pub fn dialog_view<Msg: Clone + 'static>(
    state: &ConnectionDialogState,
    on_msg: impl Fn(ConnectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg       = palette.bg;
    let bg1      = palette.bg1;
    let fg       = palette.fg;
    let fg_dim   = palette.fg_dim;
    let fg_dim2  = palette.fg_dim2;
    let border   = palette.border;
    let ok       = palette.ok;
    let ok_dim   = Color { a: 0.3, ..ok };
    let accent   = palette.accent;

    let connecting = matches!(state.step, DialogStep::Step1 { connecting: true });
    let is_step2   = matches!(state.step, DialogStep::Step2);

    // ── Step chips ────────────────────────────────────────────────────────────
    // Approximated arrow: [active chip] › [inactive chip]
    let chip1_bg = ok;
    let chip1_fg = bg;
    let chip1_label = if is_step2 { "✓ Target server" } else { "1 · Target server" };

    let chip2_bg = if is_step2 { ok } else { bg1 };
    let chip2_fg = if is_step2 { bg } else { fg_dim2 };
    let chip2_label = "2 · Proxy string";

    let chip1_bg2 = if is_step2 { ok_dim } else { ok };
    let chip1_fg2 = if is_step2 { ok } else { bg };

    let chip = |label: String, bg_c: Color, fg_c: Color| -> Element<'static, Msg> {
        container(
            text(label).size(10).color(fg_c).font(iced::Font::MONOSPACE)
        )
        .padding(Padding { top: 3.0, bottom: 3.0, left: 10.0, right: 10.0 })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg_c)),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
    };

    let step_chips = row![
        chip(chip1_label.to_owned(), chip1_bg2, chip1_fg2),
        text("›").size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
        chip(chip2_label.to_owned(), chip2_bg, chip2_fg),
    ]
    .spacing(4)
    .align_y(Alignment::Center);

    // ── Modal header ──────────────────────────────────────────────────────────
    let header = container(
        row![
            column![
                text("New Connection")
                    .size(16).color(fg).font(iced::Font::MONOSPACE),
                step_chips,
            ]
            .spacing(8)
            .width(Length::Fill),
            button(text("✕").size(13).color(fg_dim2).font(iced::Font::MONOSPACE))
                .on_press(on_msg(ConnectionsMsg::DialogCancel))
                .padding(Padding::from([2, 6]))
                .style(move |_, _| button::Style {
                    background: None,
                    border: Border::default(),
                    text_color: fg_dim2,
                    ..Default::default()
                }),
        ]
        .align_y(Alignment::Start)
    )
    .width(Length::Fill)
    .padding(Padding { top: 18.0, bottom: 14.0, left: 24.0, right: 24.0 })
    .style(move |_| container::Style {
        border: Border { color: border, width: 0.0, radius: 0.0.into() },
        ..Default::default()
    });

    // ── Step body ─────────────────────────────────────────────────────────────
    let body: Element<Msg> = if is_step2 {
        step2_view(state, on_msg, palette)
    } else {
        step1_view(state, on_msg, palette)
    };

    // ── Footer ────────────────────────────────────────────────────────────────
    let cancel_btn = button(
        text("Cancel").size(11).color(fg_dim).font(iced::Font::MONOSPACE)
    )
    .on_press(on_msg(ConnectionsMsg::DialogCancel))
    .padding(Padding::from([5, 14]))
    .style(move |_, _| button::Style {
        background: None,
        border: Border { color: border, width: 1.0, radius: 4.0.into() },
        text_color: fg_dim,
        ..Default::default()
    });

    let footer: Element<Msg> = if is_step2 {
        let back_btn = button(
            text("← Back").size(11).color(fg_dim).font(iced::Font::MONOSPACE)
        )
        .on_press(on_msg(ConnectionsMsg::DialogBack))
        .padding(Padding::from([5, 14]))
        .style(move |_, _| button::Style {
            background: None,
            border: Border { color: border, width: 1.0, radius: 4.0.into() },
            text_color: fg_dim,
            ..Default::default()
        });

        let done_btn = button(
            text("Done").size(11).color(bg).font(iced::Font::MONOSPACE)
        )
        .on_press(on_msg(ConnectionsMsg::DialogDone))
        .padding(Padding::from([5, 14]))
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(ok)),
            border: Border::default(),
            text_color: bg,
            ..Default::default()
        });

        container(
            row![back_btn, iced::widget::Space::new(Length::Fill, 0), done_btn]
                .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .padding(Padding { top: 12.0, bottom: 12.0, left: 24.0, right: 24.0 })
        .style(move |_| container::Style {
            border: Border { color: border, width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    } else {
        let primary_label = if connecting { "Connecting…" } else { "Connect →" };
        let primary_bg = if connecting {
            Color { r: ok.r * 0.3, g: ok.g * 0.3, b: ok.b * 0.3, a: 1.0 }
        } else {
            ok
        };
        let primary_fg = if connecting { Color { a: 0.5, ..bg } } else { bg };

        let mut connect_btn = button(
            text(primary_label).size(11).color(primary_fg).font(iced::Font::MONOSPACE)
        )
        .padding(Padding::from([5, 14]))
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(primary_bg)),
            border: Border::default(),
            text_color: primary_fg,
            ..Default::default()
        });

        if !connecting {
            connect_btn = connect_btn.on_press(on_msg(ConnectionsMsg::DialogConnect));
        }

        container(
            row![cancel_btn, iced::widget::Space::new(Length::Fill, 0), connect_btn]
                .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .padding(Padding { top: 12.0, bottom: 12.0, left: 24.0, right: 24.0 })
        .style(move |_| container::Style {
            border: Border { color: border, width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    };

    // ── Modal card ────────────────────────────────────────────────────────────
    let divider_style = move |_: &_| container::Style {
        background: Some(iced::Background::Color(border)),
        ..Default::default()
    };

    let modal = container(
        column![
            header,
            container(iced::widget::Space::new(Length::Fill, 1.0))
                .width(Length::Fill).style(divider_style),
            body,
            container(iced::widget::Space::new(Length::Fill, 1.0))
                .width(Length::Fill).style(divider_style),
            footer,
        ]
        .spacing(0)
    )
    .width(700)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border { color: border, width: 1.0, radius: 8.0.into() },
        ..Default::default()
    });

    // ── Scrim + centered modal ────────────────────────────────────────────────
    let scrim_color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.6 };

    container(modal)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(scrim_color)),
            ..Default::default()
        })
        .into()
}
```

- [ ] **Step 2: Compile check**

```bash
cargo check 2>&1 | grep 'dialog' | head -20
```

Expected: no errors in `dialog.rs`.

- [ ] **Step 3: Commit**

```bash
git add src/ui/dialog.rs
git commit -m "feat: dialog_view with scrim overlay, step chips, footer"
```

---

## Task 6: Sidebar wiring

**Files:**
- Modify: `src/ui/sidebar/mod.rs`

- [ ] **Step 1: Add dialog field and update `SidebarState`**

In `src/ui/sidebar/mod.rs`, update imports and `SidebarState`:

At the top, add:
```rust
use crate::ui::dialog::ConnectionDialogState;
```

Change `SidebarState` struct:
```rust
pub struct SidebarState {
    pub databases: Vec<DatabaseItem>,
    pub connections: Vec<ConnectionItem>,
    pub clients: Vec<ClientItem>,
    pub saved_views: Vec<SavedView>,
    pub dialog: Option<ConnectionDialogState>,
}
```

- [ ] **Step 2: Update `SidebarState::new`**

Remove the hardcoded mock `ConnectionItem` (empty connections triggers auto-open). Update `new()`:

```rust
pub fn new() -> Self {
    Self {
        connections: vec![],   // empty → dialog opens on launch
        dialog: None,
        databases: vec![
            // ... keep existing database mock data unchanged ...
        ],
        clients: vec![],
        saved_views: vec![
            SavedView { id: 0, label: "slow queries (>500ms)".into() },
            SavedView { id: 1, label: "COLLSCANs only".into() },
            SavedView { id: 2, label: "writes to orders".into() },
        ],
    }
}
```

- [ ] **Step 3: Update `connections_panel` call in `view()`**

Find the `connections_panel(...)` call in `SidebarState::view` and add `self.dialog.is_some()`:

```rust
connections_panel(
    &self.connections,
    self.dialog.is_some(),
    move |m| on_msg(SidebarMsg::Connections(m)),
    palette,
),
```

- [ ] **Step 4: Handle all dialog messages in `update()`**

Replace the `SidebarMsg::Connections(m)` arm in `update()`:

```rust
SidebarMsg::Connections(m) => match m {
    ConnectionsMsg::Select(id) => {
        for c in &mut self.connections {
            c.active = c.id == id;
        }
    }
    ConnectionsMsg::Add => {
        if self.dialog.is_none() {
            self.dialog = Some(ConnectionDialogState::new());
        }
    }
    ConnectionsMsg::DialogUriChanged(s) => {
        if let Some(d) = &mut self.dialog {
            d.uri = s;
            d.error = None; // clear error on edit
        }
    }
    ConnectionsMsg::DialogNameChanged(s) => {
        if let Some(d) = &mut self.dialog {
            d.name = s;
        }
    }
    ConnectionsMsg::DialogColorChanged(c) => {
        if let Some(d) = &mut self.dialog {
            d.color = c;
        }
    }
    ConnectionsMsg::DialogConnect => {
        // Set connecting state; App::update handles the async task
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
            let next_id = self.connections.iter().map(|c| c.id).max().unwrap_or(0) + 1;
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
            self.connections.push(ConnectionItem {
                id: next_id,
                label,
                topology,
                uri: d.uri.clone(),
                proxy_port: d.proxy_port,
                color: d.color,
                active: true,
                live: true,
            });
            // deactivate previous
            let last = self.connections.len() - 1;
            for (i, c) in self.connections.iter_mut().enumerate() {
                c.active = i == last;
            }
        }
        self.dialog = None;
    }
    ConnectionsMsg::DialogCancel => {
        self.dialog = None;
    }
    ConnectionsMsg::DialogCopyUri => {
        // handled in App::update to produce clipboard Task
    }
},
```

- [ ] **Step 5: Compile check**

```bash
cargo check 2>&1 | head -30
```

Expected: errors only in `main.rs` (overlay not wired yet). No errors in `sidebar/`.

- [ ] **Step 6: Commit**

```bash
git add src/ui/sidebar/mod.rs
git commit -m "feat: wire dialog state into SidebarState"
```

---

## Task 7: App wiring (overlay + async stub + startup)

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add imports**

At the top of `src/main.rs`, add to the existing `use` block:

```rust
use iced::widget::stack;
use std::time::Duration;
use ui::{
    dialog::dialog_view,
    sidebar::connections::ConnectionsMsg,
    // ... existing imports ...
};
```

Full updated import block:

```rust
use iced::{mouse, widget::{column, container, mouse_area, row, scrollable, stack}, Element, Length, Subscription, Task};
use data::{mock::MockSource, model::QueryEntry, source::DataSource, types::QueryId};
use theme::{Density, Dock, Palette, Theme};
use ui::{
    dialog::dialog_view,
    feed::{FeedMsg, FeedState, FEED_SCROLL_ID},
    inspector::{InspectorMsg, InspectorState},
    sidebar::{connections::ConnectionsMsg, SidebarMsg, SidebarState},
    statusbar::{statusbar, StatusInfo},
    topbar::{conn_bar::ConnInfo, topbar, MenuMsg},
};
```

- [ ] **Step 2: Add `Message::CopyToClipboard` variant**

In the `Message` enum:

```rust
#[derive(Debug, Clone)]
enum Message {
    QueriesReceived(Vec<QueryEntry>),
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
    CopyToClipboard(String),
}
```

- [ ] **Step 3: Open dialog on startup if no connections**

In `App::new()`, after constructing `app`, check connections and open dialog:

```rust
fn new() -> (Self, Task<Message>) {
    let mut app = Self {
        feed: FeedState::new(),
        inspector: InspectorState::new(),
        sidebar: SidebarState::new(),
        theme: Theme::Dark,
        density: Density::Compact,
        capturing: true,
        tx: None,
        sidebar_width: 220.0,
        sidebar_dragging: false,
    };
    // Auto-open dialog on launch when no connections saved
    if app.sidebar.connections.is_empty() {
        app.sidebar.dialog = Some(ui::dialog::ConnectionDialogState::new());
    }
    (app, Task::none())
}
```

- [ ] **Step 4: Handle `DialogConnect` and `CopyToClipboard` in `update()`**

In `App::update`, update the `Message::Sidebar` arm to intercept `DialogConnect` and `DialogCopyUri` before delegating:

```rust
Message::Sidebar(ref m) => {
    // Intercept messages that need to produce Tasks
    match m {
        SidebarMsg::Connections(ConnectionsMsg::DialogConnect) => {
            self.sidebar.update(m.clone());
            // Stub: 800ms delay then success on port 27117
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
                return iced::clipboard::write(uri);
            }
        }
        _ => {}
    }

    // Default: delegate to sidebar
    self.sidebar.update(m.clone());
    if let SidebarMsg::Databases(_) = m {
        self.feed.filter.set_scope(
            self.sidebar.active_db(),
            self.sidebar.active_coll(),
        );
    }
}
Message::CopyToClipboard(_) => {} // consumed by clipboard::write task
```

- [ ] **Step 5: Add dialog overlay in `view()`**

Replace the final `container(column![top, body, status]...)` in `view()` with:

```rust
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
    let palette = self.theme.palette();
    stack![
        base,
        dialog_view(
            dialog_state,
            |m| Message::Sidebar(SidebarMsg::Connections(m)),
            &palette,
        )
    ]
    .into()
} else {
    base
}
```

- [ ] **Step 6: Build and run**

```bash
cargo build 2>&1 | tail -20
```

Expected: clean build, 0 errors.

```bash
cargo run
```

Expected: app opens, dialog appears immediately (no connections on startup). Fill in URI, click "Connect →", see "Connecting…" for ~800ms, then step 2 with proxy URI. "Copy" copies to clipboard. "Done" adds connection to sidebar. "+" button opens dialog again.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire dialog overlay and async connect stub into App"
```

---

## Self-Review Notes

- **Spec coverage:** All spec sections covered — step 1 (form, connecting, error), step 2 (banner, proxy URI, routing table), startup auto-open, "+" disable when open, Back/Done/Cancel/Copy all wired.
- **Proxy port:** Hardcoded `27117` in stub — spec explicitly defers real port assignment. No ambiguity.
- **Arrow chip shape:** CSS `clip-path` not available in iced; approximated with colored containers + `›` separator. Visually close, functionally identical.
- **`DialogCopyUri` in `sidebar.update`:** Has no-op arm — the actual clipboard task fires in `App::update` before sidebar sees it. Sidebar arm is unreachable but present for exhaustiveness.
- **`Palette: Copy`** — all color values extracted before closures per project pattern. No `&Palette` captured in `'static` closures.
