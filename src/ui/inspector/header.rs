use crate::{data::model::QueryEntry, theme::Palette, ui::widgets::op_badge::op_badge};
use iced::{
    widget::{button, container, row, text},
    Border, Element, Length, Padding,
};

pub fn inspector_header<Msg: Clone + 'static>(
    entry: &QueryEntry,
    maximized: bool,
    on_close: Msg,
    on_maximize: Msg,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let bg2 = palette.bg2;
    let border_color = palette.border;
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;

    let coll = entry.coll.as_str().to_string();
    let app = entry.app.as_str().to_string();
    let id = entry.id.into_inner();

    let bg_hover = palette.bg_hover;
    let action_btn_msg = move |label: &'static str, msg: Msg| -> Element<'static, Msg> {
        button(
            text(label)
                .size(11)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE),
        )
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 6.0,
            right: 6.0,
        })
        .on_press(msg)
        .style(move |_, status| button::Style {
            background: match status {
                iced::widget::button::Status::Hovered => Some(iced::Background::Color(bg_hover)),
                _ => None,
            },
            border: Border::default(),
            ..Default::default()
        })
        .into()
    };

    let maximize_label = if maximized { "↙" } else { "↗" };

    container(
        row![
            op_badge(&entry.op, palette),
            iced::widget::Space::new(6, 0),
            text("shop.")
                .size(fs)
                .color(fg_dim)
                .font(iced::Font::MONOSPACE),
            text(coll).size(fs).color(fg).font(iced::Font::MONOSPACE),
            text(format!(" · {}", app))
                .size(fs)
                .color(fg_dim)
                .font(iced::Font::MONOSPACE),
            text(format!(" · #{}", id))
                .size(fs - 1.0)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE),
            iced::widget::Space::new(Length::Fill, 0),
            action_btn_msg(maximize_label, on_maximize),
            action_btn_msg("✕", on_close),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center)
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 12.0,
            right: 6.0,
        }),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg2)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
