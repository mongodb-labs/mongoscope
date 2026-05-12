use iced::{
    widget::{button, column, container, pick_list, row, text, text_input},
    Alignment, Border, Color, Element, Length, Padding,
};
use crate::theme::Palette;
use crate::ui::sidebar::connections::ConnectionColor;
use crate::ui::sidebar::connections::ConnectionsMsg;

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

pub fn step1_view<Msg: Clone + 'static>(
    state: &ConnectionDialogState,
    on_msg: impl Fn(ConnectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let connecting = matches!(state.step, DialogStep::Step1 { connecting: true });

    // palette values extracted before closures (Palette: Copy)
    let bg1      = palette.bg1;
    let bg2      = palette.bg2;
    let fg       = palette.fg;
    let fg_dim   = palette.fg_dim;
    let fg_dim2  = palette.fg_dim2;
    let border   = palette.border;
    let border2  = palette.border2;
    let ok       = palette.ok;
    let danger   = palette.danger;

    // field bg/border change when connecting
    let field_bg     = if connecting { Color { a: 0.5, ..bg2 } } else { bg1 };
    let field_border = if connecting { border } else { border2 };
    let field_fg     = if connecting { fg_dim2 } else { fg };

    // ── URI field ─────────────────────────────────────────────────────────────
    let uri_border_color = if state.error.is_some() { danger } else { field_border };
    let uri_val = state.uri.clone();

    let uri_input: Element<Msg> = if connecting {
        container(
            text(uri_val.clone()).size(11).color(field_fg).font(iced::Font::MONOSPACE)
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
        .font(iced::Font::MONOSPACE)
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

pub fn step2_view<Msg: Clone + 'static>(
    state: &ConnectionDialogState,
    on_msg: impl Fn(ConnectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
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

    let target_host = state.uri
        .trim_start_matches("mongodb://")
        .trim_start_matches("mongodb+srv://")
        .split('/')
        .next()
        .unwrap_or(&state.uri)
        .to_owned();

    let proxy_port = state.proxy_port;
    let proxy_uri  = format!("mongodb://localhost:{}/?directConnection=true", proxy_port);

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
