use iced::{widget::{button, column, container, row, text}, Border, Element, Length, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone)]
pub struct ConnectionItem {
    pub id: usize,
    pub label: String,
    pub topology: String,  // e.g. "mongodb+srv · 3 nodes · rs0" or "direct"
    pub active: bool,
    pub live: bool,
}

#[derive(Debug, Clone)]
pub enum ConnectionsMsg {
    Select(usize),
    Add,
}

pub fn connections_panel<Msg: Clone + 'static>(
    items: &[ConnectionItem],
    on_msg: impl Fn(ConnectionsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0     = palette.bg;
    let bg_sel  = palette.bg_sel;
    let bg_hover = palette.bg_hover;
    let fg      = palette.fg;
    let fg_dim  = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let ok      = palette.ok;
    let accent  = palette.accent;

    let rows: Vec<Element<Msg>> = items.iter().map(|item| {
        let is_active = item.active;
        let bg      = if is_active { bg_sel } else { bg0 };
        let dot_color = if item.live { ok } else { fg_dim2 };
        let id      = item.id;
        let label   = item.label.clone();
        let topo    = item.topology.clone();
        let live    = item.live;

        let dot = container(iced::widget::Space::new(7, 7))
            .width(7).height(7)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(dot_color)),
                border: Border { radius: 3.5.into(), ..Default::default() },
                ..Default::default()
            });

        let mut inner = row![
            dot,
            column![
                text(label).size(11).color(fg).font(iced::Font::MONOSPACE),
                text(topo).size(10).color(fg_dim2).font(iced::Font::MONOSPACE),
            ].spacing(1).width(Length::Fill),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        if live {
            inner = inner.push(
                container(text("LIVE").size(9).color(accent).font(iced::Font::MONOSPACE))
                    .padding(Padding { top: 1.0, bottom: 1.0, left: 4.0, right: 4.0 })
                    .style(move |_| container::Style {
                        border: Border { color: accent, width: 1.0, radius: 2.0.into() },
                        ..Default::default()
                    })
            );
        }

        button(inner)
            .padding(Padding { top: 6.0, bottom: 6.0, left: 8.0, right: 8.0 })
            .width(Length::Fill)
            .on_press(on_msg(ConnectionsMsg::Select(id)))
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

    let add_btn = button(
        row![
            text("+").size(12).color(fg_dim).font(iced::Font::MONOSPACE),
            text("Add connection").size(11).color(fg_dim).font(iced::Font::MONOSPACE),
        ].spacing(6).align_y(iced::Alignment::Center)
    )
    .padding(Padding { top: 5.0, bottom: 5.0, left: 8.0, right: 8.0 })
    .width(Length::Fill)
    .on_press(on_msg(ConnectionsMsg::Add))
    .style(move |_, _| button::Style {
        background: None,
        border: Border::default(),
        ..Default::default()
    });

    let mut col = column(rows).spacing(1);
    col = col.push(add_btn);
    col.padding(Padding { top: 4.0, bottom: 4.0, left: 0.0, right: 0.0 }).into()
}
