use iced::{widget::{container, row, text}, Border, Color, Element, Length, Padding};
use crate::theme::Palette;

pub fn logo<Msg: 'static>(palette: &Palette) -> Element<'static, Msg> {
    let accent = palette.accent;
    let square = container(iced::widget::Space::new(14, 14))
        .width(14)
        .height(14)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(accent)),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        });

    row![
        square,
        text("Mongoscope")
            .size(13)
            .color(palette.fg)
            .font(iced::Font::MONOSPACE),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .padding(Padding { left: 12.0, right: 8.0, top: 0.0, bottom: 0.0 })
    .width(Length::Shrink)
    .into()
}
