use crate::theme::Palette;
use iced::{
    widget::{container, row, text},
    Border, Element, Length, Padding,
};

pub fn table_header<Msg: 'static>(palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let bg2 = palette.bg2;
    let border_color = palette.border;
    let fg_dim2 = palette.fg_dim2;

    let col = |label: &'static str, w: u16| -> Element<'static, Msg> {
        text(label)
            .size(fs - 1.0)
            .color(fg_dim2)
            .font(iced::Font::MONOSPACE)
            .width(w)
            .into()
    };

    container(
        row![
            col("TIME", 60),
            col("OP", 80),
            col("COLL", 120),
            col("APP", 120),
            col("PLAN", 100),
            col("DOCS", 100),
            col("WARN", 160),
            container(
                text("LATENCY")
                    .size(fs - 1.0)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE)
            )
            .width(Length::Fill),
        ]
        .spacing(4)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 10.0,
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
