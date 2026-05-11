pub mod cells;
pub mod header;
pub mod row;

pub use header::table_header;
pub use row::table_row;

use iced::{widget::column, Element, Length};
use crate::{data::{model::QueryEntry, types::QueryId}, theme::Palette};

pub fn table_view<Msg: Clone + 'static>(
    entries: &[&QueryEntry],
    selected: Option<QueryId>,
    on_select: impl Fn(QueryId) -> Msg + Copy,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let header = table_header::<Msg>(palette, fs);

    let rows: Vec<Element<Msg>> = entries.iter().map(|entry| {
        let is_selected = selected.map_or(false, |id| id == entry.id);
        table_row(entry, is_selected, on_select, palette, fs)
    }).collect();

    column![
        header,
        column(rows).spacing(0).width(Length::Fill),
    ]
    .spacing(0)
    .width(Length::Fill)
    .into()
}
