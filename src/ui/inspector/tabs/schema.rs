use iced::{widget::{column, scrollable, text}, Element, Length, Padding};
use crate::{
    data::model::QueryEntry,
    theme::Palette,
    ui::widgets::schema_row::{schema_row, SchemaField},
};

pub fn schema_tab<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let fields = infer_fields(entry);

    let mut children: Vec<Element<'static, Msg>> = vec![
        text(format!("Schema · {}", entry.coll.as_str()))
            .size(fs - 1.0).color(palette.fg_dim2)
            .font(iced::Font::MONOSPACE).into(),
    ];

    for field in fields {
        children.push(schema_row::<Msg>(field, palette, fs));
    }

    scrollable(
        column(children).spacing(0)
            .padding(Padding { top: 8.0, bottom: 8.0, left: 0.0, right: 0.0 })
    )
    .height(Length::Fill)
    .into()
}

fn infer_fields(entry: &QueryEntry) -> Vec<SchemaField> {
    // Build mock schema from filter keys and pipeline stages
    let mut fields = vec![
        SchemaField { name: "_id".into(), depth: 0, type_str: "ObjectId".into(), coverage: 1.0, samples: vec!["ObjectId(...)".into()] },
    ];

    for key in entry.filter.as_ref().map(|f| f.keys().cloned().collect::<Vec<_>>()).unwrap_or_default() {
        let (type_str, samples) = field_type_samples(&key);
        let depth = key.matches('.').count();
        fields.push(SchemaField {
            name: key.clone(),
            depth,
            type_str,
            coverage: 0.95,
            samples,
        });
    }

    fields
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
