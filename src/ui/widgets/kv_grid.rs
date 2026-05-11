use iced::{widget::{column, container, row, text}, Border, Color, Element, Length, Padding};
use crate::theme::Palette;

pub struct KvRow {
    pub key: String,
    pub value: String,
}

impl KvRow {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self { key: key.into(), value: value.into() }
    }
}

pub fn kv_grid<Msg: 'static>(
    rows: Vec<KvRow>,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let border_c = Color { r: palette.border.r, g: palette.border.g, b: palette.border.b, a: 1.0 };

    // Split into two columns
    let mid = (rows.len() + 1) / 2;
    let mut col_a = Vec::new();
    let mut col_b = Vec::new();

    for (i, r) in rows.into_iter().enumerate() {
        let cell = kv_row(r, palette, fs, border_c);
        if i < mid { col_a.push(cell); } else { col_b.push(cell); }
    }

    row![
        column(col_a).spacing(0).width(Length::Fill),
        column(col_b).spacing(0).width(Length::Fill),
    ]
    .spacing(24)
    .into()
}

fn kv_row<Msg: 'static>(r: KvRow, palette: &Palette, fs: f32, border_c: Color) -> Element<'static, Msg> {
    let key_color = palette.fg_dim;
    let val_color = palette.fg;

    container(
        row![
            text(r.key).size(fs).color(key_color).font(iced::Font::MONOSPACE).width(Length::Fill),
            text(r.value).size(fs).color(val_color).font(iced::Font::MONOSPACE),
        ]
        .padding(Padding { top: 3.0, bottom: 3.0, left: 0.0, right: 0.0 })
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        border: Border {
            color: Color { r: border_c.r, g: border_c.g, b: border_c.b, a: 0.5 },
            width: 0.0,
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}
