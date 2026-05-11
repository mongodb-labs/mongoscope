use iced::{widget::{column, container, row, scrollable, text}, Border, Color, Element, Length, Padding};
use indexmap::IndexMap;
use crate::{
    data::model::{BsonDoc, BsonVal, Op, QueryEntry},
    theme::Palette,
    ui::widgets::bson_view::bson_view,
};

fn ghost_action<Msg: 'static>(label: &str, fg: Color, bg: Color, border: Color) -> Element<'static, Msg> {
    container(text(label.to_string()).size(10).color(fg).font(iced::Font::MONOSPACE))
        .padding(Padding { top: 3.0, bottom: 3.0, left: 10.0, right: 10.0 })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: border, width: 1.0, radius: 5.0.into() },
            ..Default::default()
        })
        .into()
}

fn mock_doc(coll: &str, i: usize) -> BsonDoc {
    let mut doc: BsonDoc = IndexMap::new();
    doc.insert("_id".into(), BsonVal::ObjectId(format!("66f{:02x}c4", i + 1)));
    match coll {
        "orders" => {
            doc.insert("userId".into(), BsonVal::ObjectId("65fe21c3a8b4e9c2d1f04a12".into()));
            doc.insert("total".into(), BsonVal::Float(49.0 + i as f64 * 17.33));
            doc.insert("status".into(), BsonVal::Str("paid".into()));
            doc.insert("createdAt".into(), BsonVal::IsoDate(format!("2026-04-{}T09:{:02}:41Z", 20 + i, 12 + i)));
        }
        "products" => {
            let names = ["Linen Shirt", "Field Jacket", "Canvas Sneakers"];
            doc.insert("sku".into(), BsonVal::Str(format!("SKU-{}-BLK-M", 88421 + i)));
            doc.insert("name".into(), BsonVal::Str(names.get(i).copied().unwrap_or("—").into()));
            doc.insert("price".into(), BsonVal::Int((49 + i as i64 * 20) as i64));
            doc.insert("inStock".into(), BsonVal::Bool(true));
        }
        _ => {
            doc.insert("type".into(), BsonVal::Str("event".into()));
            doc.insert("ts".into(), BsonVal::IsoDate(format!("2026-04-24T{}:22:08Z", 14 + i)));
        }
    }
    doc
}

fn build_response(entry: &QueryEntry) -> BsonDoc {
    let coll = entry.coll.as_str();
    let is_read = matches!(&entry.op, Op::Find | Op::FindOne | Op::Aggregate | Op::CountDocuments);
    let n = entry.docs_returned.as_ref().map(|d| d.into_inner()).unwrap_or(1) as usize;

    if is_read {
        let count = n.min(3);
        let batch: Vec<BsonVal> = (0..count).map(|i| BsonVal::Doc(mock_doc(coll, i))).collect();
        let mut cursor: BsonDoc = IndexMap::new();
        cursor.insert("firstBatch".into(), BsonVal::Array(batch));
        cursor.insert("id".into(), BsonVal::NumberLong(0));
        cursor.insert("ns".into(), BsonVal::Str(format!("shop.{}", coll)));
        let mut resp: BsonDoc = IndexMap::new();
        resp.insert("cursor".into(), BsonVal::Doc(cursor));
        resp.insert("ok".into(), BsonVal::Int(1));
        resp
    } else {
        let mut resp: BsonDoc = IndexMap::new();
        resp.insert("n".into(), BsonVal::Int(n as i64));
        if matches!(&entry.op, Op::UpdateOne | Op::UpdateMany) {
            resp.insert("nModified".into(), BsonVal::Int(n as i64));
        }
        resp.insert("ok".into(), BsonVal::Int(1));
        resp
    }
}

pub fn response_tab<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let bg = palette.bg;
    let bg1 = palette.bg1;
    let border_color = palette.border;
    let fs_small = (fs - 1.0).max(9.0);

    let n_docs = entry.docs_returned.as_ref().map(|d| d.into_inner()).unwrap_or(1);
    let byte_est = n_docs as usize * 412;

    // ── Response header
    let header = container(
        row![
            row![
                text("OP_MSG").size(fs_small).color(fg_dim2).font(iced::Font::MONOSPACE),
                text(" · ").size(fs_small).color(fg_dim2).font(iced::Font::MONOSPACE),
                text("response").size(fs_small).color(fg_dim).font(iced::Font::MONOSPACE),
                text(" · ").size(fs_small).color(fg_dim2).font(iced::Font::MONOSPACE),
                text(format!("{} docs", n_docs)).size(fs_small).color(fg).font(iced::Font::MONOSPACE),
                text(" · ").size(fs_small).color(fg_dim2).font(iced::Font::MONOSPACE),
                text(format!("{} B", byte_est)).size(fs_small).color(fg).font(iced::Font::MONOSPACE),
            ].align_y(iced::Alignment::Center),
            iced::widget::Space::new(Length::Fill, 0),
            row![
                ghost_action("export", fg_dim, bg, border_color),
                ghost_action("as json", fg_dim, bg, border_color),
                ghost_action("diff", fg_dim, bg, border_color),
            ].spacing(4).align_y(iced::Alignment::Center),
        ]
        .align_y(iced::Alignment::Center)
        .padding(Padding { top: 8.0, bottom: 8.0, left: 14.0, right: 8.0 })
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg1)),
        border: Border { color: border_color, width: 0.0, radius: 0.0.into() },
        ..Default::default()
    });

    let resp_doc = build_response(entry);
    let bson_el = bson_view(&resp_doc, palette, fs);

    let body = scrollable(
        column![bson_el]
            .padding(Padding { top: 10.0, bottom: 10.0, left: 8.0, right: 8.0 })
    )
    .height(Length::Fill);

    column![header, body].spacing(0).into()
}
