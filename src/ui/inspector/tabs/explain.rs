use crate::{
    data::model::{Plan, QueryEntry},
    theme::Palette,
    ui::widgets::{
        flame_row::{flame_row, FlameRowData, FlameRowKind},
        mini_card::mini_card,
    },
};
use iced::{
    widget::{button, column, container, row, scrollable, text},
    Border, Color, Element, Length, Padding,
};

#[derive(Debug, Clone, Default)]
pub struct ExplainState {
    pub index_applied: bool,
}

#[derive(Debug, Clone)]
pub enum ExplainMsg {
    CopyIndex,
    RunIndex,
}

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

/// A proportional bar for before/after comparison.
fn plan_bar<Msg: 'static>(
    label: &str,
    ms: u32,
    max_ms: f32,
    bar_color: Color,
    fg_dim: Color,
    fs: f32,
) -> Element<'static, Msg> {
    let label = label.to_owned();
    let ms_label = if ms == 0 {
        "~0ms".to_owned()
    } else {
        format!("~{}ms", ms)
    };
    // bar width as a portion: max_ms maps to 90%
    let fill_pct = if max_ms > 0.0 {
        ((ms as f32 / max_ms) * 90.0).clamp(2.0, 90.0) as u16
    } else {
        2
    };
    let rest_pct = (100u16.saturating_sub(fill_pct)).max(1);

    let bar = row![
        container(iced::widget::Space::new(Length::Fill, 4))
            .width(Length::FillPortion(fill_pct))
            .height(4)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bar_color)),
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        iced::widget::Space::new(Length::FillPortion(rest_pct), 4),
    ]
    .width(Length::Fill);

    column![
        row![
            text(label)
                .size(fs - 1.0)
                .color(fg_dim)
                .font(iced::Font::MONOSPACE)
                .width(Length::Fill),
            text(ms_label)
                .size(fs - 1.0)
                .color(fg_dim)
                .font(iced::Font::MONOSPACE),
        ]
        .spacing(4),
        bar,
    ]
    .spacing(2)
    .width(Length::Fill)
    .into()
}

