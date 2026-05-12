// TODO: remove when real backend is wired up — currently all mock data
#![allow(dead_code)]
use crate::theme::Palette;
use iced::{widget::button, Border, Color, Element, Padding};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhostVariant {
    Default,
    Active,
    Solid,
    Danger,
}

pub fn ghost_button<Msg: Clone + 'static>(
    label: &str,
    variant: GhostVariant,
    msg: Msg,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let (bg, fg, border_color) = match variant {
        GhostVariant::Default => (palette.bg, palette.fg, palette.border),
        GhostVariant::Active => (palette.bg_sel, palette.fg, palette.accent),
        GhostVariant::Solid => (palette.accent, palette.accent_fg, palette.accent),
        GhostVariant::Danger => (
            palette.bg,
            palette.danger,
            Color {
                r: palette.danger.r,
                g: palette.danger.g,
                b: palette.danger.b,
                a: 0.4,
            },
        ),
    };

    button(
        iced::widget::text(label.to_string())
            .size(fs)
            .color(fg)
            .font(iced::Font::MONOSPACE),
    )
    .padding(Padding {
        top: 3.0,
        bottom: 3.0,
        left: 10.0,
        right: 10.0,
    })
    .on_press(msg)
    .style(move |_, _status| button::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 5.0.into(),
        },
        text_color: fg,
        ..Default::default()
    })
    .into()
}
