use crate::{
    data::model::{Plan, QueryEntry},
    theme::Palette,
    ui::widgets::{
        flame_row::{flame_row, FlameRowData, FlameRowKind},
        mini_card::mini_card,
    },
};
use iced::{
    widget::{column, container, row, scrollable, text},
    Border, Color, Element, Length, Padding,
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

fn action_label<'a, Msg: 'a>(
    label: &'a str,
    active: bool,
    palette: &Palette,
    fs: f32,
) -> Element<'a, Msg> {
    let (bg, fg, border) = if active {
        (palette.bg_sel, palette.fg, palette.accent)
    } else {
        (palette.bg, palette.fg_dim, palette.border)
    };
    container(text(label).size(fs).color(fg).font(iced::Font::MONOSPACE))
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 10.0,
            right: 10.0,
        })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border {
                color: border,
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        })
        .into()
}

pub fn explain_tab<Msg: 'static>(
    entry: &QueryEntry,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let total_ms = entry.latency_ms.into_inner();
    let stages = explain_stages(entry, total_ms);
    let fs_small = (fs - 1.0).max(9.0);

    let plan_label = entry
        .plan
        .as_ref()
        .map(|p| p.label())
        .unwrap_or_else(|| "—".to_string());
    let is_bad = matches!(entry.plan, Some(Plan::CollScan));
    let plan_color = if is_bad { palette.danger } else { palette.fg };
    let border_c = Color {
        r: palette.border.r,
        g: palette.border.g,
        b: palette.border.b,
        a: 1.0,
    };

    // ── explainhd
    let explainhd = column![
        row![
            row![
                text("winning plan · ")
                    .size(fs_small)
                    .color(palette.fg_dim)
                    .font(iced::Font::MONOSPACE),
                text(plan_label)
                    .size(fs_small)
                    .color(plan_color)
                    .font(iced::Font::MONOSPACE),
            ],
            iced::widget::Space::new(Length::Fill, 0),
            row![
                action_label("Tree", false, palette, fs_small),
                action_label("Flame", true, palette, fs_small),
                action_label("Raw", false, palette, fs_small),
                action_label("Rejected plans (3)", false, palette, fs_small),
            ]
            .spacing(4),
        ]
        .align_y(iced::Alignment::Center)
        .padding(Padding {
            top: 0.0,
            bottom: 8.0,
            left: 0.0,
            right: 0.0
        }),
        separator(border_c),
    ]
    .width(Length::Fill);

    // ── flame stages
    let mut stage_rows: Vec<Element<Msg>> = Vec::new();
    for data in stages {
        stage_rows.push(flame_row(data, palette, fs));
    }
    let stages_col = column(stage_rows).spacing(3);

    // ── suggestions card
    let first_key = entry
        .filter
        .as_ref()
        .and_then(|f| f.keys().next().cloned())
        .unwrap_or_else(|| "field".to_string());
    let coll = entry.coll.as_str().to_string();
    let docs_ex = entry
        .docs_examined
        .as_ref()
        .map(|d| d.into_inner())
        .unwrap_or(0);
    let docs_ret = entry
        .docs_returned
        .as_ref()
        .map(|d| d.into_inner())
        .unwrap_or(0);
    let index_label = entry
        .index
        .as_ref()
        .map(|i| i.as_str().to_string())
        .unwrap_or_else(|| "—".to_string());

    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let accent = palette.accent;
    let border = palette.border;
    let border2 = Color {
        r: border.r,
        g: border.g,
        b: border.b,
        a: 0.7,
    };

    let sugg_content: Element<'static, Msg> = if is_bad {
        column![
            text("SUGGESTIONS")
                .size(9)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE),
            iced::widget::Space::new(0, 6),
            // suggestion 1
            row![
                text("create index")
                    .size(fs_small)
                    .color(accent)
                    .font(iced::Font::MONOSPACE)
                    .width(110),
                text(format!("db.{}.createIndex({{ {}: 1 }})", coll, first_key))
                    .size(fs_small)
                    .color(fg_dim)
                    .font(iced::Font::MONOSPACE)
                    .width(Length::Fill),
                text("est. ~4ms · 99.9% faster")
                    .size(10)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
            container(iced::widget::Space::new(Length::Fill, 0))
                .height(1)
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(border2)),
                    ..Default::default()
                }),
            // suggestion 2
            row![
                text("add covered projection")
                    .size(fs_small)
                    .color(accent)
                    .font(iced::Font::MONOSPACE)
                    .width(110),
                text(".project({ _id: 0, … })")
                    .size(fs_small)
                    .color(fg_dim)
                    .font(iced::Font::MONOSPACE)
                    .width(Length::Fill),
                text("avoid FETCH stage")
                    .size(10)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(6)
        .into()
    } else {
        column![
            text("SUGGESTIONS")
                .size(9)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE),
            iced::widget::Space::new(0, 6),
            row![
                text("looks healthy")
                    .size(fs_small)
                    .color(palette.ok)
                    .font(iced::Font::MONOSPACE)
                    .width(110),
                text(format!(
                    "Plan uses an index scan on {} with {} examined for {} returned.",
                    index_label, docs_ex, docs_ret
                ))
                .size(fs_small)
                .color(fg_dim)
                .font(iced::Font::MONOSPACE),
            ]
            .spacing(10),
        ]
        .spacing(6)
        .into()
    };

    let sugg_card = mini_card(sugg_content, palette);

    scrollable(
        column![explainhd, stages_col, sugg_card]
            .spacing(12)
            .padding(Padding {
                top: 14.0,
                bottom: 14.0,
                left: 16.0,
                right: 16.0,
            }),
    )
    .height(Length::Fill)
    .into()
}

