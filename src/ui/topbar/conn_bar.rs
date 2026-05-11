use iced::{widget::{container, row, text}, Border, Color, Element, Length};
use crate::theme::Palette;

pub struct ConnInfo {
    pub host: String,
    pub uri: String,
    pub rs_name: Option<String>,
    pub connected: bool,
}

pub fn conn_bar<Msg: 'static>(info: &ConnInfo, palette: &Palette) -> Element<'static, Msg> {
    let dot_color = if info.connected { palette.ok } else { palette.danger };
    let dot = container(iced::widget::Space::new(8, 8))
        .width(8)
        .height(8)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(dot_color)),
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        });

    let rs_label = info.rs_name.as_deref().unwrap_or("standalone");
    let label = format!("{} · {} · {}", info.host, info.uri, rs_label);

    row![
        dot,
        text(label).size(11).color(palette.fg_dim).font(iced::Font::MONOSPACE),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center)
    .width(Length::Shrink)
    .into()
}
