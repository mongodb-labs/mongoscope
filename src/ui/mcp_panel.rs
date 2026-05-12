use iced::{
    widget::{button, column, container, mouse_area, row, text},
    Alignment, Background, Border, Color, Element, Font, Length, Padding,
};
use crate::theme::Palette;

#[derive(Debug, Clone, PartialEq)]
pub enum McpServerState {
    Stopped,
    Starting,
    Running { port: u16 },
}

#[derive(Debug, Clone)]
pub struct McpPanelState {
    pub open: bool,
    pub server: McpServerState,
}

#[derive(Debug, Clone)]
pub enum McpMsg {
    Toggle,
    StartStop,
    Started,
    CopyConfig,
    Noop,
}

impl McpPanelState {
    pub fn new() -> Self {
        Self { open: false, server: McpServerState::Stopped }
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Returns true if caller should fire async start task.
    pub fn begin_start(&mut self) -> bool {
        if matches!(self.server, McpServerState::Stopped) {
            self.server = McpServerState::Starting;
            true
        } else {
            false
        }
    }

    pub fn on_started(&mut self, port: u16) {
        if matches!(self.server, McpServerState::Starting) {
            self.server = McpServerState::Running { port };
        }
    }

    pub fn stop(&mut self) {
        if matches!(self.server, McpServerState::Running { .. }) {
            self.server = McpServerState::Stopped;
        }
    }
}

fn status_dot<Msg: 'static>(color: Color) -> Element<'static, Msg> {
    container(iced::widget::Space::new(0, 0))
        .width(7)
        .height(7)
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            border: Border { radius: 3.5.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

fn port_chip<Msg: 'static>(port: u16, palette: &Palette) -> Element<'static, Msg> {
    let ok  = palette.ok;
    let bg  = Color { a: 0.15, ..ok };
    let bdr = Color { a: 0.40, ..ok };
    container(
        text(format!(":{port}")).size(10).color(ok).font(Font::MONOSPACE)
    )
    .padding(Padding { top: 2.0, bottom: 2.0, left: 8.0, right: 8.0 })
    .style(move |_| container::Style {
        background: Some(Background::Color(bg)),
        border: Border { color: bdr, width: 1.0, radius: 3.0.into() },
        ..Default::default()
    })
    .into()
}

fn tool_row_el<Msg: 'static>(name: &str, desc: &str, palette: &Palette) -> Element<'static, Msg> {
    let name_color = palette.op_read;
    let desc_color = palette.fg_dim2;
    column![
        text(name.to_owned()).size(11).color(name_color).font(Font::MONOSPACE),
        text(desc.to_owned()).size(10).color(desc_color).font(Font::MONOSPACE),
    ]
    .spacing(2)
    .into()
}

fn config_section_view<Msg: Clone + 'static>(
    state: &McpPanelState,
    on_msg: impl Fn(McpMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let fg_dim  = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let bg1     = palette.bg1;
    let border2 = palette.border2;

    // sec_label helper (duplicated here for independence from overlay_view)
    let sec_label = |s: &'static str| -> Element<'static, Msg> {
        container(
            text(s).size(9).color(fg_dim2).font(Font::MONOSPACE)
        )
        .width(Length::Fill)
        .padding(Padding { top: 0.0, bottom: 4.0, left: 0.0, right: 0.0 })
        .into()
    };

    if let McpServerState::Running { port } = &state.server {
        let p = *port;
        let config_text = format!(
            "\"mongoscope\": {{\n  \"url\": \"http://localhost:{p}/mcp\"\n}}"
        );
        let code_block = container(
            text(config_text).size(10).color(fg_dim).font(Font::MONOSPACE)
        )
        .width(Length::Fill)
        .padding(Padding::from([10, 12]))
        .style(move |_| container::Style {
            background: Some(Background::Color(bg1)),
            border: Border { color: border2, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        });

        let copy_btn = button(
            text("Copy").size(10).color(fg_dim2).font(Font::MONOSPACE)
        )
        .on_press(on_msg(McpMsg::CopyConfig))
        .padding(Padding::from([4, 10]))
        .style(move |_, _| button::Style {
            background: Some(Background::Color(bg1)),
            border: Border { color: border2, width: 1.0, radius: 3.0.into() },
            text_color: fg_dim2,
            ..Default::default()
        });

        column![
            sec_label("CONFIGURE IN MCP.JSON"),
            code_block,
            container(copy_btn)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Right),
        ]
        .spacing(6)
        .into()
    } else {
        let pending_color = Color { a: 0.4, ..fg_dim2 };
        container(
            text("Port assigned on start")
                .size(10)
                .color(pending_color)
                .font(Font::MONOSPACE),
        )
        .width(Length::Fill)
        .padding(Padding::from([14, 12]))
        .style(move |_| container::Style {
            background: Some(Background::Color(bg1)),
            border: Border { color: border2, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
    }
}

fn footer_view<Msg: Clone + 'static>(
    state: &McpPanelState,
    on_msg: impl Fn(McpMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg1    = palette.bg1;
    let fg_dim = palette.fg_dim;
    let border = palette.border;
    let ok     = palette.ok;
    let warn   = palette.warn;

    let (btn_text, btn_bg, btn_fg, btn_border, btn_enabled) = match &state.server {
        McpServerState::Stopped => (
            "Start server", ok, palette.accent_fg, Color::TRANSPARENT, true
        ),
        McpServerState::Starting => (
            "Starting…", bg1, warn, warn, false
        ),
        McpServerState::Running { .. } => (
            "Stop server", bg1, fg_dim, border, true
        ),
    };

    let mut footer_btn = button(
        text(btn_text).size(11).color(btn_fg).font(Font::MONOSPACE)
    )
    .padding(Padding { top: 7.0, bottom: 7.0, left: 0.0, right: 0.0 })
    .style(move |_, _| button::Style {
        background: Some(Background::Color(btn_bg)),
        border: Border { color: btn_border, width: 1.0, radius: 3.0.into() },
        text_color: btn_fg,
        ..Default::default()
    });

    if btn_enabled {
        footer_btn = footer_btn.on_press(on_msg(McpMsg::StartStop));
    }

    container(
        container(footer_btn).width(Length::Fill)
    )
    .width(Length::Fill)
    .padding(Padding::from([12, 16]))
    .style(move |_| container::Style {
        border: Border { color: border, width: 1.0, radius: 0.0.into() },
        ..Default::default()
    })
    .into()
}

pub fn overlay_view<Msg: Clone + 'static>(
    state: &McpPanelState,
    on_msg: impl Fn(McpMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg2     = palette.bg2;
    let fg      = palette.fg;
    let fg_dim2 = palette.fg_dim2;
    let border  = palette.border;
    let ok      = palette.ok;
    let warn    = palette.warn;

    // ── Status row ────────────────────────────────────────────────────────────
    let (dot_color, status_label, status_color) = match &state.server {
        McpServerState::Stopped        => (palette.fg_dim2, "Stopped",   palette.fg_dim2),
        McpServerState::Starting       => (warn,            "Starting…", warn),
        McpServerState::Running { .. } => (ok,              "Running",   ok),
    };

    let mut status_row_content = row![
        status_dot(dot_color),
        text(status_label).size(11).color(status_color).font(Font::MONOSPACE),
    ]
    .spacing(7)
    .align_y(Alignment::Center);

    if let McpServerState::Running { port } = &state.server {
        status_row_content = status_row_content.push(port_chip(*port, palette));
    }

    // ── Header ────────────────────────────────────────────────────────────────
    let close_btn = button(
        text("✕").size(12).color(fg_dim2).font(Font::MONOSPACE)
    )
    .on_press(on_msg(McpMsg::Toggle))
    .padding(Padding::from([2, 6]))
    .style(move |_, _| button::Style {
        background: None,
        border: Border::default(),
        text_color: fg_dim2,
        ..Default::default()
    });

    let header = container(
        row![
            column![
                text("MCP Server").size(13).color(fg).font(Font::MONOSPACE),
                status_row_content,
            ]
            .spacing(8)
            .width(Length::Fill),
            close_btn,
        ]
        .align_y(Alignment::Start),
    )
    .width(Length::Fill)
    .padding(Padding { top: 14.0, bottom: 12.0, left: 16.0, right: 16.0 });

    // ── Section label helper ──────────────────────────────────────────────────
    let sec_label = |s: &'static str| -> Element<'static, Msg> {
        container(
            text(s).size(9).color(fg_dim2).font(Font::MONOSPACE)
        )
        .width(Length::Fill)
        .padding(Padding { top: 0.0, bottom: 4.0, left: 0.0, right: 0.0 })
        .into()
    };

    // ── Tools section ─────────────────────────────────────────────────────────
    let tools = column![
        sec_label("AVAILABLE TOOLS"),
        tool_row_el("list_requests",     "Get all captured requests & responses", palette),
        tool_row_el("get_request",       "Fetch full details of a request by ID", palette),
        tool_row_el("highlight_request", "Select + highlight a row in the feed UI", palette),
    ]
    .spacing(8);

    // ── Config section ────────────────────────────────────────────────────────
    let config_section = config_section_view(state, on_msg, palette);

    // ── Body ──────────────────────────────────────────────────────────────────
    let body = container(
        column![tools, config_section].spacing(18)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding::from([14, 16]));

    // ── Footer button ─────────────────────────────────────────────────────────
    let footer = footer_view(state, on_msg, palette);

    // ── Drawer panel ──────────────────────────────────────────────────────────
    let divider_style = move |_: &_| container::Style {
        background: Some(Background::Color(border)),
        ..Default::default()
    };

    let drawer_content = column![
        header,
        container(iced::widget::Space::new(Length::Fill, 1.0))
            .width(Length::Fill).style(divider_style),
        body,
        container(iced::widget::Space::new(Length::Fill, 1.0))
            .width(Length::Fill).style(divider_style),
        footer,
    ]
    .spacing(0);

    let drawer = mouse_area(
        container(drawer_content)
            .width(300)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(bg2)),
                border: Border { color: border, width: 1.0, radius: 0.0.into() },
                ..Default::default()
            }),
    )
    .on_press(on_msg(McpMsg::Noop));

    let drawer_positioned = container(drawer)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right);

    // ── Backdrop ──────────────────────────────────────────────────────────────
    let scrim = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.65 };

    let backdrop = mouse_area(
        container(iced::widget::Space::new(Length::Fill, Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(scrim)),
                ..Default::default()
            }),
    )
    .on_press(on_msg(McpMsg::Toggle));

