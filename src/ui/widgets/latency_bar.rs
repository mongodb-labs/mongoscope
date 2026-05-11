use iced::{widget::{container, row, text}, Color, Element, Length};
use crate::theme::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyClass {
    Ok,
    Warn,
    Slow,
}

pub fn latency_class(ms: u32) -> LatencyClass {
    if ms >= 1000 { LatencyClass::Slow }
    else if ms >= 100 { LatencyClass::Warn }
    else { LatencyClass::Ok }
}

pub fn format_latency(ms: u32) -> String {
    if ms >= 1000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}

pub fn latency_color(ms: u32, p: &Palette) -> Color {
    match latency_class(ms) {
        LatencyClass::Ok => p.ok,
        LatencyClass::Warn => p.warn,
        LatencyClass::Slow => p.danger,
    }
}

/// Full latency cell: bar + value label.
pub fn latency_bar<Msg: 'static>(ms: u32, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let color = latency_color(ms, palette);
    let fill_pct = (((ms as f64 + 1.0).log10() * 28.0) as f32).min(100.0);
    let bg2 = palette.bg2;

    let track = container(
        container(iced::widget::Space::new(Length::Fill, 4))
            .width(Length::FillPortion((fill_pct as u16).max(1)))
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(color)),
                border: iced::Border { radius: 2.0.into(), ..Default::default() },
                ..Default::default()
            })
    )
    .width(Length::Fill)
    .height(4)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg2)),
        border: iced::Border { radius: 2.0.into(), ..Default::default() },
        ..Default::default()
    });

    let label = text(format_latency(ms))
        .size(fs)
        .color(color)
        .font(iced::Font::MONOSPACE)
        .width(44);

    row![track, label]
        .spacing(6)
        .align_y(iced::Alignment::Center)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_latency_ms() {
        assert_eq!(format_latency(4), "4ms");
        assert_eq!(format_latency(999), "999ms");
    }

    #[test]
    fn format_latency_seconds() {
        assert_eq!(format_latency(1000), "1.00s");
        assert_eq!(format_latency(4821), "4.82s");
    }

    #[test]
    fn latency_class_boundaries() {
        assert_eq!(latency_class(0), LatencyClass::Ok);
        assert_eq!(latency_class(99), LatencyClass::Ok);
        assert_eq!(latency_class(100), LatencyClass::Warn);
        assert_eq!(latency_class(999), LatencyClass::Warn);
        assert_eq!(latency_class(1000), LatencyClass::Slow);
    }
}
