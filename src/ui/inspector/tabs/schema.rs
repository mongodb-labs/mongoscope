use crate::{
    data::model::{CollectionSchema, QueryEntry},
    theme::Palette,
    ui::widgets::schema_row::{schema_row, SchemaField},
};
use iced::{
    widget::{column, container, row, scrollable, text},
    Color, Element, Length, Padding,
};

pub fn schema_tab<Msg: 'static>(
    entry: &QueryEntry,
    schema: Option<&CollectionSchema>,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let db = entry.db.as_str().to_string();
    let coll = entry.coll.as_str().to_string();
    let fs_small = (fs - 1.0).max(9.0);
    let border_c = Color {
        r: palette.border.r,
        g: palette.border.g,
        b: palette.border.b,
        a: 1.0,
    };

    match schema {
        None => scrollable(
            column![text(format!("No schema available for {}.{}", db, coll))
                .size(fs_small)
                .color(palette.fg_dim)
                .font(iced::Font::MONOSPACE)]
            .padding(Padding {
                top: 14.0,
                bottom: 14.0,
                left: 16.0,
                right: 16.0,
            }),
        )
        .height(Length::Fill)
        .into(),
        Some(s) => {
            let sampled_docs = s.sampled_docs;
            let fields: Vec<SchemaField> = s
                .fields
                .iter()
                .map(|f| SchemaField {
                    name: f.name.to_string(),
                    depth: f.name.matches('.').count(),
                    type_str: f.type_str.to_string(),
                    coverage: f.coverage_pct as f32 / 100.0,
                    samples: f.samples.iter().map(|s| s.to_string()).collect(),
                })
                .collect();

            // ── schema header with border-bottom
            let schemahd = column![
                row![
                    text(format!("schema of {}.{}", db, coll))
                        .size(fs_small)
                        .color(palette.fg)
                        .font(iced::Font::MONOSPACE),
                    iced::widget::Space::new(8, 0),
                    text(format!("· inferred from {} sampled docs", sampled_docs))
                        .size(fs_small)
                        .color(palette.fg_dim)
                        .font(iced::Font::MONOSPACE),
                ]
                .padding(Padding {
                    top: 0.0,
                    bottom: 6.0,
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
            .width(Length::Fill);

            let mut children: Vec<Element<'static, Msg>> = vec![schemahd.into()];
            for field in fields {
                children.push(schema_row::<Msg>(field, palette, fs));
            }

            scrollable(column(children).spacing(0).padding(Padding {
                top: 14.0,
                bottom: 14.0,
                left: 16.0,
                right: 16.0,
            }))
            .height(Length::Fill)
            .into()
        }
    }
}
