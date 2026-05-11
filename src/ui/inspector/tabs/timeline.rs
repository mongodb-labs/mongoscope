use iced::{widget::{column, scrollable, text}, Element, Length, Padding};
use crate::{
    data::model::QueryEntry,
    theme::Palette,
    ui::widgets::gantt::{gantt_row, GanttPhase},
};

pub fn timeline_tab<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let total_ms = entry.latency_ms.into_inner();

    let phases = mock_phases(total_ms);

    let mut children: Vec<Element<'static, Msg>> = vec![
        text("Query phases").size(fs - 1.0).color(palette.fg_dim2)
            .font(iced::Font::MONOSPACE).into(),
    ];

    for phase in phases {
        children.push(gantt_row(phase, palette, fs));
    }

    scrollable(
        column(children).spacing(4)
            .padding(Padding { top: 8.0, bottom: 8.0, left: 12.0, right: 12.0 })
    )
    .height(Length::Fill)
    .into()
}

fn mock_phases(total_ms: u32) -> Vec<GanttPhase> {
    use iced::Color;
    let t = total_ms as f32;
    // Phase offsets as fractions of total
    let fracs: &[(&str, f32, f32, [u8; 3])] = &[
        ("parse",   0.00, 0.02, [0x5B, 0x8D, 0xEE]),
        ("auth",    0.02, 0.05, [0xA8, 0x7C, 0xF6]),
        ("plan",    0.05, 0.12, [0xF7, 0xC4, 0x8F]),
        ("execute", 0.12, 0.90, [0x5B, 0xC8, 0xAF]),
        ("serial",  0.90, 0.97, [0xF4, 0x8F, 0x8F]),
        ("net",     0.97, 1.00, [0x8F, 0xC5, 0xF4]),
    ];

    fracs.iter().map(|(label, start, end, rgb)| GanttPhase {
        label: label.to_string(),
        offset_ms: (start * t) as u32,
        ms: ((end - start) * t).max(1.0) as u32,
        total_ms,
        color: Color::from_rgb8(rgb[0], rgb[1], rgb[2]),
    }).collect()
}
