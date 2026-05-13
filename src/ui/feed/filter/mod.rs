pub mod kind_chips;
pub mod parser;
pub mod search_input;

pub use kind_chips::{kind_chips, KindFilter};
pub use parser::{Filter, Preset};
pub use search_input::search_input;

use crate::{data::model::QueryEntry, theme::Palette};
use iced::{
    widget::{button, container, row, text},
    Border, Element, Length, Padding,
};

#[derive(Debug, Clone)]
pub enum FilterMsg {
    TextChanged(String),
    TextSubmit,
    KindSelected(KindFilter),
    #[allow(dead_code)]
    ClearFilter,
}

pub struct FilterState {
    pub input: String,
    pub filter: Filter,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            filter: Filter::default(),
        }
    }

    fn sync_input(&mut self) {
        self.input = self.filter.to_string();
    }

    pub fn set_scope(&mut self, db: Option<String>, coll: Option<String>) {
        self.filter.db = db;
        self.filter.coll = coll;
        self.sync_input();
    }

    pub fn set_app(&mut self, app: Option<String>) {
        self.filter.app = app;
        self.sync_input();
    }

    pub fn set_preset(&mut self, preset: Option<Preset>) {
        self.filter.preset = preset;
        self.sync_input();
    }

    pub fn active_preset(&self) -> Option<Preset> {
        self.filter.preset
    }

    pub fn matches(&self, entry: &QueryEntry) -> bool {
        self.filter.matches(entry)
    }

    pub fn update(&mut self, msg: FilterMsg) {
        match msg {
            FilterMsg::TextChanged(t) => {
                let kind = self.filter.kind;
                self.filter = Filter::parse(&t);
                self.filter.kind = kind;
                self.input = t;
            }
            FilterMsg::TextSubmit => {
                let kind = self.filter.kind;
                self.filter = Filter::parse(&self.input);
                self.filter.kind = kind;
            }
            FilterMsg::KindSelected(k) => {
                self.filter.kind = k;
            }
            FilterMsg::ClearFilter => {
                self.input.clear();
                self.filter = Filter::default();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn view<'a, Msg: Clone + 'static>(
        &'a self,
        on_msg: impl Fn(FilterMsg) -> Msg + 'static + Copy,
        on_pause: Msg,
        on_clear: Msg,
        scroll_locked: bool,
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
        let count_color = if visible_count < total_count {
            accent
        } else {
            fg_dim2
        };

        let pause_label = if scroll_locked { "▶" } else { "||" };
        let pause_color = if scroll_locked { warn } else { fg_dim };

        let pause_btn = button(
            text(pause_label)
                .size(11)
                .color(pause_color)
                .font(iced::Font::MONOSPACE),
        )
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 6.0,
            right: 6.0,
        })
        .on_press(on_pause)
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        });

        let clear_btn = button(
            text("✕")
                .size(11)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE),
        )
        .padding(Padding {
            top: 3.0,
            bottom: 3.0,
            left: 6.0,
            right: 6.0,
        })
        .on_press(on_clear)
        .style(move |_, _| button::Style {
            background: None,
            border: Border::default(),
            ..Default::default()
        });

        container(
            row![
                search_input(
                    self.input.clone(),
                    "filter: db:shop coll:orders app:api slow",
                    move |t| on_msg(FilterMsg::TextChanged(t)),
                    on_msg(FilterMsg::TextSubmit),
                    &palette,
                ),
                kind_chips(
                    self.filter.kind,
                    move |k| on_msg(FilterMsg::KindSelected(k)),
                    &palette
                ),
                iced::widget::Space::new(Length::Fill, 0),
                text(count_str)
                    .size(11)
                    .color(count_color)
                    .font(iced::Font::MONOSPACE),
                pause_btn,
                clear_btn,
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding(Padding {
                top: 6.0,
                bottom: 6.0,
                left: 10.0,
                right: 6.0,
            }),
        )
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg1)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_scope_updates_filter_and_input() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), None);
        assert_eq!(fs.filter.db, Some("shop".into()));
        assert_eq!(fs.input, "db:shop");
    }

    #[test]
    fn set_scope_db_and_coll() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        assert_eq!(fs.input, "db:shop coll:orders");
    }

    #[test]
    fn set_scope_replaces_existing() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("old".into()), Some("x".into()));
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        assert_eq!(fs.input, "db:shop coll:orders");
    }

    #[test]
    fn set_scope_none_clears_fields() {
        let mut fs = FilterState::new();
        fs.set_scope(Some("shop".into()), Some("orders".into()));
        fs.set_scope(None, None);
        assert_eq!(fs.filter.db, None);
        assert_eq!(fs.filter.coll, None);
        assert_eq!(fs.input, "");
    }

    #[test]
    fn set_app_updates_filter_and_input() {
        let mut fs = FilterState::new();
        fs.set_app(Some("myapi".into()));
        assert_eq!(fs.filter.app, Some("myapi".into()));
        assert_eq!(fs.input, "app:myapi");
    }

    #[test]
    fn set_preset_slow_queries() {
        let mut fs = FilterState::new();
        fs.set_preset(Some(Preset::SlowQueries));
        assert_eq!(fs.filter.preset, Some(Preset::SlowQueries));
        assert_eq!(fs.input, "slow");
    }

    #[test]
    fn set_preset_collscan_only() {
        let mut fs = FilterState::new();
        fs.set_preset(Some(Preset::CollScanOnly));
        assert_eq!(fs.filter.preset, Some(Preset::CollScanOnly));
        assert_eq!(fs.input, "collscan");
    }

    #[test]
    fn set_preset_none_clears() {
        let mut fs = FilterState::new();
        fs.set_preset(Some(Preset::SlowQueries));
        fs.set_preset(None);
        assert_eq!(fs.filter.preset, None);
        assert_eq!(fs.input, "");
    }

    #[test]
    fn text_changed_preserves_kind() {
        let mut fs = FilterState::new();
        fs.filter.kind = KindFilter::Find;
        fs.update(FilterMsg::TextChanged("db:shop".into()));
        assert_eq!(fs.filter.kind, KindFilter::Find);
        assert_eq!(fs.filter.db, Some("shop".into()));
    }
}
