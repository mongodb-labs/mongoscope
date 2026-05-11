use iced::{widget::{button, row}, Border, Element, Length, Padding};
use crate::{data::types::QueryId, theme::Palette, ui::sidebar::clients::app_color_for};
use super::cells::*;
use crate::data::model::QueryEntry;

pub fn table_row<Msg: Clone + 'static>(
    entry: &QueryEntry,
    selected: bool,
    on_select: impl Fn(QueryId) -> Msg + Copy,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let bg_sel  = palette.bg_sel;
    let bg_hover = palette.bg_hover;
    let bg      = if selected { bg_sel } else { palette.bg };
    let border_color = palette.border;
    let id = entry.id;
    let app_color = app_color_for(entry.app.as_str());

    let content = row![
        time_cell::<Msg>(entry, palette, fs),
        op_cell::<Msg>(entry, palette, fs),
        coll_cell::<Msg>(entry, palette, fs),
        app_cell::<Msg>(entry, app_color, palette, fs),
        plan_cell::<Msg>(entry, palette, fs),
        docs_cell::<Msg>(entry, palette, fs),
        warn_cell::<Msg>(entry, palette, fs),
        latency_cell::<Msg>(entry, palette, fs),
    ]
    .spacing(4)
    .padding(Padding { top: 3.0, bottom: 3.0, left: 10.0, right: 10.0 })
    .align_y(iced::Alignment::Center);

    button(content)
        .width(Length::Fill)
        .height(28)
        .padding(Padding::ZERO)
        .on_press(on_select(id))
        .style(move |_, status| {
            let bg_actual = match status {
                iced::widget::button::Status::Hovered => bg_hover,
                _ => bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg_actual)),
                border: Border { color: border_color, width: 0.0, radius: 0.0.into() },
                ..Default::default()
            }
        })
        .into()
}
