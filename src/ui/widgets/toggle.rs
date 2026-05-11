use iced::{widget::{button, container}, Border, Color, Element, Padding};
use crate::theme::Palette;

/// 24×14 pill toggle. on=true → accent fill; off → border2 fill.
pub fn toggle<Msg: Clone + 'static>(on: bool, msg: Msg, palette: &Palette) -> Element<'static, Msg> {
    let track_color = if on { palette.accent } else { palette.border2 };
    let knob_x: f32 = if on { 10.0 } else { 0.0 };

    // Knob
    let knob = container(iced::widget::Space::new(10, 10))
        .width(10)
        .height(10)
        .style(|_| container::Style {
            background: Some(iced::Background::Color(Color::WHITE)),
            border: Border { radius: 5.0.into(), ..Default::default() },
            ..Default::default()
        });

    let track_content = container(knob)
        .width(24)
        .height(14)
        .padding(Padding { top: 2.0, bottom: 2.0, left: 2.0 + knob_x, right: 0.0 })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(track_color)),
            border: Border { radius: 7.0.into(), ..Default::default() },
            ..Default::default()
        });

    button(track_content)
        .padding(Padding::ZERO)
        .width(24)
        .height(14)
        .on_press(msg)
        .style(|_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        })
        .into()
}
