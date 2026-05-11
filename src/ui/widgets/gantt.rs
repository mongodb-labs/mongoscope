use iced::{widget::{container, row, text}, Border, Color, Element, Length, Padding};
use crate::theme::Palette;

pub struct GanttPhase {
    pub label: String,
    pub ms: u32,
    pub offset_ms: u32,
    pub total_ms: u32,
    pub color: Color,
}

pub fn gantt_row<Msg: 'static>(phase: GanttPhase, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let left_pct = if phase.total_ms == 0 { 0.0f32 }
        else { (phase.offset_ms as f32 / phase.total_ms as f32 * 100.0).max(0.0) };
    let width_pct = if phase.total_ms == 0 { 1.0f32 }
        else { (phase.ms as f32 / phase.total_ms as f32 * 100.0).max(1.0) };

    let color = phase.color;
    let accent_fg = palette.accent_fg;
    let bg2 = palette.bg2;

    // We use a layered approach: bg track, then overlay bar using FillPortion
    // Left spacer + bar + right spacer
    let left_u = left_pct as u16;
    let bar_u = width_pct as u16;
    let right_u = 100u16.saturating_sub(left_u + bar_u);

    let bar = container(
        text(format!("{}ms", phase.ms)).size(10).color(accent_fg).font(iced::Font::MONOSPACE)
    )
    .height(18)
    .padding(Padding { left: 5.0, right: 5.0, top: 0.0, bottom: 0.0 })
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(color)),
        border: Border { radius: 3.0.into(), ..Default::default() },
        ..Default::default()
    });

    let track_inner = row![
        iced::widget::Space::new(Length::FillPortion(left_u.max(0) + 1), 18),
        bar.width(Length::FillPortion(bar_u.max(1))),
        iced::widget::Space::new(Length::FillPortion(right_u.max(0) + 1), 18),
    ];

    let track = container(track_inner)
        .height(18)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg2)),
            border: Border { radius: 3.0.into(), ..Default::default() },
            ..Default::default()
        });

    row![
        text(phase.label).size(fs).color(palette.fg_dim).font(iced::Font::MONOSPACE).width(90),
        track,
    ]
    .spacing(10)
    .align_y(iced::Alignment::Center)
    .into()
}
