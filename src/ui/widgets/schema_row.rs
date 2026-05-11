use iced::{widget::{column, container, row, text}, Border, Color, Element, Length, Padding};
use crate::theme::Palette;

pub struct SchemaField {
    pub name: String,
    pub depth: usize,
    pub type_str: String,
    pub coverage: f32,
    pub samples: Vec<String>,
}

pub fn schema_row<Msg: 'static>(field: SchemaField, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let indent = 4.0 + field.depth as f32 * 14.0;
    let leaf = field.name.split('.').last().unwrap_or(&field.name).to_string();
    let prefix = if field.depth > 0 { "└─ " } else { "" };
    let name_str = format!("{}{}", prefix, leaf);
    let pct_str = format!("{}%", (field.coverage * 100.0) as u32);
    let samples_str = field.samples.join(" · ");
    let accent = palette.accent;
    let bg2 = palette.bg2;
    let border_c = Color { r: palette.border.r, g: palette.border.g, b: palette.border.b, a: 0.7 };

    let cov_fill = (field.coverage * 100.0) as u16;
    let cov_rest = 100u16.saturating_sub(cov_fill).max(1);

    let cov_bar = container(
        row![
            container(iced::widget::Space::new(Length::Fill, 4))
                .width(Length::FillPortion(cov_fill.max(1)))
                .height(4)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(accent)),
                    border: Border { radius: 2.0.into(), ..Default::default() },
                    ..Default::default()
                }),
            iced::widget::Space::new(Length::FillPortion(cov_rest), 4),
        ]
    )
    .width(80)
    .height(4)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg2)),
        border: Border { radius: 2.0.into(), ..Default::default() },
        ..Default::default()
    });

    let fs_mono = (fs - 0.5).max(9.0);

    column![
        row![
            container(
                text(name_str).size(fs_mono).color(palette.fg).font(iced::Font::MONOSPACE)
            )
            .padding(Padding { left: indent, top: 0.0, bottom: 0.0, right: 0.0 })
            .width(180),
            text(field.type_str).size(10.0).color(palette.fg_dim2).font(iced::Font::MONOSPACE)
                .width(100),
            row![
                cov_bar,
                text(pct_str).size(10.0).color(palette.fg_dim).font(iced::Font::MONOSPACE),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
            .width(120),
            text(samples_str).size(10.0).color(palette.fg_dim2).font(iced::Font::MONOSPACE)
                .width(Length::Fill),
        ]
        .padding(Padding { top: 4.0, bottom: 4.0, left: 0.0, right: 0.0 }),
        container(iced::widget::Space::new(Length::Fill, 0))
            .height(1)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(border_c)),
                ..Default::default()
            }),
    ]
    .into()
}