    iced::widget::stack![backdrop, drawer_positioned].into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_panel_is_closed_and_stopped() {
        let p = McpPanelState::new();
        assert!(!p.open);
        assert_eq!(p.server, McpServerState::Stopped);
    }

    #[test]
    fn toggle_opens_and_closes() {
        let mut p = McpPanelState::new();
        p.toggle(); assert!(p.open);
        p.toggle(); assert!(!p.open);
    }

    #[test]
    fn begin_start_transitions_to_starting_and_returns_true() {
        let mut p = McpPanelState::new();
        let fired = p.begin_start();
        assert!(fired);
        assert_eq!(p.server, McpServerState::Starting);
    }

    #[test]
    fn begin_start_while_starting_does_nothing() {
        let mut p = McpPanelState::new();
        p.begin_start();
        let fired = p.begin_start();
        assert!(!fired);
        assert_eq!(p.server, McpServerState::Starting);
    }

    #[test]
    fn on_started_transitions_to_running_port_3717() {
        let mut p = McpPanelState::new();
        p.begin_start();
        p.on_started(3717);
        assert_eq!(p.server, McpServerState::Running { port: 3717 });
    }

    #[test]
    fn stop_transitions_running_to_stopped() {
        let mut p = McpPanelState::new();
        p.begin_start();
        p.on_started(3717);
        p.stop();
        assert_eq!(p.server, McpServerState::Stopped);
    }

    #[test]
    fn stop_while_stopped_does_nothing() {
        let mut p = McpPanelState::new();
        p.stop();
        assert_eq!(p.server, McpServerState::Stopped);
    }
}
