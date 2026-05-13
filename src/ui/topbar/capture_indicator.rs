use crate::theme::Palette;
use iced::{
    widget::{button, container, row, text},
    Border, Color, Element, Padding,
};

pub fn capture_indicator<Msg: Clone + 'static>(
    capturing: bool,
    on_toggle: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let (dot_color, label_color, bg_color, label) = if capturing {
        let danger = palette.danger;
        let accent_fg = palette.accent_fg;
        (accent_fg, accent_fg, danger, "CAPTURING")
    } else {
        let muted = palette.fg_dim;
        let bg = Color {
            a: 0.15,
            ..palette.fg_dim
        };
        (muted, muted, bg, "PAUSED")
    };

    let pill = container(
        row![
            container(iced::widget::Space::new(6, 6))
                .width(6)
                .height(6)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(dot_color)),
                    border: Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            text(label)
                .size(10)
                .color(label_color)
                .font(iced::Font::MONOSPACE),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center),
    )
    .padding(Padding {
        top: 3.0,
        bottom: 3.0,
        left: 8.0,
        right: 8.0,
    })
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg_color)),
        border: Border {
            radius: 10.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    button(pill)
        .padding(Padding::ZERO)
        .on_press(on_toggle)
        .style(|_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        })
        .into()
}
