use crate::theme::Palette;
use iced::{widget::container, Border, Element, Length, Padding};

/// Titled card shell. Children passed as a pre-built Element.
pub fn mini_card<Msg: 'static>(
    content: Element<'static, Msg>,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg = palette.bg1;
    let border_c = palette.border;

    container(content)
        .width(Length::Fill)
        .padding(Padding {
            top: 10.0,
            bottom: 10.0,
            left: 12.0,
            right: 12.0,
        })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border {
                color: border_c,
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        })
        .into()
}
