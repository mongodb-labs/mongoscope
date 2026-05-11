use iced::{widget::{container, row, text}, Border, Element, Length, Padding};
use crate::theme::Palette;

pub struct SchemaField {
    pub name: String,
    pub depth: usize,
    pub type_str: String,
    pub coverage: f32,  // 0.0–1.0
    pub samples: Vec<String>,
}

pub fn schema_row<Msg: 'static>(field: SchemaField, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let indent = 4.0 + field.depth as f32 * 14.0;
    let prefix = if field.depth > 0 { "└─ ".to_string() } else { String::new() };
    let name_str = format!("{}{}", prefix, field.name);
    let pct_str = format!("{}%", (field.coverage * 100.0) as u32);
    let samples_str = field.samples.join(" · ");
    let accent = palette.accent;
    let bg2 = palette.bg2;

    let cov_bar = container(
        container(iced::widget::Space::new(Length::Fill, 4))
            .width(Length::FillPortion((field.coverage * 100.0) as u16))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(accent)),
                border: Border { radius: 2.0.into(), ..Default::default() },
                ..Default::default()
            })
    )
    .width(80)
    .height(4)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg2)),
        border: Border { radius: 2.0.into(), ..Default::default() },
        ..Default::default()
    });

    row![
        text(name_str).size(fs).color(palette.fg_dim).font(iced::Font::MONOSPACE)
            .width(180),
        text(field.type_str).size(10).color(palette.fg_dim2).font(iced::Font::MONOSPACE)
            .width(100),
        row![cov_bar, text(pct_str).size(10).color(palette.fg_dim).font(iced::Font::MONOSPACE)]
            .spacing(6).align_y(iced::Alignment::Center).width(120),
        text(samples_str).size(10).color(palette.fg_dim2).font(iced::Font::MONOSPACE)
            .width(Length::Fill),
    ]
    .padding(Padding { top: 4.0, bottom: 4.0, left: indent, right: 0.0 })
    .into()
}
