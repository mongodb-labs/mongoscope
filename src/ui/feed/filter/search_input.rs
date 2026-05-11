use iced::{widget::{container, text_input}, Border, Element, Length, Padding};
use crate::theme::Palette;

pub fn search_input<'a, Msg: Clone + 'static>(
    value: &'a str,
    placeholder: &'static str,
    on_change: impl Fn(String) -> Msg + 'static,
    on_submit: Msg,
    palette: &Palette,
) -> Element<'a, Msg> {
    let bg2     = palette.bg2;
    let fg      = palette.fg;
    let fg_dim2 = palette.fg_dim2;
    let border  = palette.border;
    let accent  = palette.accent;

    container(
        text_input(placeholder, value)
            .size(12)
            .padding(Padding { top: 5.0, bottom: 5.0, left: 10.0, right: 10.0 })
            .on_input(on_change)
            .on_submit(on_submit)
            .font(iced::Font::MONOSPACE)
            .style(move |_, _| text_input::Style {
                background: iced::Background::Color(bg2),
                border: Border { color: border, width: 1.0, radius: 5.0.into() },
                icon: fg,
                placeholder: fg_dim2,
                value: fg,
                selection: accent,
            })
    )
    .width(Length::Fill)
    .into()
}
