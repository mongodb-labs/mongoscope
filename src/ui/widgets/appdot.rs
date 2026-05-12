use iced::{widget::container, Border, Color, Element};

/// 8×8 colored square with border_radius=2. Purely decorative — no message.
pub fn appdot<Msg: 'static>(color: [u8; 3]) -> Element<'static, Msg> {
    let c = Color::from_rgb8(color[0], color[1], color[2]);
    container(iced::widget::Space::new(8, 8))
        .width(8)
        .height(8)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(c)),
            border: Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
