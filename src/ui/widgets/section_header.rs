// TODO: remove when real backend is wired up — currently all mock data
#![allow(dead_code)]
use iced::{widget::text, Color, Element};

/// 10px uppercase letter-spaced label used in sidebar sections and card titles.
pub fn section_header<Msg: 'static>(label: &str, color: Color) -> Element<'static, Msg> {
    text(label.to_uppercase()).size(10).color(color).into()
}
