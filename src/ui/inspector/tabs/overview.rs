use iced::{widget::{button, column, container, row, scrollable, text}, Border, Color, Element, Length, Padding};
use crate::{
    data::model::{QueryEntry, Op},
    theme::Palette,
    ui::widgets::{
        kv_grid::{kv_grid, KvRow},
        op_badge::op_badge,
        plan_chip::plan_chip,
    },
};

fn format_latency(ms: u32) -> String {
    if ms >= 1000 { format!("{:.2}s", ms as f64 / 1000.0) }
    else { format!("{}ms", ms) }
}

pub fn overview_tab<'a, Msg: Clone + 'static>(
    entry: &'a QueryEntry,
    palette: &Palette,
    fs: f32,
) -> Element<'a, Msg> {
    let latency = entry.latency_ms.into_inner();
    let coll = entry.coll.as_str().to_string();
    let lat_str = format_latency(latency);

    // latency color class
    let lat_color = if latency >= 1000 { palette.danger }
        else if latency >= 100 { palette.warn }
        else { palette.ok };

    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;

    // ── Hero section
    let hero = container(
        row![
            row![
                op_badge(&entry.op, palette),
                iced::widget::Space::new(8, 0),
                text(format!("shop.{}", coll)).size(fs + 1.0).color(fg).font(iced::Font::MONOSPACE),
            ].align_y(iced::Alignment::Center),
            iced::widget::Space::new(Length::Fill, 0),
            column![
                text(lat_str.clone()).size(22).color(lat_color).font(iced::Font::MONOSPACE),
                text("total wall clock").size(9).color(fg_dim2).font(iced::Font::MONOSPACE),
            ].align_x(iced::Alignment::End).spacing(2),
        ]
        .align_y(iced::Alignment::Center)
    )
    .width(Length::Fill)
    .padding(Padding { top: 10.0, bottom: 10.0, left: 12.0, right: 12.0 });

    // ── Warn banner
    let warn_el: Option<Element<Msg>> = entry.warn.as_ref().map(|w| {
        let warn_str = w.clone();
        let warn_color = palette.warn;
        let warn_bg = Color { r: warn_color.r, g: warn_color.g, b: warn_color.b, a: 0.10 };
        let warn_border = Color { r: warn_color.r, g: warn_color.g, b: warn_color.b, a: 0.35 };
        let accent = palette.accent;
        let fg_dim2 = palette.fg_dim2;

        container(
            row![
                text("◆").size(fs).color(warn_color),
                column![
                    text(warn_str).size(fs).color(warn_color).font(iced::Font::MONOSPACE),
                    text("Tap Explain → Flame to see where time was spent.")
                        .size(fs - 1.5).color(fg_dim2).font(iced::Font::MONOSPACE),
                ].spacing(2).width(Length::Fill),
                container(
                    text("Suggest index").size(10).color(accent).font(iced::Font::MONOSPACE)
                )
                .padding(Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 })
                .style(move |_| container::Style {
                    background: None,
                    border: Border { color: accent, width: 1.0, radius: 3.0.into() },
                    ..Default::default()
                }),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .padding(Padding { top: 8.0, bottom: 8.0, left: 12.0, right: 12.0 })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(warn_bg)),
            border: Border { color: warn_border, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
    });

    // ── kv-grid stats
    let namespace = format!("shop.{}", coll);
    let op_label = entry.op.label().to_string();
    let index_str = entry.index.as_ref()
        .map(|i| i.as_str().to_string())
        .unwrap_or_else(|| "⚠ none".to_string());
    let docs_ex = entry.docs_examined.as_ref()
        .map(|d| d.into_inner().to_string())
        .unwrap_or("—".to_string());
    let docs_ret = entry.docs_returned.as_ref()
        .map(|d| d.into_inner().to_string())
        .unwrap_or("—".to_string());
    let ratio_str = match (entry.docs_examined.as_ref(), entry.docs_returned.as_ref()) {
        (Some(ex), Some(ret)) if ret.into_inner() > 0 => {
            format!("{:.1}×", ex.into_inner() as f64 / ret.into_inner() as f64)
        }
        _ => "—".to_string(),
    };
    let conn_id = format!("conn-{}", 1420 + (entry.id.into_inner() % 40));
    let t_ms = entry.t_ms.into_inner();
    let started = format!("t+{}ms", t_ms);

    let stats = vec![
        KvRow::new("operation", op_label),
        KvRow::new("namespace", namespace),
        KvRow::new("plan", entry.plan.as_ref().map(|p| p.label()).unwrap_or_else(|| "—".to_string())),
        KvRow::new("index", index_str),
        KvRow::new("docs examined", docs_ex),
        KvRow::new("docs returned", docs_ret),
        KvRow::new("examined / returned", ratio_str),
        KvRow::new("latency", lat_str),
        KvRow::new("client", entry.app.as_str().to_string()),
        KvRow::new("connection id", conn_id),
        KvRow::new("started", started),
    ];

    let kv = container(kv_grid(stats, palette, fs))
        .padding(Padding { top: 8.0, bottom: 8.0, left: 12.0, right: 12.0 })
        .width(Length::Fill);

    // ── Plan badge row
    let plan_row: Option<Element<Msg>> = entry.plan.as_ref().map(|p| {
        container(plan_chip(p, palette))
            .padding(Padding { top: 0.0, bottom: 8.0, left: 12.0, right: 12.0 })
            .into()
    });

    // ── Efficiency mini-card
    let ratio_val = match (entry.docs_examined.as_ref(), entry.docs_returned.as_ref()) {
        (Some(ex), Some(ret)) if ret.into_inner() > 0 => {
            ex.into_inner() as f64 / ret.into_inner() as f64
        }
        _ => 0.0,
    };
    let efficiency = if ratio_val < 2.0 { "optimal" } else if ratio_val < 50.0 { "fair" } else { "poor" };
    let eff_color = if ratio_val < 2.0 { palette.ok } else if ratio_val < 50.0 { palette.warn } else { palette.danger };
    let eff_pct = if ratio_val <= 0.0 { 100.0f32 }
        else { (100.0 / (ratio_val as f32).log10().max(0.01) / 3.0).max(4.0).min(100.0) };

    let bg1 = palette.bg1;
    let bg2 = palette.bg2;
    let border_color = palette.border;

    let eff_bar = container(
        column![
            text("efficiency").size(9).color(fg_dim2).font(iced::Font::MONOSPACE),
            iced::widget::Space::new(0, 4),
            container(
                container(iced::widget::Space::new(Length::Fill, 6))
                    .width(Length::FillPortion((eff_pct as u16).max(1)))
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(eff_color)),
                        border: Border { radius: 2.0.into(), ..Default::default() },
                        ..Default::default()
                    })
            )
            .width(Length::Fill)
            .height(6)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg2)),
                border: Border { radius: 2.0.into(), ..Default::default() },
                ..Default::default()
            }),
            iced::widget::Space::new(0, 4),
            row![
                text("optimal (1×)").size(9).color(fg_dim2).font(iced::Font::MONOSPACE),
                iced::widget::Space::new(Length::Fill, 0),
                text("fair (50×)").size(9).color(fg_dim2).font(iced::Font::MONOSPACE),
                iced::widget::Space::new(Length::Fill, 0),
                text("poor (1000×+)").size(9).color(fg_dim2).font(iced::Font::MONOSPACE),
            ],
            text(format!("→ {}", efficiency)).size(10).color(eff_color).font(iced::Font::MONOSPACE),
        ]
        .spacing(2)
        .padding(Padding { top: 8.0, bottom: 8.0, left: 10.0, right: 10.0 })
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg1)),
        border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
        ..Default::default()
    });

    let mut children: Vec<Element<Msg>> = vec![hero.into()];
    if let Some(w) = warn_el { children.push(w); }
    children.push(kv.into());
    if let Some(p) = plan_row { children.push(p); }
    children.push(
        container(eff_bar)
            .padding(Padding { top: 0.0, bottom: 8.0, left: 12.0, right: 12.0 })
            .width(Length::Fill)
            .into()
    );

    scrollable(column(children).spacing(0))
        .height(Length::Fill)
        .into()
}
