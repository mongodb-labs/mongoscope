use iced::{widget::{container, row, text}, Element, Length, Padding};
use crate::{
    data::model::QueryEntry,
    theme::Palette,
    ui::widgets::{
        appdot::appdot,
        latency_bar::latency_bar,
        op_badge::op_badge,
        plan_chip::plan_chip,
    },
};

pub fn time_cell<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let t = entry.t_ms.into_inner();
    let secs = (t / 1000) % 60;
    let millis = t % 1000;
    text(format!("{:02}.{:03}", secs, millis))
        .size(fs)
        .color(palette.fg_dim2)
        .font(iced::Font::MONOSPACE)
        .width(60)
        .into()
}

pub fn op_cell<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    container(op_badge(&entry.op, palette))
        .width(80)
        .into()
}

pub fn coll_cell<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    text(entry.coll.as_str().to_string())
        .size(fs)
        .color(palette.fg)
        .font(iced::Font::MONOSPACE)
        .width(120)
        .into()
}

pub fn app_cell<Msg: 'static>(entry: &QueryEntry, app_color: [u8; 3], palette: &Palette, fs: f32) -> Element<'static, Msg> {
    row![
        appdot::<Msg>(app_color),
        text(entry.app.as_str().to_string())
            .size(fs)
            .color(palette.fg_dim)
            .font(iced::Font::MONOSPACE),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .width(120)
    .into()
}

pub fn plan_cell<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    container(match &entry.plan {
        Some(plan) => plan_chip(plan, palette),
        None => text("—").size(fs).color(palette.fg_dim2).font(iced::Font::MONOSPACE).into(),
    })
    .width(100)
    .into()
}

pub fn docs_cell<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let label = match (entry.docs_examined.as_ref(), entry.docs_returned.as_ref()) {
        (Some(ex), Some(ret)) => format!("{} → {}", ex.into_inner(), ret.into_inner()),
        _ => "—".into(),
    };
    text(label)
        .size(fs)
        .color(palette.fg_dim)
        .font(iced::Font::MONOSPACE)
        .width(100)
        .into()
}

pub fn latency_cell<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    container(latency_bar(entry.latency_ms.into_inner(), palette, fs))
        .width(Length::Fill)
        .into()
}

pub fn warn_cell<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let label = entry.warn.as_deref().unwrap_or("").to_string();
    text(label)
        .size(fs - 1.0)
        .color(palette.warn)
        .font(iced::Font::MONOSPACE)
        .width(160)
        .into()
}
