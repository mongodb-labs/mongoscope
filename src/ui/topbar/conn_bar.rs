use crate::theme::Palette;
use iced::{
    widget::{button, container, row, text},
    Border, Element, Length, Padding,
};

pub struct ConnInfo {
    pub uri: String,
    pub connected: bool,
}

pub fn conn_bar<Msg: Clone + 'static>(
    info: &ConnInfo,
    on_copy: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let dot_color = if info.connected {
        palette.ok
    } else {
        palette.danger
    };
    let dot = container(iced::widget::Space::new(8, 8))
        .width(8)
        .height(8)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(dot_color)),
            border: Border {
                radius: 4.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

    let uri = info.uri.clone();
    let fg = palette.fg_dim;
    let fg_dim = palette.fg_dim;
    let bg_hover = palette.bg2;

    let copy_btn = button(
        text("Copy")
            .size(10)
            .color(fg_dim)
            .font(iced::Font::MONOSPACE),
    )
    .on_press(on_copy)
    .padding(Padding {
        top: 2.0,
        bottom: 2.0,
        left: 6.0,
        right: 6.0,
    })
    .style(move |_, status| button::Style {
        background: match status {
            button::Status::Hovered => Some(iced::Background::Color(bg_hover)),
            _ => None,
        },
        border: Border {
            color: fg_dim,
            width: 1.0,
            radius: 3.0.into(),
        },
        text_color: fg_dim,
        ..Default::default()
    });

    row![
        dot,
        text(uri).size(11).color(fg).font(iced::Font::MONOSPACE),
        copy_btn,
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .width(Length::Shrink)
    .into()
}
