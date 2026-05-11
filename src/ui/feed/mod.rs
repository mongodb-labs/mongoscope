pub mod buckets;
pub mod density_lane;
pub mod filter;
pub mod table;

pub use buckets::Buckets;
pub use density_lane::density_lane;
pub use filter::{FilterMsg, FilterState};
pub use table::table_view;

use iced::{widget::{column, lazy, scrollable}, Element, Length};
use crate::{
    data::{model::QueryEntry, types::QueryId},
    theme::{Density, Palette},
};

pub const FEED_SCROLL_ID: &str = "feed_table";

#[derive(Debug, Clone)]
pub enum FeedMsg {
    Filter(FilterMsg),
    SelectEntry(QueryId),
    TogglePause,
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
    pub last_scroll_y: f32,
    pub pending_scroll_to: u32,  // scroll_to(y=0) tasks in flight
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
            last_scroll_y: 0.0,
            pending_scroll_to: 0,
        }
    }

    pub fn push_entry(&mut self, entry: QueryEntry) -> bool {
        if self.paused { return false; }
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
            FeedMsg::Scrolled(vp) => {
                let y = vp.absolute_offset().y;
                // If scroll_to(y=0) was in flight and this is y≈0, it's programmatic — ignore
                if y < 1.0 && self.pending_scroll_to > 0 {
                    self.pending_scroll_to = self.pending_scroll_to.saturating_sub(1);
                    return;
                }
                if y > 5.0 {
                    self.scroll_locked = true;
                } else if y < 1.0 && self.last_scroll_y > 5.0 {
                    self.scroll_locked = false;
                }
                self.last_scroll_y = y;
            }
        }
    }

    pub fn visible_entries(&self) -> Vec<&QueryEntry> {
        self.entries.iter()
            .filter(|e| {
                self.filter.kind.matches(&e.op) && self.filter.expr.matches(e)
            })
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

        let take_n = if self.scroll_locked { 500 } else { 150 };
        let visible: Vec<QueryEntry> = self.visible_entries()
            .into_iter().take(take_n).cloned().collect();
        let selected = self.selected;

        let total_count = self.entries.len();
        let visible_count = visible.len();
        let first_id = visible.first().map(|e| e.id);
        let dep = (visible_count, first_id, selected);

        let table_inner = lazy(dep, move |_| {
            let refs: Vec<&QueryEntry> = visible.iter().collect();
            table_view(&refs, selected, move |id| on_msg(FeedMsg::SelectEntry(id)), &palette, fs)
        });

        let table = scrollable(table_inner)
            .id(scrollable::Id::new(FEED_SCROLL_ID))
            .on_scroll(move |vp| on_msg(FeedMsg::Scrolled(vp)))
            .width(Length::Fill)
            .height(Length::Fill);

        column![
            self.filter.view(
                move |m| on_msg(FeedMsg::Filter(m)),
                on_msg(FeedMsg::TogglePause),
                self.paused,
                visible_count,
                total_count,
                palette,
            ),
            density_lane(bucket_data, max_total, palette_copy),
            table,
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
