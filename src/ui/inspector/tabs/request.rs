use iced::{widget::{column, container, row, scrollable, text}, Border, Element, Length, Padding};
use crate::{
    data::model::QueryEntry,
    theme::Palette,
    ui::widgets::bson_view::bson_view,
};

pub fn request_tab<'a, Msg: Clone + 'static>(entry: &'a QueryEntry, palette: &Palette, fs: f32) -> Element<'a, Msg> {
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let fg = palette.fg;
    let bg1 = palette.bg1;
    let border_color = palette.border;
    let id = entry.id.into_inner();

    // Approximate wire bytes: sum lengths of filter/pipeline/update fields
    let approx_bytes = {
        let base = 80usize;
        let filter_bytes = entry.filter.as_ref().map(|f| format!("{:?}", f).len()).unwrap_or(0);
        let pipe_bytes = entry.pipeline.as_ref().map(|p| p.len() * 40).unwrap_or(0);
        let upd_bytes = entry.update.as_ref().map(|u| format!("{:?}", u).len()).unwrap_or(0);
        base + filter_bytes + pipe_bytes + upd_bytes
    };

    // ── Request header
    let header = container(
        row![
            row![
                text("OP_MSG").size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
                text(" · ").size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
                text(format!("msg_id={}", id)).size(10).color(fg).font(iced::Font::MONOSPACE),
                text(" · ").size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
                text(format!("{} B", approx_bytes)).size(10).color(fg).font(iced::Font::MONOSPACE),
            ].align_y(iced::Alignment::Center),
            iced::widget::Space::new(Length::Fill, 0),
            row![
                text("copy").size(10).color(fg_dim).font(iced::Font::MONOSPACE),
                text("  as shell").size(10).color(fg_dim).font(iced::Font::MONOSPACE),
                text("  raw bytes").size(10).color(fg_dim).font(iced::Font::MONOSPACE),
            ].align_y(iced::Alignment::Center),
        ]
        .align_y(iced::Alignment::Center)
        .padding(Padding { top: 6.0, bottom: 6.0, left: 12.0, right: 8.0 })
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg1)),
        border: Border { color: border_color, width: 1.0, radius: 0.0.into() },
        ..Default::default()
    });

    let mut bson_children: Vec<Element<Msg>> = Vec::new();

    // Show $db + op label as first fields
    bson_children.push(
        text(format!("$db: \"shop\"")).size(fs).color(palette.tok_str)
            .font(iced::Font::MONOSPACE).into()
    );
    let op_label = entry.op.label().to_string();
    let coll = entry.coll.as_str().to_string();
    bson_children.push(
        text(format!("{}: \"{}\"", op_label.to_lowercase(), coll))
            .size(fs).color(palette.tok_str).font(iced::Font::MONOSPACE).into()
    );

    if let Some(filter) = &entry.filter {
        bson_children.push(
            text("filter:").size(fs - 1.0).color(fg_dim2).font(iced::Font::MONOSPACE).into()
        );
        bson_children.push(bson_view(filter, palette, fs));
    }

    if let Some(pipeline) = &entry.pipeline {
        bson_children.push(
            text("pipeline:").size(fs - 1.0).color(fg_dim2).font(iced::Font::MONOSPACE).into()
        );
        for stage in pipeline {
            bson_children.push(bson_view(stage, palette, fs));
        }
    }

    if let Some(update) = &entry.update {
        bson_children.push(
            text("updates:").size(fs - 1.0).color(fg_dim2).font(iced::Font::MONOSPACE).into()
        );
        bson_children.push(bson_view(update, palette, fs));
    }

    bson_children.push(
        text("lsid: { id: UUID(\"a4b...\") }").size(fs).color(fg_dim2)
            .font(iced::Font::MONOSPACE).into()
    );

    if bson_children.len() == 2 {
        bson_children.push(
            text("—").size(fs).color(fg_dim2).font(iced::Font::MONOSPACE).into()
        );
    }

    let body = scrollable(
        column(bson_children).spacing(4)
            .padding(Padding { top: 8.0, bottom: 8.0, left: 12.0, right: 12.0 })
    )
    .height(Length::Fill);

    column![
        header,
        body,
    ]
    .spacing(0)
    .into()
}
