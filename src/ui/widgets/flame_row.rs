use iced::{widget::{container, row, text}, Border, Color, Element, Length, Padding};
use crate::theme::Palette;

pub enum FlameRowKind { Ok, Warn, Bad }

pub struct FlameRowData {
    pub name: String,
    pub ms: u32,
    pub total_ms: u32,
    pub docs: Option<u64>,
    pub note: Option<String>,
    pub kind: FlameRowKind,
}

pub fn flame_row<Msg: 'static>(data: FlameRowData, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let color = match data.kind {
        FlameRowKind::Ok => palette.ok,
        FlameRowKind::Warn => palette.warn,
        FlameRowKind::Bad => palette.danger,
    };
    let label_color = match data.kind {
        FlameRowKind::Ok => palette.fg,
        FlameRowKind::Warn => palette.warn,
        FlameRowKind::Bad => palette.danger,
    };

    let pct = if data.total_ms == 0 { 3.0f32 }
    else { ((data.ms as f32 / data.total_ms as f32) * 100.0).max(3.0) };

    let fill_pct = (pct as u16).max(1);
    let rest_pct = 100u16.saturating_sub(fill_pct).max(1);

    let fill_label = Color { r: palette.accent_fg.r, g: palette.accent_fg.g, b: palette.accent_fg.b, a: 1.0 };
    let bg2 = palette.bg2;

    let bar_fill = container(
        text(format!("{}ms", data.ms)).size(10).color(fill_label).font(iced::Font::MONOSPACE)
    )
    .padding(Padding { left: 6.0, right: 6.0, top: 0.0, bottom: 0.0 })
    .height(18)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(color)),
        border: Border { radius: 3.0.into(), ..Default::default() },
        ..Default::default()
    });

    let bar_track = container(
        row![
            bar_fill.width(Length::FillPortion(fill_pct)),
            iced::widget::Space::new(Length::FillPortion(rest_pct), 18),
        ]
    )
    .height(18)
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg2)),
        border: Border { radius: 3.0.into(), ..Default::default() },
        ..Default::default()
    });

    let docs_str = data.docs.map(|d| format!("{} docs", format_num(d))).unwrap_or_default();
    let note_str = data.note.unwrap_or_default();
    let fs_small = (fs - 1.0).max(9.0);

    row![
        text(data.name).size(fs_small).color(label_color).font(iced::Font::MONOSPACE).width(160),
        bar_track,
        text(docs_str).size(fs_small).color(palette.fg_dim).font(iced::Font::MONOSPACE).width(90),
        text(note_str).size(fs_small).color(palette.fg_dim2).font(iced::Font::MONOSPACE),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

fn format_num(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(c);
    }
    out.chars().rev().collect()
}
