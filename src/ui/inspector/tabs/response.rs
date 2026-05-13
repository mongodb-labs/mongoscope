use crate::{data::model::QueryEntry, theme::Palette, ui::widgets::bson_view::bson_view};
use iced::{
    widget::{column, container, row, scrollable, text},
    Border, Color, Element, Length, Padding,
};

fn ghost_action<'a, Msg: 'a>(
    label: &'a str,
    fg: Color,
    bg: Color,
    border: Color,
) -> Element<'a, Msg> {
    container(text(label).size(10).color(fg).font(iced::Font::MONOSPACE))
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 10.0,
            right: 10.0,
        })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border {
                color: border,
                width: 1.0,
                radius: 5.0.into(),
            },
            ..Default::default()
        })
        .into()
}

pub fn response_tab<Msg: 'static>(
    entry: &QueryEntry,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let bg = palette.bg;
    let bg1 = palette.bg1;
    let border_color = palette.border;
    let fs_small = (fs - 1.0).max(9.0);

    let n_docs = entry.response_docs.len() as u64;
    let byte_est = entry.response_docs.len() * 412;

    // ── Response header
    let header = container(
        row![
            row![
                text("OP_MSG")
                    .size(fs_small)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
                text(" · ")
                    .size(fs_small)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
                text("response")
                    .size(fs_small)
                    .color(fg_dim)
                    .font(iced::Font::MONOSPACE),
                text(" · ")
                    .size(fs_small)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
                text(format!("{} docs", n_docs))
                    .size(fs_small)
                    .color(fg)
                    .font(iced::Font::MONOSPACE),
                text(" · ")
                    .size(fs_small)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
                text(format!("{} B", byte_est))
                    .size(fs_small)
                    .color(fg)
                    .font(iced::Font::MONOSPACE),
            ]
            .align_y(iced::Alignment::Center),
            iced::widget::Space::new(Length::Fill, 0),
            row![
                ghost_action("export", fg_dim, bg, border_color),
                ghost_action("as json", fg_dim, bg, border_color),
                ghost_action("diff", fg_dim, bg, border_color),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        ]
        .align_y(iced::Alignment::Center)
        .padding(Padding {
            top: 8.0,
            bottom: 8.0,
            left: 14.0,
            right: 8.0,
        }),
    )
    .width(Length::Fill)
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg1)),
        border: Border {
            color: border_color,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    });

    let mut doc_elements: Vec<Element<'static, Msg>> = Vec::new();
    for doc in &entry.response_docs {
        doc_elements.push(bson_view(doc, palette, fs));
    }

    let body = scrollable(column(doc_elements).spacing(6).padding(Padding {
        top: 10.0,
        bottom: 10.0,
        left: 8.0,
        right: 8.0,
    }))
    .height(Length::Fill);

    column![header, body].spacing(0).into()
}
