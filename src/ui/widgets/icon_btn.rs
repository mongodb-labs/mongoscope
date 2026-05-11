use iced::{widget::button, Border, Element, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone, Copy)]
pub enum IconSize { Normal, Small }

impl IconSize {
    fn px(self) -> f32 { match self { IconSize::Normal => 24.0, IconSize::Small => 20.0 } }
}

pub fn icon_button<Msg: Clone + 'static>(
    label: &str,
    size: IconSize,
    msg: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let dim = size.px();
    let bg = palette.bg;
    let border_c = palette.border;
    let fg = palette.fg_dim;

    button(
        iced::widget::text(label.to_string())
            .size(11)
            .color(fg)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
    )
    .width(dim)
    .height(dim)
    .padding(Padding::ZERO)
    .on_press(msg)
    .style(move |_, _| button::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border { color: border_c, width: 1.0, radius: 5.0.into() },
        text_color: fg,
        ..Default::default()
    })
    .into()
}
