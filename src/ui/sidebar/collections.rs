use iced::{widget::{button, column, row, text}, Border, Element, Length, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone)]
pub struct CollectionItem {
    pub name: String,
    pub docs: u64,
    pub size: String,
    pub idx: u8,
    pub active: bool,
}

impl CollectionItem {
    pub fn docs_str(&self) -> String {
        let d = self.docs;
        if d >= 1_000_000 { format!("{:.1}M docs", d as f64 / 1_000_000.0) }
        else if d >= 1_000 { format!("{:.0}K docs", d as f64 / 1_000.0) }
        else { format!("{} docs", d) }
    }
}

#[derive(Debug, Clone)]
pub enum CollectionsMsg {
    Select(String),
}

pub fn collections_panel<Msg: Clone + 'static>(
    items: &[CollectionItem],
    on_msg: impl Fn(CollectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0      = palette.bg;
    let bg_sel   = palette.bg_sel;
    let bg_hover = palette.bg_hover;
    let fg       = palette.fg;
    let fg_dim   = palette.fg_dim;
    let fg_dim2  = palette.fg_dim2;

    let rows: Vec<Element<Msg>> = items.iter().map(|item| {
        let is_active = item.active;
        let bg   = if is_active { bg_sel } else { bg0 };
        let name = item.name.clone();
        let sub  = format!("{} · {}", item.docs_str(), item.size);
        let idx  = format!("{}i", item.idx);
        let name_click = name.clone();

        button(
            row![
                text("◧").size(11).color(fg_dim2).font(iced::Font::MONOSPACE),
                column![
                    text(name).size(11).color(fg).font(iced::Font::MONOSPACE),
                    text(sub).size(9).color(fg_dim2).font(iced::Font::MONOSPACE),
                ].spacing(1).width(Length::Fill),
                text(idx).size(9).color(fg_dim).font(iced::Font::MONOSPACE),
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center)
        )
        .padding(Padding { top: 5.0, bottom: 5.0, left: 8.0, right: 8.0 })
        .width(Length::Fill)
        .on_press(on_msg(CollectionsMsg::Select(name_click)))
        .style(move |_, status| button::Style {
            background: Some(iced::Background::Color(
                match status {
                    iced::widget::button::Status::Hovered if !is_active => bg_hover,
                    _ => bg,
                }
            )),
            border: Border::default(),
            ..Default::default()
        })
        .into()
    }).collect();

    column(rows).spacing(1).padding(Padding { top: 4.0, bottom: 4.0, left: 0.0, right: 0.0 }).into()
}