pub fn explain_tab<Msg: Clone + 'static>(
    entry: &QueryEntry,
    state: &ExplainState,
    on_msg: impl Fn(ExplainMsg) -> Msg + 'static + Copy,
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
    let fg = palette.fg;
    let danger = palette.danger;
    let warn = palette.warn;
    let ok = palette.ok;
    let tok_call = palette.tok_call;
    let tok_str = palette.tok_str;
    let tok_num = palette.tok_num;
    let index_applied = state.index_applied;

    let sugg_content: Element<'static, Msg> = if is_bad {
        // ── Before ms values (from explain_stages mock)
        let collscan_ms = (total_ms as f32 * 0.92) as u32;
        let sort_ms = (total_ms as f32 * 0.06) as u32;
        let limit_before_ms: u32 = 1;

        // ── After ms values
        let ixscan_ms: u32 = 1;
        let fetch_ms: u32 = 2;
        let sort_after_ms: u32 = 1;
        let limit_after_ms: u32 = 0;

        // bar color for before column: dimmed if index applied
        let before_bar_color = Color {
            a: if index_applied { 0.40 } else { 0.7 },
            ..danger
        };
        let before_label_color = Color {
            a: if index_applied { 0.4 } else { 1.0 },
            ..fg_dim
        };
        let before_max = collscan_ms as f32; // widest in before column

        // After column styling
        let after_border_color = if index_applied {
            Color { a: 0.6, ..ok }
        } else {
            border
        };
        let after_bg_color = if index_applied {
            Color { a: 0.08, ..ok }
        } else {
            Color::TRANSPARENT
        };
        let after_bar_color = Color { a: 0.8, ..ok };
        let after_max: f32 = fetch_ms as f32; // widest in after column

        // Before column header
        let before_header_color = Color {
            a: if index_applied { 0.4 } else { 1.0 },
            ..fg_dim2
        };
        let before_col: Element<'static, Msg> = container(
            column![
                text("before")
                    .size(9)
                    .color(before_header_color)
                    .font(iced::Font::MONOSPACE),
                iced::widget::Space::new(0, 4),
                plan_bar::<Msg>(
                    "COLLSCAN",
                    collscan_ms,
                    before_max,
                    before_bar_color,
                    before_label_color,
                    fs_small
                ),
                plan_bar::<Msg>(
                    "FETCH",
                    0,
                    before_max,
                    before_bar_color,
                    before_label_color,
                    fs_small
                ),
                plan_bar::<Msg>(
                    "SORT",
                    sort_ms,
                    before_max,
                    before_bar_color,
                    before_label_color,
                    fs_small
                ),
                plan_bar::<Msg>(
                    "LIMIT",
                    limit_before_ms,
                    before_max,
                    before_bar_color,
                    before_label_color,
                    fs_small
                ),
            ]
            .spacing(4)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .padding(Padding {
            top: 8.0,
            bottom: 8.0,
            left: 8.0,
            right: 8.0,
        })
        .style(move |_| container::Style {
            border: Border {
                color: border,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into();

        // After column header
        let after_header_color = if index_applied { ok } else { fg_dim2 };
        let est_badge_bg = Color { a: 0.15, ..warn };
        let est_badge_border = Color { a: 0.30, ..warn };
        let after_header: Element<'static, Msg> = if index_applied {
            let est_badge: Element<'static, Msg> =
                container(text("EST.").size(9).color(warn).font(iced::Font::MONOSPACE))
                    .padding(Padding {
                        top: 1.0,
                        bottom: 1.0,
                        left: 4.0,
                        right: 4.0,
                    })
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(est_badge_bg)),
                        border: Border {
                            color: est_badge_border,
                            width: 1.0,
                            radius: 3.0.into(),
                        },
                        ..Default::default()
                    })
                    .into();
            row![
                text("✓ index applied")
                    .size(9)
                    .color(after_header_color)
                    .font(iced::Font::MONOSPACE),
                est_badge,
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center)
            .into()
        } else {
            text("after (est.)")
                .size(9)
                .color(after_header_color)
                .font(iced::Font::MONOSPACE)
                .into()
        };
        let after_col: Element<'static, Msg> = container(
            column![
                after_header,
                iced::widget::Space::new(0, 4),
                plan_bar::<Msg>(
                    "IXSCAN",
                    ixscan_ms,
                    after_max,
                    after_bar_color,
                    fg_dim,
                    fs_small
                ),
                plan_bar::<Msg>(
                    "FETCH",
                    fetch_ms,
                    after_max,
                    after_bar_color,
                    fg_dim,
                    fs_small
                ),
                plan_bar::<Msg>(
                    "SORT",
                    sort_after_ms,
                    after_max,
                    after_bar_color,
                    fg_dim,
                    fs_small
                ),
                plan_bar::<Msg>(
                    "LIMIT",
                    limit_after_ms,
                    after_max,
                    after_bar_color,
                    fg_dim,
                    fs_small
                ),
            ]
            .spacing(4)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .padding(Padding {
            top: 8.0,
            bottom: 8.0,
            left: 8.0,
            right: 8.0,
        })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(after_bg_color)),
            border: Border {
                color: after_border_color,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into();

        // ── Code pill row
        // "index" label pill
        let index_label_bg = Color { a: 0.12, ..accent };
        let index_pill: Element<'static, Msg> = container(
            text("index")
                .size(fs_small)
                .color(accent)
                .font(iced::Font::MONOSPACE),
        )
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 8.0,
            right: 8.0,
        })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(index_label_bg)),
            border: Border {
                color: accent,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into();

        // Syntax-highlighted command
        let coll2 = coll.clone();
        let first_key2 = first_key.clone();
        let cmd_display: Element<'static, Msg> = container(
            row![
                text("db.")
                    .size(fs_small)
                    .color(fg_dim)
                    .font(iced::Font::MONOSPACE),
                text(coll2)
                    .size(fs_small)
                    .color(fg)
                    .font(iced::Font::MONOSPACE),
                text(".createIndex")
                    .size(fs_small)
                    .color(tok_call)
                    .font(iced::Font::MONOSPACE),
                text("({ ")
                    .size(fs_small)
                    .color(fg)
                    .font(iced::Font::MONOSPACE),
                text(format!("\"{}\"", first_key2))
                    .size(fs_small)
                    .color(tok_str)
                    .font(iced::Font::MONOSPACE),
                text(": ")
                    .size(fs_small)
                    .color(fg)
                    .font(iced::Font::MONOSPACE),
                text("1")
                    .size(fs_small)
                    .color(tok_num)
                    .font(iced::Font::MONOSPACE),
                text(" })")
                    .size(fs_small)
                    .color(fg)
                    .font(iced::Font::MONOSPACE),
            ]
            .spacing(0)
            .align_y(iced::Alignment::Center),
        )
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 8.0,
            right: 8.0,
        })
        .style(move |_| container::Style {
            border: Border {
                color: border2,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into();

        // Copy button
        let bg1 = palette.bg1;
        let copy_btn: Element<'static, Msg> = button(
            text("Copy")
                .size(fs_small)
                .color(fg_dim)
                .font(iced::Font::MONOSPACE),
        )
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 8.0,
            right: 8.0,
        })
        .on_press(on_msg(ExplainMsg::CopyIndex))
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(bg1)),
            border: Border {
                color: border2,
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: fg_dim,
            ..Default::default()
        })
        .into();

        // Run button
        let run_label = if index_applied {
            "✓ Created"
        } else {
            "▶ Run"
        };
        let run_bg = if index_applied {
            Color { a: 0.15, ..accent }
        } else {
            bg1
        };
        let run_fg = if index_applied { accent } else { fg_dim };
        let run_btn: Element<'static, Msg> = button(
            text(run_label)
                .size(fs_small)
                .color(run_fg)
                .font(iced::Font::MONOSPACE),
        )
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 8.0,
            right: 8.0,
        })
        .on_press(on_msg(ExplainMsg::RunIndex))
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(run_bg)),
            border: Border {
                color: if index_applied { accent } else { border2 },
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: run_fg,
            ..Default::default()
        })
        .into();

        let code_pill_row: Element<'static, Msg> =
            row![index_pill, cmd_display, copy_btn, run_btn,]
                .spacing(6)
                .align_y(iced::Alignment::Center)
                .into();

        let italic_font = iced::Font {
            style: iced::font::Style::Italic,
            ..iced::Font::MONOSPACE
        };

        let mut sugg_col: Vec<Element<'static, Msg>> = vec![
            text("SUGGESTIONS")
                .size(9)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE)
                .into(),
            iced::widget::Space::new(0, 6).into(),
            // Before/after split
            row![before_col, after_col].spacing(8).into(),
        ];

        if index_applied {
            sugg_col.push(
                text("actual speedup depends on data distribution")
                    .size(9)
                    .color(fg_dim2)
                    .font(italic_font)
                    .into(),
            );
        }

        sugg_col.push(
            // Separator
            container(iced::widget::Space::new(Length::Fill, 0))
                .height(1)
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(border2)),
                    ..Default::default()
                })
                .into(),
        );
        sugg_col.push(code_pill_row);
        sugg_col.push(
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
            .align_y(iced::Alignment::Center)
            .into(),
        );

        column(sugg_col).spacing(6).into()
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
                    .color(ok)
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

    // suppress unused variable warnings for non-bad path
    let _ = warn;

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
