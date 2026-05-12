pub mod buckets;
pub mod density_lane;
pub mod filter;
pub mod table;

pub use buckets::Buckets;
pub use density_lane::density_lane;
pub use filter::{FilterMsg, FilterState};
pub use table::{table_header, table_view};

use crate::{
    data::{model::QueryEntry, types::QueryId},
    theme::{Density, Palette},
};
use iced::{
    widget::{column, lazy, scrollable},
    Border, Element, Length,
};

pub const FEED_SCROLL_ID: &str = "feed_table";

#[derive(Debug, Clone)]
pub enum FeedMsg {
    Filter(FilterMsg),
    SelectEntry(QueryId),
    TogglePause,
    ClearEntries,
    Scrolled(scrollable::Viewport),
}

pub struct FeedState {
    pub filter: FilterState,
    pub entries: Vec<QueryEntry>,
    pub selected: Option<QueryId>,
    pub buckets: Buckets,
    pub now_ms: u64,
    pub paused: bool,
    pub scroll_locked: bool,
    pub frozen_entries: Vec<QueryEntry>,
    pub pending_scroll_to: u32, // scroll_to(y=0) tasks in flight
    pub pending_scroll_by: u32, // scroll_by(dy) tasks in flight
    pub scroll_y: f32,          // latest real scroll position
    pub prev_scroll_y: f32,     // scroll position before that (direction detection)
}

impl FeedState {
    pub fn new() -> Self {
        Self {
            filter: FilterState::new(),
            entries: Vec::new(),
            selected: None,
            buckets: Buckets::new(500),
            now_ms: 0,
            paused: false,
            scroll_locked: false,
            frozen_entries: Vec::new(),
            pending_scroll_to: 0,
            pending_scroll_by: 0,
            scroll_y: 0.0,
            prev_scroll_y: 0.0,
        }
    }

    pub fn push_entry(&mut self, entry: QueryEntry) -> bool {
        if self.paused {
            return false;
        }
        self.now_ms = entry.t_ms.into_inner();
        self.buckets.push(&entry, self.now_ms);
        self.entries.insert(0, entry);
        self.entries.truncate(2000);
        true
    }

    pub fn update(&mut self, msg: FeedMsg) {
        match msg {
            FeedMsg::Filter(m) => self.filter.update(m),
            FeedMsg::SelectEntry(id) => {
                self.selected = Some(id);
                self.scroll_locked = true;
            }
            FeedMsg::TogglePause => {
                self.paused = !self.paused;
                if !self.paused {
                    self.scroll_locked = false;
                }
            }
            FeedMsg::ClearEntries => {
                self.entries.clear();
                self.frozen_entries.clear();
                self.buckets = Buckets::new(500);
                self.selected = None;
            }
            FeedMsg::Scrolled(vp) => {
                let y = vp.absolute_offset().y;
                // Programmatic scroll_to(y=0) in flight: ignore the y≈0 event it produces
                if y < 1.0 && self.pending_scroll_to > 0 {
                    self.pending_scroll_to = self.pending_scroll_to.saturating_sub(1);
                    return;
                }
                // Programmatic scroll_by(dy) in flight: ignore the y>0 event it produces.
                // Without this guard, a scroll_by issued while scroll_locked=true can land
                // after the user scrolls back to top, firing y=dy > threshold → re-locks.
                if y > 50.0 && self.pending_scroll_by > 0 {
                    self.pending_scroll_by = self.pending_scroll_by.saturating_sub(1);
                    return;
                }
                // 50px threshold: macOS momentum/trackpad drift stays well below this;
                // intentional scroll-down easily exceeds it.
                if y > 50.0 {
                    self.scroll_locked = true;
                } else if y < 1.0 && self.scroll_locked {
                    self.scroll_locked = false;
                }
                self.prev_scroll_y = self.scroll_y;
                self.scroll_y = y;
            }
        }
    }

    pub fn visible_entries(&self) -> Vec<&QueryEntry> {
        self.entries
            .iter()
            .filter(|e| self.filter.kind.matches(&e.op) && self.filter.expr.matches(e))
            .collect()
    }

    pub fn view<'a, Msg: Clone + 'static>(
        &'a self,
        on_msg: impl Fn(FeedMsg) -> Msg + 'static + Copy,
        palette: Palette,
        density: Density,
    ) -> Element<'a, Msg> {
        let fs = density.fs_base();
        let max_total = self.buckets.max_total();
        let bucket_data = &self.buckets.data;
        let palette_copy = palette;
        let bg = palette.bg;
        let fg_dim2 = palette.fg_dim2;

        let take_n = if self.scroll_locked { 500 } else { 150 };
        let visible: Vec<QueryEntry> = self
            .visible_entries()
            .into_iter()
            .take(take_n)
            .cloned()
            .collect();
        let selected = self.selected;

        let total_count = self.entries.len();
        let visible_count = visible.len();
        let first_id = visible.first().map(|e| e.id);
        let dep = (visible_count, first_id, selected);

        let table_inner = lazy(dep, move |_| {
            let refs: Vec<&QueryEntry> = visible.iter().collect();
            table_view(
                &refs,
                selected,
                move |id| on_msg(FeedMsg::SelectEntry(id)),
                &palette,
                fs,
            )
        });

        let header = table_header::<Msg>(&palette, fs);

        let table = scrollable(table_inner)
            .id(scrollable::Id::new(FEED_SCROLL_ID))
            .on_scroll(move |vp| on_msg(FeedMsg::Scrolled(vp)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme, status| {
                let a = match status {
                    scrollable::Status::Active => 0.0,
                    scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => 1.0,
                };
                scrollable::Style {
                    container: iced::widget::container::Style::default(),
                    vertical_rail: scrollable::Rail {
                        background: Some(iced::Background::Color(iced::Color { a: a * 0.5, ..bg })),
                        border: Border::default(),
                        scroller: scrollable::Scroller {
                            color: iced::Color { a, ..fg_dim2 },
                            border: Border {
                                radius: 4.0.into(),
                                ..Default::default()
                            },
                        },
                    },
                    horizontal_rail: scrollable::Rail {
                        background: None,
                        border: Border::default(),
                        scroller: scrollable::Scroller {
                            color: iced::Color { a: 0.0, ..fg_dim2 },
                            border: Border::default(),
                        },
                    },
                    gap: None,
                }
            });

        column![
            self.filter.view(
                move |m| on_msg(FeedMsg::Filter(m)),
                on_msg(FeedMsg::TogglePause),
                on_msg(FeedMsg::ClearEntries),
                self.paused,
                visible_count,
                total_count,
                palette,
            ),
            density_lane(bucket_data, max_total, palette_copy),
            header,
            table,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
