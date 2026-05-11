pub mod kind_chips;
pub mod parser;
pub mod search_input;

pub use kind_chips::{kind_chips, KindFilter};
pub use parser::FilterExpr;
pub use search_input::search_input;

use iced::{widget::{button, container, row, text}, Border, Element, Length, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone)]
pub enum FilterMsg {
    TextChanged(String),
    TextSubmit,
    KindSelected(KindFilter),
    ClearFilter,
}

pub struct FilterState {
    pub text: String,
    pub kind: KindFilter,
    pub expr: FilterExpr,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            kind: KindFilter::All,
            expr: FilterExpr::default(),
        }
    }

    pub fn update(&mut self, msg: FilterMsg) {
        match msg {
            FilterMsg::TextChanged(t) => {
                self.expr = FilterExpr::parse(&t);
                self.text = t;
            }
            FilterMsg::TextSubmit => {
                self.expr = FilterExpr::parse(&self.text);
            }
            FilterMsg::KindSelected(k) => {
                self.kind = k;
            }
            FilterMsg::ClearFilter => {
                self.text.clear();
                self.kind = KindFilter::All;
                self.expr = FilterExpr::default();
            }
        }
    }

    pub fn view<'a, Msg: Clone + 'static>(
        &'a self,
        on_msg: impl Fn(FilterMsg) -> Msg + 'static + Copy,
        on_pause: Msg,
        paused: bool,
        visible_count: usize,
        total_count: usize,
        palette: Palette,
    ) -> Element<'a, Msg> {
        let bg1 = palette.bg1;
        let border_color = palette.border;
        let fg_dim = palette.fg_dim;
        let fg_dim2 = palette.fg_dim2;
        let accent = palette.accent;
        let warn = palette.warn;

        let count_str = format!("{}/{}", visible_count, total_count);
        let count_color = if visible_count < total_count { accent } else { fg_dim2 };

        // Pause button: ‖ when capturing, ▶ when paused
        let pause_label = if paused { "▶" } else { "||" };
        let pause_color = if paused { warn } else { fg_dim };

        let pause_btn = button(
            text(pause_label).size(11).color(pause_color).font(iced::Font::MONOSPACE)
        )
        .padding(Padding { top: 3.0, bottom: 3.0, left: 6.0, right: 6.0 })
        .on_press(on_pause)
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        });

        container(
            row![
                search_input(
                    &self.text,
                    "filter: coll:orders app:api slow",
                    move |t| on_msg(FilterMsg::TextChanged(t)),
                    on_msg(FilterMsg::TextSubmit),
                    &palette,
                ),
                kind_chips(self.kind, move |k| on_msg(FilterMsg::KindSelected(k)), &palette),
                iced::widget::Space::new(Length::Fill, 0),
                text(count_str).size(11).color(count_color).font(iced::Font::MONOSPACE),
                pause_btn,
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding(Padding { top: 6.0, bottom: 6.0, left: 10.0, right: 6.0 })
        )
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg1)),
            border: Border { color: border_color, width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }
}
