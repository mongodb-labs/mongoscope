use crate::theme::Palette;
use iced::{
    widget::{container, row, text},
    Border, Element, Length, Padding,
};

pub struct StatusInfo {
    pub ops_per_sec: f32,
    pub query_count: usize,
    pub slow_count: usize,
    pub theme_label: &'static str,
    pub density_label: &'static str,
}

pub fn statusbar<Msg: Clone + 'static>(
    info: &StatusInfo,
    on_theme_toggle: Msg,
    on_density_toggle: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg = palette.bg1;
    let border_color = palette.border;
    let fs = 10.0;

    let sep = || {
        text(" · ")
            .size(fs)
            .color(palette.fg_dim2)
            .font(iced::Font::MONOSPACE)
    };

    let slow_color = if info.slow_count > 0 {
        palette.danger
    } else {
        palette.fg_dim2
    };

    container(
        row![
            text(format!("{:.0} ops/s", info.ops_per_sec))
                .size(fs)
                .color(palette.fg_dim)
                .font(iced::Font::MONOSPACE),
            sep(),
            text(format!("{} queries", info.query_count))
                .size(fs)
                .color(palette.fg_dim)
                .font(iced::Font::MONOSPACE),
            sep(),
            text(format!("{} slow", info.slow_count))
                .size(fs)
                .color(slow_color)
                .font(iced::Font::MONOSPACE),
            iced::widget::Space::new(Length::Fill, 0),
            iced::widget::button(
                text(info.density_label)
                    .size(fs)
                    .color(palette.fg_dim)
                    .font(iced::Font::MONOSPACE)
            )
            .padding(Padding {
                top: 1.0,
                bottom: 1.0,
                left: 6.0,
                right: 6.0
            })
            .on_press(on_density_toggle)
            .style(|_, _| iced::widget::button::Style {
                background: None,
                border: Border::default(),
                ..Default::default()
            }),
            sep(),
            iced::widget::button(
                text(info.theme_label)
                    .size(fs)
                    .color(palette.fg_dim)
                    .font(iced::Font::MONOSPACE)
            )
            .padding(Padding {
                top: 1.0,
                bottom: 1.0,
                left: 6.0,
                right: 6.0
            })
            .on_press(on_theme_toggle)
            .style(|_, _| iced::widget::button::Style {
                background: None,
                border: Border::default(),
                ..Default::default()
            }),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center)
        .height(24)
        .padding(Padding {
            left: 12.0,
            right: 8.0,
            top: 0.0,
            bottom: 0.0,
        }),
    )
    .width(Length::Fill)
    .height(24)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
