use iced::{widget::{column, scrollable, text}, Element, Length, Padding};
use crate::{
    data::model::QueryEntry,
    theme::Palette,
    ui::widgets::bson_view::bson_view,
};

pub fn response_tab<Msg: 'static>(entry: &QueryEntry, palette: &Palette, fs: f32) -> Element<'static, Msg> {
    let mut children: Vec<Element<'static, Msg>> = Vec::new();

    if let Some(doc) = &entry.doc {
        children.push(bson_view(doc, palette, fs));
    } else {
        let summary = match (entry.docs_returned.as_ref(), entry.docs_examined.as_ref()) {
            (Some(ret), Some(ex)) => {
                format!("{} document(s) returned · {} examined", ret.into_inner(), ex.into_inner())
            }
            (Some(ret), None) => format!("{} document(s) returned", ret.into_inner()),
            _ => "no response data".into(),
        };
        children.push(
            text(summary).size(fs).color(palette.fg_dim)
                .font(iced::Font::MONOSPACE).into()
        );
    }

    scrollable(
        column(children).spacing(8)
            .padding(Padding { top: 8.0, bottom: 8.0, left: 12.0, right: 12.0 })
    )
    .height(Length::Fill)
    .into()
}
