use crate::theme::Palette;
use iced::{
    widget::{button, column, container, row, text},
    Border, Color, Element, Length, Padding,
};

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

    // TODO: remove when real backend is wired up — currently all mock data
    #[allow(dead_code)]
    pub fn to_iced(self) -> Option<Color> {
        match self {
            ConnectionColor::None => None,
            ConnectionColor::Red => Some(Color::from_rgb8(0xc0, 0x39, 0x2b)),
            ConnectionColor::Orange => Some(Color::from_rgb8(0xe6, 0x7e, 0x22)),
            ConnectionColor::Green => Some(Color::from_rgb8(0x27, 0xae, 0x60)),
            ConnectionColor::Blue => Some(Color::from_rgb8(0x29, 0x80, 0xb9)),
            ConnectionColor::Purple => Some(Color::from_rgb8(0x8e, 0x44, 0xad)),
        }
    }
}

impl std::fmt::Display for ConnectionColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionColor::None => write!(f, "No color"),
            ConnectionColor::Red => write!(f, "Red"),
            ConnectionColor::Orange => write!(f, "Orange"),
            ConnectionColor::Green => write!(f, "Green"),
            ConnectionColor::Blue => write!(f, "Blue"),
            ConnectionColor::Purple => write!(f, "Purple"),
        }
    }
}

// ── Data ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
// TODO: remove when real backend is wired up — currently all mock data
#[allow(dead_code)]
pub struct ConnectionItem {
    pub id: usize,
    pub label: String,
    pub topology: String,
    pub uri: String,
    pub proxy_port: u16,
    pub color: ConnectionColor,
    pub active: bool,
    pub live: bool,
    pub shell_version: String,
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
    DialogNoop,
}

// ── Panel view ────────────────────────────────────────────────────────────────

pub fn connections_panel<Msg: Clone + 'static>(
    items: &[ConnectionItem],
    dialog_open: bool,
    on_msg: impl Fn(ConnectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0 = palette.bg;
    let bg_sel = palette.bg_sel;
    let bg_hover = palette.bg_hover;
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let ok = palette.ok;
    let accent = palette.accent;

    let rows: Vec<Element<Msg>> = items
        .iter()
        .map(|item| {
            let is_active = item.active;
            let bg = if is_active { bg_sel } else { bg0 };
            let dot_color = if item.live { ok } else { fg_dim2 };
            let id = item.id;
            let label = item.label.clone();
            let topo = item.topology.clone();
            let live = item.live;

            let dot = container(iced::widget::Space::new(7, 7))
                .width(7)
                .height(7)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(dot_color)),
                    border: Border {
                        radius: 3.5.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

            let mut inner = row![
                dot,
                column![
                    text(label).size(11).color(fg).font(iced::Font::MONOSPACE),
                    text(topo)
                        .size(10)
                        .color(fg_dim2)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(1)
                .width(Length::Fill),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center);

            if live {
                inner = inner.push(
                    container(
                        text("LIVE")
                            .size(9)
                            .color(accent)
                            .font(iced::Font::MONOSPACE),
                    )
                    .padding(Padding {
                        top: 1.0,
                        bottom: 1.0,
                        left: 4.0,
                        right: 4.0,
                    })
                    .style(move |_| container::Style {
                        border: Border {
                            color: accent,
                            width: 1.0,
                            radius: 2.0.into(),
                        },
                        ..Default::default()
                    }),
                );
            }

            button(inner)
                .padding(Padding {
                    top: 6.0,
                    bottom: 6.0,
                    left: 8.0,
                    right: 8.0,
                })
                .width(Length::Fill)
                .on_press(on_msg(ConnectionsMsg::Select(id)))
                .style(move |_, status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        iced::widget::button::Status::Hovered if !is_active => bg_hover,
                        _ => bg,
                    })),
                    border: Border::default(),
                    ..Default::default()
                })
                .into()
        })
        .collect();

    let add_label = if dialog_open { fg_dim2 } else { fg_dim };
    let add_btn = {
        let mut b = button(
            row![
                text("+")
                    .size(12)
                    .color(add_label)
                    .font(iced::Font::MONOSPACE),
                text("Add connection")
                    .size(11)
                    .color(add_label)
                    .font(iced::Font::MONOSPACE),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        )
        .padding(Padding {
            top: 5.0,
            bottom: 5.0,
            left: 8.0,
            right: 8.0,
        })
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
    col.padding(Padding {
        top: 4.0,
        bottom: 4.0,
        left: 0.0,
        right: 0.0,
    })
    .into()
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
