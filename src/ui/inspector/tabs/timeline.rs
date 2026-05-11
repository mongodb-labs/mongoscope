use iced::{widget::{column, container, row, scrollable, text}, Border, Color, Element, Length, Padding};
use crate::{
    data::model::QueryEntry,
    theme::Palette,
    ui::widgets::{
        gantt::{gantt_row, GanttPhase},
        mini_card::mini_card,
    },
};

fn separator<Msg: 'static>(border_c: Color) -> Element<'static, Msg> {
    container(iced::widget::Space::new(Length::Fill, 0))
        .height(1)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(border_c)),
            ..Default::default()
        })
        .into()
}

struct Neighbour {
    t_str: String,
    latency: u32,
    label: String,
    is_self: bool,
}

fn mock_neighbours(entry: &QueryEntry) -> Vec<Neighbour> {
    let t = entry.t_ms.into_inner() as u32;
    let lat = entry.latency_ms.into_inner();
    let coll = entry.coll.as_str().to_string();
    let op = entry.op.label();
    let step = (lat / 4).max(20);

    vec![
        Neighbour { t_str: format!("+{}", t.saturating_sub(step * 3)), latency: (lat / 3).max(1), label: "find · users".into(), is_self: false },
        Neighbour { t_str: format!("+{}", t.saturating_sub(step * 2)), latency: (lat / 2).max(1), label: "agg · events".into(), is_self: false },
        Neighbour { t_str: format!("+{}", t.saturating_sub(step)), latency: lat * 2, label: format!("find · {}", coll), is_self: false },
        Neighbour { t_str: format!("+{}", t), latency: lat, label: format!("{} · {}", op, coll), is_self: true },
        Neighbour { t_str: format!("+{}", t + step), latency: (lat / 4).max(1), label: "ins · orders".into(), is_self: false },
        Neighbour { t_str: format!("+{}", t + step * 2), latency: lat * 3 / 4, label: "find · products".into(), is_self: false },
        Neighbour { t_str: format!("+{}", t + step * 3), latency: (lat / 5).max(1), label: "upd · users".into(), is_self: false },
    ]
}

pub fn timeline_tab<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let total_ms = entry.latency_ms.into_inner();
    let phases = mock_phases(total_ms);
    let fs_small = (fs - 1.0).max(9.0);

    let border_c = Color { r: palette.border.r, g: palette.border.g, b: palette.border.b, a: 1.0 };
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let fg = palette.fg;
    let bg_sel = palette.bg_sel;
    let border = palette.border;

    // ── timeline header (t+0 ... t+Xms)
    let timelinehd = row![
        text("t+0").size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
        iced::widget::Space::new(Length::Fill, 0),
        text(format!("t+{}ms", total_ms)).size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
    ]
    .padding(Padding { top: 0.0, bottom: 4.0, left: 0.0, right: 0.0 });

    // ── gantt rows
    let mut phase_rows: Vec<Element<Msg>> = Vec::new();
    for phase in phases {
        phase_rows.push(gantt_row(phase, palette, fs));
    }
    let gantt_col = column(phase_rows).spacing(2);

    // ── neighbours card
    let conn_id = 1420 + (entry.id.into_inner() % 40);
    let neighbours = mock_neighbours(entry);

    let mut nbr_rows: Vec<Element<'static, Msg>> = Vec::new();
    for nbr in neighbours {
        let lat = nbr.latency;
        let lat_color = if lat >= 1000 { palette.danger }
            else if lat >= 100 { palette.warn }
            else { palette.ok };
        let bar_pct = ((lat as f32 + 1.0).log10() * 60.0).max(8.0).min(280.0) / 280.0;
        let bar_fp = (bar_pct * 100.0) as u16;
        let bar_rest = 100u16.saturating_sub(bar_fp).max(1);
        let is_self = nbr.is_self;
        let row_bg = if is_self { bg_sel } else { Color::TRANSPARENT };
        let t_str = nbr.t_str;
        let label = nbr.label;
        let lat_str = if lat >= 1000 { format!("{:.1}s", lat as f32 / 1000.0) } else { format!("{}ms", lat) };

        let nbr_row = container(
            row![
                text(t_str).size(10).color(fg_dim2).font(iced::Font::MONOSPACE).width(50),
                container(
                    row![
                        container(iced::widget::Space::new(Length::Fill, 8))
                            .width(Length::FillPortion(bar_fp.max(1)))
                            .height(8)
                            .style(move |_| container::Style {
                                background: Some(iced::Background::Color(lat_color)),
                                border: Border { radius: 2.0.into(), ..Default::default() },
                                ..Default::default()
                            }),
                        iced::widget::Space::new(Length::FillPortion(bar_rest), 8),
                    ]
                )
                .width(Length::Fill)
                .height(8)
                .style(move |_| container::Style { ..Default::default() }),
                text(label).size(10).color(fg).font(iced::Font::MONOSPACE).width(Length::Fill),
                text(lat_str).size(10).color(fg_dim).font(iced::Font::MONOSPACE),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center)
            .padding(if is_self {
                Padding { top: 3.0, bottom: 3.0, left: 6.0, right: 6.0 }
            } else {
                Padding { top: 3.0, bottom: 3.0, left: 0.0, right: 0.0 }
            })
        )
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(row_bg)),
            border: if is_self { Border { radius: 4.0.into(), ..Default::default() } } else { Border::default() },
            ..Default::default()
        })
        .into();

        nbr_rows.push(nbr_row);
    }

    let nbr_content: Element<'static, Msg> = column![
        text(format!("NEIGHBOURS ON CONNECTION conn-{}", conn_id))
            .size(9).color(fg_dim2).font(iced::Font::MONOSPACE),
        iced::widget::Space::new(0, 6),
        column(nbr_rows).spacing(2),
    ]
    .spacing(0)
    .into();

    let nbr_card = mini_card(nbr_content, palette);

    scrollable(
        column![timelinehd, gantt_col, nbr_card]
            .spacing(12)
            .padding(Padding { top: 14.0, bottom: 14.0, left: 16.0, right: 16.0 })
    )
    .height(Length::Fill)
    .into()
}

fn mock_phases(total_ms: u32) -> Vec<GanttPhase> {
    use iced::Color;
    let t = total_ms as f32;
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