fn explain_stages(entry: &QueryEntry, total_ms: u32) -> Vec<FlameRowData> {
    let slow_kind = if entry.slow {
        FlameRowKind::Bad
    } else if entry.warn.is_some() {
        FlameRowKind::Warn
    } else {
        FlameRowKind::Ok
    };

    match &entry.plan {
        Some(Plan::CollScan) => vec![
            FlameRowData {
                name: "COLLSCAN".into(),
                ms: (total_ms as f32 * 0.92) as u32,
                total_ms,
                docs: entry.docs_examined.as_ref().map(|d| d.into_inner()),
                note: Some("no index usable".into()),
                kind: FlameRowKind::Bad,
            },
            FlameRowData {
                name: "SORT (in memory)".into(),
                ms: (total_ms as f32 * 0.06) as u32,
                total_ms,
                docs: entry.docs_examined.as_ref().map(|d| d.into_inner()),
                note: Some("spill risk".into()),
                kind: FlameRowKind::Warn,
            },
            FlameRowData {
                name: "LIMIT".into(),
                ms: 1,
                total_ms,
                docs: entry.docs_returned.as_ref().map(|d| d.into_inner()),
                note: None,
                kind: FlameRowKind::Ok,
            },
        ],
        Some(Plan::IxScan(idx)) => vec![
            FlameRowData {
                name: format!("IXSCAN · {}", idx.as_str()),
                ms: (total_ms as f32 * 0.35) as u32,
                total_ms,
                docs: None,
                note: None,
                kind: FlameRowKind::Ok,
            },
            FlameRowData {
                name: "FETCH".into(),
                ms: (total_ms as f32 * 0.45) as u32,
                total_ms,
                docs: entry.docs_examined.as_ref().map(|d| d.into_inner()),
                note: None,
                kind: slow_kind,
            },
            FlameRowData {
                name: "PROJECTION".into(),
                ms: (total_ms as f32 * 0.15) as u32,
                total_ms,
                docs: entry.docs_returned.as_ref().map(|d| d.into_inner()),
                note: None,
                kind: FlameRowKind::Ok,
            },
        ],
        Some(Plan::IxScanLookup(idx)) => vec![
            FlameRowData {
                name: "IXSCAN".into(),
                ms: (total_ms as f32 * 0.10) as u32,
                total_ms,
                docs: None,
                note: Some(idx.as_str().to_string()),
                kind: FlameRowKind::Ok,
            },
            FlameRowData {
                name: "$LOOKUP".into(),
                ms: (total_ms as f32 * 0.75) as u32,
                total_ms,
                docs: entry.docs_examined.as_ref().map(|d| d.into_inner()),
                note: entry.warn.clone(),
                kind: FlameRowKind::Warn,
            },
            FlameRowData {
                name: "PROJECTION".into(),
                ms: (total_ms as f32 * 0.15) as u32,
                total_ms,
                docs: None,
                note: None,
                kind: FlameRowKind::Ok,
            },
        ],
        Some(Plan::IdHack) => vec![FlameRowData {
            name: "IDHACK".into(),
            ms: total_ms,
            total_ms,
            docs: Some(1),
            note: Some("single-doc by _id".into()),
            kind: FlameRowKind::Ok,
        }],
        _ => vec![FlameRowData {
            name: "UNKNOWN".into(),
            ms: total_ms,
            total_ms,
            docs: None,
            note: None,
            kind: FlameRowKind::Warn,
        }],
    }
}
