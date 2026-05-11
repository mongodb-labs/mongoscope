use iced::{widget::{column, container, row, scrollable, text}, Color, Element, Length, Padding};
use crate::{
    data::model::QueryEntry,
    theme::Palette,
    ui::widgets::schema_row::{schema_row, SchemaField},
};

pub fn schema_tab<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let fields = infer_fields(entry);
    let coll = entry.coll.as_str().to_string();
    let fs_small = (fs - 1.0).max(9.0);
    let border_c = Color { r: palette.border.r, g: palette.border.g, b: palette.border.b, a: 1.0 };

    // ── schema header with border-bottom
    let schemahd = column![
        row![
            text(format!("schema of shop.{}", coll))
                .size(fs_small).color(palette.fg).font(iced::Font::MONOSPACE),
            iced::widget::Space::new(8, 0),
            text("· inferred from 2,000 sampled docs")
                .size(fs_small).color(palette.fg_dim).font(iced::Font::MONOSPACE),
        ]
        .padding(Padding { top: 0.0, bottom: 6.0, left: 0.0, right: 0.0 }),
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

    scrollable(
        column(children)
            .spacing(0)
            .padding(Padding { top: 14.0, bottom: 14.0, left: 16.0, right: 16.0 })
    )
    .height(Length::Fill)
    .into()
}

fn infer_fields(entry: &QueryEntry) -> Vec<SchemaField> {
    let coll = entry.coll.as_str();
    if coll == "orders" {
        vec![
            SchemaField { name: "_id".into(), depth: 0, type_str: "ObjectId".into(), coverage: 1.00, samples: vec!["ObjectId(…)".into()] },
            SchemaField { name: "userId".into(), depth: 0, type_str: "ObjectId".into(), coverage: 1.00, samples: vec!["ObjectId(…)".into()] },
            SchemaField { name: "total".into(), depth: 0, type_str: "Decimal".into(), coverage: 1.00, samples: vec!["49.00".into(), "312.40".into()] },
            SchemaField { name: "status".into(), depth: 0, type_str: "enum".into(), coverage: 1.00, samples: vec!["paid".into(), "pending".into(), "shipped".into()] },
            SchemaField { name: "items".into(), depth: 0, type_str: "Array<Doc>".into(), coverage: 1.00, samples: vec!["[{…}, {…}]".into()] },
            SchemaField { name: "items.sku".into(), depth: 1, type_str: "String".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "items.qty".into(), depth: 1, type_str: "Int32".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "shipping".into(), depth: 0, type_str: "Doc".into(), coverage: 0.96, samples: vec!["{ country, city, … }".into()] },
            SchemaField { name: "shipping.country".into(), depth: 1, type_str: "String".into(), coverage: 0.96, samples: vec!["US".into(), "DE".into(), "JP".into()] },
            SchemaField { name: "coupon".into(), depth: 0, type_str: "String?".into(), coverage: 0.21, samples: vec!["SUMMER26".into()] },
            SchemaField { name: "notes".into(), depth: 0, type_str: "String?".into(), coverage: 0.04, samples: vec![] },
            SchemaField { name: "createdAt".into(), depth: 0, type_str: "Date".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "updatedAt".into(), depth: 0, type_str: "Date".into(), coverage: 0.99, samples: vec![] },
        ]
    } else if coll == "products" {
        vec![
            SchemaField { name: "_id".into(), depth: 0, type_str: "ObjectId".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "sku".into(), depth: 0, type_str: "String".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "name".into(), depth: 0, type_str: "String".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "price".into(), depth: 0, type_str: "Decimal".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "category".into(), depth: 0, type_str: "String".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "tags".into(), depth: 0, type_str: "Array<String>".into(), coverage: 0.94, samples: vec![] },
            SchemaField { name: "inStock".into(), depth: 0, type_str: "Bool".into(), coverage: 1.00, samples: vec![] },
            SchemaField { name: "popularity".into(), depth: 0, type_str: "Int32".into(), coverage: 0.88, samples: vec![] },
        ]
    } else {
        let mut fields = vec![
            SchemaField { name: "_id".into(), depth: 0, type_str: "ObjectId".into(), coverage: 1.0, samples: vec!["ObjectId(…)".into()] },
        ];
        for key in entry.filter.as_ref().map(|f| f.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
            let (type_str, samples) = field_type_samples(&key);
            let depth = key.matches('.').count();
            fields.push(SchemaField { name: key, depth, type_str, coverage: 0.95, samples });
        }
        fields
    }
}

fn field_type_samples(key: &str) -> (String, Vec<String>) {
    if key.ends_with("At") || key.ends_with("Date") {
        ("ISODate".into(), vec!["ISODate(\"2024-01-15\")".into()])
    } else if key == "_id" || key.ends_with("Id") || key.ends_with("ID") {
        ("ObjectId".into(), vec!["ObjectId(\"...\")".into()])
    } else if key == "status" {
        ("String".into(), vec!["\"pending\"".into(), "\"shipped\"".into()])
    } else if key.contains("count") || key.contains("Count") || key.contains("price") {
        ("Number".into(), vec!["42".into(), "100".into()])
    } else {
        ("String".into(), vec!["\"…\"".into()])
    }
}
