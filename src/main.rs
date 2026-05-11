mod data;
mod theme;
mod ui;

use iced::{widget::{column, container, row, scrollable}, Element, Length, Subscription, Task};
use data::{mock::MockSource, model::QueryEntry, source::DataSource, types::QueryId};
use theme::{Density, Dock, Palette, Theme};
use ui::{
    feed::{FeedMsg, FeedState, FEED_SCROLL_ID},
    inspector::{InspectorMsg, InspectorState},
    sidebar::{SidebarMsg, SidebarState},
    statusbar::{statusbar, StatusInfo},
    topbar::{conn_bar::ConnInfo, topbar, MenuMsg},
};

#[derive(Debug, Clone)]
enum Message {
    QueriesReceived(Vec<QueryEntry>),
    Feed(FeedMsg),
    Inspector(InspectorMsg),
    Sidebar(SidebarMsg),
    ToggleTheme,
    ToggleDensity,
    ToggleCapture,
    Menu(MenuMsg),
}

struct App {
    feed: FeedState,
    inspector: InspectorState,
    sidebar: SidebarState,
    theme: Theme,
    density: Density,
    capturing: bool,
    tx: Option<tokio::sync::mpsc::Sender<QueryEntry>>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app = Self {
            feed: FeedState::new(),
            inspector: InspectorState::new(),
            sidebar: SidebarState::new(),
            theme: Theme::Dark,
            density: Density::Compact,
            capturing: true,
            tx: None,
        };
        (app, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::QueriesReceived(entries) => {
                self.sidebar.register_entries(&entries);
                let mut added = 0usize;
                for entry in entries {
                    if self.feed.push_entry(entry) { added += 1; }
                }
                if added == 0 { return Task::none(); }
                if self.feed.scroll_locked {
                    let dy = added as f32 * self.density.row_height();
                    return scrollable::scroll_by(
                        scrollable::Id::new(FEED_SCROLL_ID),
                        scrollable::AbsoluteOffset { x: 0.0, y: dy },
                    );
                } else if !self.feed.paused {
                    self.feed.pending_scroll_to += 1;
                    return scrollable::scroll_to(
                        scrollable::Id::new(FEED_SCROLL_ID),
                        scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                    );
                }
            }
            Message::Feed(m) => {
                self.feed.update(m);
            }
            Message::Inspector(m) => {
                self.inspector.update(m);
            }
            Message::Sidebar(m) => {
                self.sidebar.update(m);
            }
            Message::ToggleTheme => {
                self.theme = self.theme.toggle();
            }
            Message::ToggleDensity => {
                self.density = self.density.toggle();
            }
            Message::ToggleCapture => {
                self.capturing = !self.capturing;
            }
            Message::Menu(_) => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let palette = self.theme.palette();
        let fs = self.density.fs_base();
        let bg = palette.bg;
        let border_color = palette.border;

        let conn = ConnInfo {
            host: "localhost:27017".into(),
            uri: "mongoscope://localhost".into(),
            rs_name: Some("rs0".into()),
            connected: true,
        };

        let top = topbar(
            &conn,
            self.capturing,
            |m| Message::Menu(m),
            Message::ToggleCapture,
            &palette,
        );

        let ops_per_sec = if self.feed.entries.len() >= 10 { 12.5f32 } else { 0.0 };
        let slow_count = self.feed.entries.iter().filter(|e| e.slow).count();

        let status = statusbar(
            &StatusInfo {
                ops_per_sec,
                query_count: self.feed.entries.len(),
                slow_count,
                theme_label: self.theme.label(),
                density_label: self.density.label(),
            },
            Message::ToggleTheme,
            Message::ToggleDensity,
            &palette,
        );

        let sidebar_el = self.sidebar.view(|m| Message::Sidebar(m), &palette);

        let feed_el = self.feed.view(|m| Message::Feed(m), palette, self.density);

        let selected_entry = self.feed.selected
            .and_then(|id| self.feed.entries.iter().find(|e| e.id == id));

        let inspector_el = self.inspector.view(
            selected_entry,
            |m| Message::Inspector(m),
            palette,
            fs,
        );

        let main_pane = column![
            container(feed_el)
                .width(Length::Fill)
                .height(Length::Fill),
            container(inspector_el)
                .width(Length::Fill)
                .height(340),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        let body = row![
            sidebar_el,
            main_pane,
        ]
        .spacing(0)
        .height(Length::Fill);

        container(
            column![top, body, status].spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| iced::widget::container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border { color: border_color, width: 0.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(|| {
            iced::stream::channel(256, |mut output| async move {
                use iced::futures::SinkExt;
                let (tx, mut rx) = tokio::sync::mpsc::channel(256);
                Box::new(MockSource).start(tx);
                loop {
                    // Collect up to 8 entries or wait 100ms, whichever comes first
                    let first = rx.recv().await;
                    let Some(first) = first else { break };
                    let mut batch = vec![first];
                    let deadline = tokio::time::Instant::now() + std::time::Duration::from_millis(300);
                    loop {
                        match tokio::time::timeout_at(deadline, rx.recv()).await {
                            Ok(Some(e)) => {
                                batch.push(e);
                                if batch.len() >= 8 { break; }
                            }
                            _ => break,
                        }
                    }
                    let _ = output.send(Message::QueriesReceived(batch)).await;
                }
            })
        })
    }
}

fn main() -> iced::Result {
    iced::application("Mongoscope", App::update, App::view)
        .subscription(App::subscription)
        .window(iced::window::Settings {
            size: iced::Size::new(1400.0, 900.0),
            ..Default::default()
        })
        .run_with(App::new)
}
