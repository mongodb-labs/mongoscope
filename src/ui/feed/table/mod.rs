pub mod cells;
pub mod header;
pub mod row;

pub use header::table_header;
pub use row::table_row;

use crate::{
    data::{model::QueryEntry, types::QueryId},
    theme::Palette,
};
use iced::{widget::column, Element, Length};

pub fn table_view<Msg: Clone + 'static>(
    entries: &[&QueryEntry],
    selected: Option<QueryId>,
    on_select: impl Fn(QueryId) -> Msg + Copy,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let rows: Vec<Element<Msg>> = entries
        .iter()
        .map(|entry| {
            let is_selected = selected == Some(entry.id);
            table_row(entry, is_selected, on_select, palette, fs)
        })
        .collect();

    column(rows).spacing(0).width(Length::Fill).into()
}
