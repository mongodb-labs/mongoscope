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

    /// Replace any existing `db:` and `coll:` tokens in `self.text` with the given values,
    /// preserving all other tokens. Passing `None` removes the token.
    pub fn set_scope(&mut self, db: Option<String>, coll: Option<String>) {
        // Strip existing db: and coll: tokens
        let rest: String = self.text
            .split_whitespace()
            .filter(|t| !t.starts_with("db:") && !t.starts_with("coll:"))
            .collect::<Vec<_>>()
            .join(" ");

        let mut parts: Vec<String> = Vec::new();
        if let Some(d) = db {
            parts.push(format!("db:{}", d));
        }
        if let Some(c) = coll {
            parts.push(format!("coll:{}", c));
        }
        if !rest.is_empty() {
            parts.push(rest);
        }

        self.text = parts.join(" ");
        self.expr = FilterExpr::parse(&self.text);
    }

    pub fn view<'a, Msg: Clone + 'static>(
        &'a self,
        on_msg: impl Fn(FilterMsg) -> Msg + 'static + Copy,
        on_pause: Msg,
        on_clear: Msg,
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

        let clear_btn = button(
            text("✕").size(11).color(fg_dim2).font(iced::Font::MONOSPACE)
        )
        .padding(Padding { top: 3.0, bottom: 3.0, left: 6.0, right: 6.0 })
        .on_press(on_clear)
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        });

        container(
            row![
                search_input(
                    self.text.clone(),
                    "filter: db:shop coll:orders app:api slow",
                    move |t| on_msg(FilterMsg::TextChanged(t)),
                    on_msg(FilterMsg::TextSubmit),
                    &palette,
                ),
                kind_chips(self.kind, move |k| on_msg(FilterMsg::KindSelected(k)), &palette),
                iced::widget::Space::new(Length::Fill, 0),
                text(count_str).size(11).color(count_color).font(iced::Font::MONOSPACE),
                pause_btn,
                clear_btn,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_scope_injects_db_token() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), None);
        assert_eq!(fs.text, "db:shop");
        assert_eq!(fs.expr.db, Some("shop".into()));
        assert_eq!(fs.expr.coll, None);
    }

    #[test]
    fn set_scope_injects_db_and_coll() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        assert_eq!(fs.text, "db:shop coll:orders");
    }

    #[test]
    fn set_scope_replaces_existing_db_token() {
        let mut fs = FilterState::new();
        fs.text = "db:old coll:x foo".into();
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        assert_eq!(fs.text, "db:shop coll:orders foo");
    }

    #[test]
    fn set_scope_none_removes_tokens() {
        let mut fs = FilterState::new();
        fs.text = "db:shop coll:orders foo".into();
        fs.set_scope(None, None);
        assert_eq!(fs.text, "foo");
    }

    #[test]
    fn set_scope_db_only_removes_coll() {
        let mut fs = FilterState::new();
        fs.text = "db:shop coll:orders".into();
        fs.set_scope(Some("shop".into()), None);
        assert_eq!(fs.text, "db:shop");
    }
}
