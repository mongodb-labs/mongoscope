use crate::theme::Palette;
use iced::{
    widget::{button, container, row, text},
    Border, Element, Padding,
};

pub fn capture_indicator<Msg: Clone + 'static>(
    capturing: bool,
    on_toggle: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    if !capturing {
        return iced::widget::Space::new(0, 0).into();
    }

    let danger = palette.danger;
    let accent_fg = palette.accent_fg;

    let pill = container(
        row![
            container(iced::widget::Space::new(6, 6))
                .width(6)
                .height(6)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(accent_fg)),
                    border: Border {
                        radius: 3.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            text("CAPTURING")
                .size(10)
                .color(accent_fg)
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
        background: Some(iced::Background::Color(danger)),
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
