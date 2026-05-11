use iced::{widget::{column, scrollable, text}, Element, Length, Padding};
use crate::{
    data::model::{Plan, QueryEntry},
    theme::Palette,
    ui::widgets::flame_row::{flame_row, FlameRowData, FlameRowKind},
};

pub fn explain_tab<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let total_ms = entry.latency_ms.into_inner();
    let stages = explain_stages(entry, total_ms);

    let mut children: Vec<Element<'static, Msg>> = vec![
        text("Execution stages").size(fs - 1.0).color(palette.fg_dim2)
            .font(iced::Font::MONOSPACE).into(),
    ];

    for data in stages {
        children.push(flame_row(data, palette, fs));
    }

    scrollable(
        column(children).spacing(4)
            .padding(Padding { top: 8.0, bottom: 8.0, left: 12.0, right: 12.0 })
    )
    .height(Length::Fill)
    .into()
}

fn explain_stages(entry: &QueryEntry, total_ms: u32) -> Vec<FlameRowData> {
    let slow_kind = if entry.slow { FlameRowKind::Bad }
        else if entry.warn.is_some() { FlameRowKind::Warn }
        else { FlameRowKind::Ok };

    match &entry.plan {
        Some(Plan::CollScan) => vec![
            FlameRowData { name: "COLLSCAN".into(), ms: (total_ms as f32 * 0.85) as u32, total_ms, docs: entry.docs_examined.as_ref().map(|d| d.into_inner()), note: Some("missing index".into()), kind: FlameRowKind::Bad },
            FlameRowData { name: "PROJECTION".into(), ms: (total_ms as f32 * 0.10) as u32, total_ms, docs: entry.docs_returned.as_ref().map(|d| d.into_inner()), note: None, kind: FlameRowKind::Ok },
        ],
        Some(Plan::IxScan(idx)) => vec![
            FlameRowData { name: "IXSCAN".into(), ms: (total_ms as f32 * 0.15) as u32, total_ms, docs: None, note: Some(idx.as_str().to_string()), kind: FlameRowKind::Ok },
            FlameRowData { name: "FETCH".into(), ms: (total_ms as f32 * 0.70) as u32, total_ms, docs: entry.docs_examined.as_ref().map(|d| d.into_inner()), note: None, kind: slow_kind },
            FlameRowData { name: "PROJECTION".into(), ms: (total_ms as f32 * 0.15) as u32, total_ms, docs: entry.docs_returned.as_ref().map(|d| d.into_inner()), note: None, kind: FlameRowKind::Ok },
        ],
        Some(Plan::IxScanLookup(idx)) => vec![
            FlameRowData { name: "IXSCAN".into(), ms: (total_ms as f32 * 0.10) as u32, total_ms, docs: None, note: Some(idx.as_str().to_string()), kind: FlameRowKind::Ok },
            FlameRowData { name: "$LOOKUP".into(), ms: (total_ms as f32 * 0.75) as u32, total_ms, docs: entry.docs_examined.as_ref().map(|d| d.into_inner()), note: entry.warn.clone(), kind: FlameRowKind::Warn },
            FlameRowData { name: "PROJECTION".into(), ms: (total_ms as f32 * 0.15) as u32, total_ms, docs: None, note: None, kind: FlameRowKind::Ok },
        ],
        Some(Plan::IdHack) => vec![
            FlameRowData { name: "IDHACK".into(), ms: total_ms, total_ms, docs: Some(1), note: None, kind: FlameRowKind::Ok },
        ],
        _ => vec![
            FlameRowData { name: "UNKNOWN".into(), ms: total_ms, total_ms, docs: None, note: None, kind: FlameRowKind::Warn },
        ],
    }
}
