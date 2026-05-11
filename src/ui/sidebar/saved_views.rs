use iced::{widget::{button, column, row, text}, Border, Element, Length, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone)]
pub struct SavedView {
    pub id: usize,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum SavedViewsMsg {
    Load(usize),
    Delete(usize),
    Save,
}

pub fn saved_views_panel<Msg: Clone + 'static>(
    views: &[SavedView],
    on_msg: impl Fn(SavedViewsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0     = palette.bg;
    let bg_hover = palette.bg_hover;
    let fg      = palette.fg;
    let fg_dim  = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let accent  = palette.accent;
    let border  = palette.border;

    let mut rows: Vec<Element<Msg>> = views.iter().map(|v| {
        let id    = v.id;
        let label = v.label.clone();

        button(
            row![
                text("★").size(11).color(fg_dim2).font(iced::Font::MONOSPACE),
                text(label).size(11).color(fg).font(iced::Font::MONOSPACE),
                iced::widget::Space::new(Length::Fill, 0),
                button(text("×").size(11).color(fg_dim2).font(iced::Font::MONOSPACE))
                    .padding(Padding { top: 0.0, bottom: 0.0, left: 4.0, right: 0.0 })
                    .on_press(on_msg(SavedViewsMsg::Delete(id)))
                    .style(|_, _| button::Style { background: None, border: Border::default(), ..Default::default() }),
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center)
        )
        .padding(Padding { top: 5.0, bottom: 5.0, left: 8.0, right: 8.0 })
        .width(Length::Fill)
        .on_press(on_msg(SavedViewsMsg::Load(id)))
        .style(move |_, status| button::Style {
            background: Some(iced::Background::Color(
                match status {
                    iced::widget::button::Status::Hovered => bg_hover,
                    _ => bg0,
                }
            )),
            border: Border::default(),
            ..Default::default()
        })
        .into()
    }).collect();

    rows.push(
        button(
            row![
                text("+").size(12).color(fg_dim).font(iced::Font::MONOSPACE),
                text("Save current view").size(11).color(accent).font(iced::Font::MONOSPACE),
            ].spacing(5).align_y(iced::Alignment::Center)
        )
        .padding(Padding { top: 5.0, bottom: 5.0, left: 8.0, right: 8.0 })
        .width(Length::Fill)
        .on_press(on_msg(SavedViewsMsg::Save))
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(bg0)),
            border: Border { color: border, width: 1.0, radius: 3.0.into() },
            ..Default::default()
        })
        .into()
    );

    column(rows).spacing(2).padding(Padding { top: 4.0, bottom: 4.0, left: 0.0, right: 0.0 }).into()
}
