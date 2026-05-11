use iced::{widget::{button, column, row, text}, Border, Element, Length, Padding};
use crate::{theme::Palette, ui::widgets::appdot::appdot};

pub fn app_color_for(app: &str) -> [u8; 3] {
    let hash: u32 = app.bytes().fold(5381u32, |h, b| h.wrapping_mul(33).wrapping_add(b as u32));
    let hue = (hash % 360) as f32;
    hsl_to_rgb(hue, 0.65, 0.55)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [u8; 3] {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = if h < 60.0 { (c, x, 0.0) }
        else if h < 120.0 { (x, c, 0.0) }
        else if h < 180.0 { (0.0, c, x) }
        else if h < 240.0 { (0.0, x, c) }
        else if h < 300.0 { (x, 0.0, c) }
        else { (c, 0.0, x) };
    [((r1 + m) * 255.0) as u8, ((g1 + m) * 255.0) as u8, ((b1 + m) * 255.0) as u8]
}

#[derive(Debug, Clone)]
pub struct ClientItem {
    pub name: String,
    pub color: [u8; 3],
    pub active: bool,
}

#[derive(Debug, Clone)]
pub enum ClientsMsg {
    Toggle(String),
}

pub fn clients_panel<Msg: Clone + 'static>(
    items: &[ClientItem],
    on_msg: impl Fn(ClientsMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0     = palette.bg;
    let bg_sel  = palette.bg_sel;
    let bg_hover = palette.bg_hover;
    let fg      = palette.fg;
    let fg_dim2 = palette.fg_dim2;

    if items.is_empty() {
        return column![
            text("No clients seen yet").size(10).color(fg_dim2).font(iced::Font::MONOSPACE)
        ]
        .padding(Padding { top: 6.0, bottom: 6.0, left: 8.0, right: 8.0 })
        .into();
    }

    let rows: Vec<Element<Msg>> = items.iter().map(|item| {
        let is_active = item.active;
        let bg    = if is_active { bg_sel } else { bg0 };
        let name  = item.name.clone();
        let nclick = name.clone();
        let color = item.color;

        button(
            row![
                appdot::<Msg>(color),
                text(name).size(11).color(fg).font(iced::Font::MONOSPACE),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center)
        )
        .padding(Padding { top: 5.0, bottom: 5.0, left: 8.0, right: 8.0 })
        .width(Length::Fill)
        .on_press(on_msg(ClientsMsg::Toggle(nclick)))
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
