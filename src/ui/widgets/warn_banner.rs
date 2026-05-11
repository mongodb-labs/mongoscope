use iced::{widget::{column, container, row, text}, Border, Color, Element, Length, Padding};
use crate::theme::Palette;
use super::ghost_btn::{ghost_button, GhostVariant};

pub fn warn_banner<Msg: Clone + 'static>(
    title: &str,
    subtitle: &str,
    action_label: &str,
    action_msg: Msg,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let bg = Color { r: palette.warn.r, g: palette.warn.g, b: palette.warn.b, a: 0.14 };
    let border_c = Color { r: palette.warn.r, g: palette.warn.g, b: palette.warn.b, a: 0.30 };

    let body = row![
        text("◆").size(13).color(palette.warn),
        column![
            text(title.to_string()).size(fs).color(palette.warn),
            text(subtitle.to_string()).size(fs).color(palette.fg_dim),
        ]
        .spacing(2)
        .width(Length::Fill),
        ghost_button(action_label, GhostVariant::Default, action_msg, palette, fs),
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center);

    container(body)
        .width(Length::Fill)
        .padding(Padding { top: 10.0, bottom: 10.0, left: 12.0, right: 12.0 })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: border_c, width: 1.0, radius: 6.0.into() },
            ..Default::default()
        })
        .into()
}
