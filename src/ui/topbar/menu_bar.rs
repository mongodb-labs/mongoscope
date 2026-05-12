use crate::theme::Palette;
use iced::{
    widget::{button, text},
    Border, Element, Padding,
};

#[derive(Debug, Clone)]
pub enum MenuMsg {
    File,
    Capture,
    Rules,
    View,
    Help,
}

fn menu_btn<Msg: Clone + 'static>(
    label: &str,
    msg: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let fg = palette.fg;
    let bg_hover = palette.bg_hover;
    let label = label.to_string();

    button(text(label).size(12).color(fg).font(iced::Font::MONOSPACE))
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 10.0,
        })
        .on_press(msg)
        .style(move |_, status| button::Style {
            background: match status {
                iced::widget::button::Status::Hovered => Some(iced::Background::Color(bg_hover)),
                _ => None,
            },
            border: Border {
                radius: 3.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

pub fn menu_bar<Msg: Clone + 'static>(
    on_msg: impl Fn(MenuMsg) -> Msg + 'static,
    palette: &Palette,
) -> Element<'static, Msg> {
    iced::widget::row![
        menu_btn("File", on_msg(MenuMsg::File), palette),
        menu_btn("Capture", on_msg(MenuMsg::Capture), palette),
        menu_btn("Rules", on_msg(MenuMsg::Rules), palette),
        menu_btn("View", on_msg(MenuMsg::View), palette),
        menu_btn("Help", on_msg(MenuMsg::Help), palette),
    ]
    .spacing(0)
    .align_y(iced::Alignment::Center)
    .into()
}
