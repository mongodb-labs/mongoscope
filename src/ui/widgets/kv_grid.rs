use crate::theme::Palette;
use iced::{
    widget::{column, container, row, text},
    Color, Element, Length, Padding,
};

pub struct KvRow {
    pub key: String,
    pub value: String,
}

impl KvRow {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

pub fn kv_grid<Msg: 'static>(
    rows: Vec<KvRow>,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let mid = rows.len().div_ceil(2);
    let mut col_a: Vec<Element<Msg>> = Vec::new();
    let mut col_b: Vec<Element<Msg>> = Vec::new();

    for (i, r) in rows.into_iter().enumerate() {
        let cell = kv_row(r, palette, fs);
        if i < mid {
            col_a.push(cell);
        } else {
            col_b.push(cell);
        }
    }

    row![
        column(col_a).spacing(0).width(Length::Fill),
        column(col_b).spacing(0).width(Length::Fill),
    ]
    .spacing(24)
    .into()
}

fn kv_row<Msg: 'static>(r: KvRow, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let key_c = palette.fg_dim;
    let val_c = palette.fg;
    let border_c = Color {
        r: palette.border.r,
        g: palette.border.g,
        b: palette.border.b,
        a: 0.7,
    };
    let fs_small = (fs - 1.0).max(9.0);

    column![
        row![
            text(r.key)
                .size(fs_small)
                .color(key_c)
                .font(iced::Font::MONOSPACE)
                .width(Length::Fill),
            text(r.value)
                .size(fs_small)
                .color(val_c)
                .font(iced::Font::MONOSPACE),
        ]
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 0.0,
            right: 0.0
        }),
        container(iced::widget::Space::new(Length::Fill, 0))
            .height(1)
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(border_c)),
                ..Default::default()
            }),
    ]
    .into()
}
