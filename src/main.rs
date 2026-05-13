mod data;
mod theme;
mod ui;

use data::{mock::MockSource, model::QueryEntry, source::DataSource};
use iced::{
    mouse,
    widget::{column, container, mouse_area, row, scrollable, stack},
    Element, Length, Subscription, Task,
};
use std::time::Duration;
use theme::{Density, Theme};
use ui::{
    dialog::dialog_view,
    feed::{FeedMsg, FEED_SCROLL_ID},
    inspector::{
        tabs::{ExplainMsg, ExplainState},
        InspectorMsg, InspectorState,
    },
    mcp_panel::{McpMsg, McpPanelState, McpServerState},
    sidebar::{connections::ConnectionsMsg, filters::FilterPanelMsg, SidebarMsg, SidebarState},
    statusbar::{statusbar, StatusInfo},
    topbar::{conn_bar::ConnInfo, topbar, MenuMsg},
};

#[derive(Debug, Clone, Default)]
pub enum InspectorPanel {
    #[default]
    Closed,
    Open {
        height: f32,
        drag_start: Option<(f32, f32)>,
    },
    Maximized {
        prev_height: f32,
    },
}

#[derive(Debug, Clone)]
enum Message {
    QueriesReceived(usize, Vec<QueryEntry>),
    Feed(FeedMsg),
    Inspector(InspectorMsg),
    Sidebar(SidebarMsg),
    ToggleTheme,
    ToggleDensity,
    ToggleCapture,
    // TODO: remove when real backend is wired up — currently all mock data
    #[allow(dead_code)]
    Menu(MenuMsg),
    SidebarResizeStart,
    SidebarResizeMove(f32),
    SidebarResizeEnd,
    InspectorResizeStart,
    InspectorResizeMove(f32),
    InspectorResizeEnd,
    Mcp(McpMsg),
}

struct App {
    inspector: InspectorState,
    sidebar: SidebarState,
    theme: Theme,
    density: Density,
    sidebar_width: f32,
    sidebar_dragging: bool,
    inspector_panel: InspectorPanel,
    mcp_panel: McpPanelState,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self {
            inspector: InspectorState::new(),
            sidebar: SidebarState::new(),
            theme: Theme::Dark,
            density: Density::Compact,
            sidebar_width: 220.0,
            sidebar_dragging: false,
            inspector_panel: InspectorPanel::default(),
            mcp_panel: McpPanelState::new(),
        };
        if app.sidebar.connections.is_empty() {
            app.sidebar.dialog = Some(ui::dialog::ConnectionDialogState::new());
        }
        (app, Task::none())
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::QueriesReceived(conn_id, entries) => {
                let active_id = self.sidebar.active_id;
                if let Some(conn) = self
                    .sidebar
                    .connections
                    .iter_mut()
                    .find(|c| c.item.id == conn_id)
                {
                    if !conn.capturing {
                        return Task::none();
                    }
                    conn.register_entries(&entries);
                    let mut added = 0usize;
                    for entry in entries {
                        if conn.feed.push_entry(entry) {
                            added += 1;
                        }
                    }
                    let is_active = active_id == Some(conn_id);
                    if added == 0 || !is_active {
                        return Task::none();
                    }
                    if conn.feed.scroll_locked {
                        let scrolling_up = conn.feed.scroll_y < conn.feed.prev_scroll_y;
                        if !scrolling_up {
                            let dy = added as f32 * self.density.row_height();
                            conn.feed.pending_scroll_by += 1;
                            return scrollable::scroll_by(
                                scrollable::Id::new(FEED_SCROLL_ID),
                                scrollable::AbsoluteOffset { x: 0.0, y: dy },
                            );
                        }
                    } else {
                        conn.feed.pending_scroll_to += 1;
                        return scrollable::scroll_to(
                            scrollable::Id::new(FEED_SCROLL_ID),
                            scrollable::AbsoluteOffset { x: 0.0, y: 0.0 },
                        );
                    }
                }
                Task::none()
            }
            Message::Feed(m) => {
                let prev_selected = self.sidebar.active().and_then(|c| c.feed.selected);
                if let Some(conn) = self.sidebar.active_mut() {
                    conn.feed.update(m);
                }
                let new_selected = self.sidebar.active().and_then(|c| c.feed.selected);
                if new_selected != prev_selected {
                    self.inspector.explain = ExplainState::default();
                    if new_selected.is_some()
                        && matches!(self.inspector_panel, InspectorPanel::Closed)
                    {
                        self.inspector_panel = InspectorPanel::Open {
                            height: 300.0,
                            drag_start: None,
                        };
                        // Inspector opening shrinks the feed viewport, which fires a
                        // spurious Scrolled(y≈0). Absorb it so scroll_locked is preserved.
                        if let Some(conn) = self.sidebar.active_mut() {
                            conn.feed.pending_layout_reflow += 1;
                        }
                    }
                }
                Task::none()
            }
            Message::Inspector(InspectorMsg::Close) => {
                self.inspector_panel = InspectorPanel::Closed;
                Task::none()
            }
            Message::Inspector(InspectorMsg::ToggleMaximize) => {
                self.inspector_panel = match &self.inspector_panel {
                    InspectorPanel::Open { height, .. } => InspectorPanel::Maximized {
                        prev_height: *height,
                    },
                    InspectorPanel::Maximized { prev_height } => InspectorPanel::Open {
                        height: *prev_height,
                        drag_start: None,
                    },
                    InspectorPanel::Closed => InspectorPanel::Closed,
                };
                Task::none()
            }
            Message::Inspector(InspectorMsg::Explain(ExplainMsg::CopyIndex)) => {
                if let Some(entry) = self.sidebar.active().and_then(|c| {
                    c.feed
                        .selected
                        .and_then(|id| c.feed.entries.iter().find(|e| e.id == id))
                }) {
                    let first_key = entry
                        .filter
                        .as_ref()
                        .and_then(|f| f.keys().next().cloned())
                        .unwrap_or_else(|| "field".to_string());
                    let cmd = format!(
                        "db.{}.createIndex({{ {}: 1 }})",
                        entry.coll.as_str(),
                        first_key
                    );
                    return iced::clipboard::write::<Message>(cmd);
                }
                Task::none()
            }
            Message::Inspector(m) => {
                self.inspector.update(m);
                Task::none()
            }
            Message::Sidebar(ref m) => {
                match m {
                    SidebarMsg::Connections(ConnectionsMsg::DialogConnect) => {
                        self.sidebar.update(m.clone());
                        return Task::perform(
                            async {
                                tokio::time::sleep(Duration::from_millis(800)).await;
                                Result::<u16, String>::Ok(27117)
                            },
                            |r| {
                                Message::Sidebar(SidebarMsg::Connections(
                                    ConnectionsMsg::DialogConnectResult(r),
                                ))
                            },
                        );
                    }
                    SidebarMsg::Connections(ConnectionsMsg::DialogCopyUri) => {
                        if let Some(d) = &self.sidebar.dialog {
                            let uri = format!(
                                "mongodb://localhost:{}/?directConnection=true",
                                d.proxy_port
                            );
                            return iced::clipboard::write::<Message>(uri);
                        }
                    }
                    _ => {}
                }
                self.sidebar.update(m.clone());
                if let SidebarMsg::Databases(_) = m {
                    let (db, coll) = (self.sidebar.active_db(), self.sidebar.active_coll());
                    if let Some(conn) = self.sidebar.active_mut() {
                        conn.feed.filter.set_scope(db, coll);
                    }
                }
                if let SidebarMsg::Clients(_) = m {
                    let app = self.sidebar.active_client();
                    if let Some(conn) = self.sidebar.active_mut() {
                        conn.feed.filter.set_app(app);
                    }
                }
                if let SidebarMsg::Filters(FilterPanelMsg::Toggle(preset)) = &m {
                    let preset = *preset;
                    if let Some(conn) = self.sidebar.active_mut() {
                        let current = conn.feed.filter.filter.preset;
                        let next = if current == Some(preset) {
                            None
                        } else {
                            Some(preset)
                        };
                        conn.feed.filter.set_preset(next);
                    }
                }
                Task::none()
            }
            Message::ToggleTheme => {
                self.theme = self.theme.toggle();
                Task::none()
            }
            Message::ToggleDensity => {
                self.density = self.density.toggle();
                Task::none()
            }
            Message::ToggleCapture => {
                if let Some(conn) = self.sidebar.active_mut() {
                    conn.capturing = !conn.capturing;
                }
                Task::none()
            }
            Message::Menu(_) => Task::none(),
            Message::SidebarResizeStart => {
                self.sidebar_dragging = true;
                Task::none()
            }
            Message::SidebarResizeMove(x) => {
                if self.sidebar_dragging {
                    self.sidebar_width = x.clamp(120.0, 400.0);
                }
                Task::none()
            }
            Message::SidebarResizeEnd => {
                self.sidebar_dragging = false;
                Task::none()
            }
            Message::InspectorResizeStart => {
                // Set drag_start to Some so the subscription activates immediately.
                // NAN y is a sentinel meaning "first CursorMoved not yet seen".
                if let InspectorPanel::Open { height, drag_start } = &mut self.inspector_panel {
                    *drag_start = Some((f32::NAN, *height));
                }
                Task::none()
            }
            Message::InspectorResizeMove(y) => {
                if let InspectorPanel::Open { height, drag_start } = &mut self.inspector_panel {
                    if let Some((start_y, start_h)) = drag_start {
                        if start_y.is_nan() {
                            // First move after ResizeStart: record baseline.
                            *drag_start = Some((y, *start_h));
                        } else {
                            *height = (*start_h + (*start_y - y)).clamp(120.0, 600.0);
                        }
                    }
                }
                Task::none()
            }
            Message::InspectorResizeEnd => {
                if let InspectorPanel::Open { drag_start, .. } = &mut self.inspector_panel {
                    *drag_start = None;
                }
                Task::none()
            }
            Message::Mcp(m) => match m {
                McpMsg::Toggle => {
                    self.mcp_panel.toggle();
                    Task::none()
                }
                McpMsg::StartStop => {
                    if self.mcp_panel.begin_start() {
                        Task::perform(
                            async { tokio::time::sleep(Duration::from_millis(800)).await },
                            |_| Message::Mcp(McpMsg::Started),
                        )
                    } else {
                        self.mcp_panel.stop();
                        Task::none()
                    }
                }
                McpMsg::Started => {
                    self.mcp_panel.on_started(ui::mcp_panel::MOCK_MCP_PORT);
                    Task::none()
                }
                McpMsg::CopyConfig => {
                    if let McpServerState::Running { port } = self.mcp_panel.server {
                        let snippet = format!(
                            "\"mongoscope\": {{\n  \"url\": \"http://localhost:{port}/mcp\"\n}}"
                        );
                        iced::clipboard::write::<Message>(snippet)
                    } else {
                        Task::none()
                    }
                }
                McpMsg::Noop => Task::none(),
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let palette = self.theme.palette();
        let fs = self.density.fs_base();
        let bg = palette.bg;
        let border_color = palette.border;

        let capturing = self.sidebar.active().map(|c| c.capturing).unwrap_or(false);

        let conn = ConnInfo {
            host: "localhost:27017".into(),
            uri: "mongoscope://localhost".into(),
            rs_name: Some("rs0".into()),
            connected: true,
        };

        let top = topbar(
            &conn,
            capturing,
            Message::Menu,
            Message::ToggleCapture,
            &self.mcp_panel.server,
            Message::Mcp(McpMsg::Toggle),
            &palette,
        );

        let (ops_per_sec, query_count, slow_count) = self
            .sidebar
            .active()
            .map(|c| {
                let ops = if c.feed.entries.len() >= 10 {
                    12.5f32
                } else {
                    0.0
                };
                let slow = c.feed.entries.iter().filter(|e| e.slow).count();
                (ops, c.feed.entries.len(), slow)
            })
            .unwrap_or((0.0, 0, 0));

        let status = statusbar(
            &StatusInfo {
                ops_per_sec,
                query_count,
                slow_count,
                theme_label: self.theme.label(),
                density_label: self.density.label(),
            },
            Message::ToggleTheme,
            Message::ToggleDensity,
            &palette,
        );

        let active_preset = self
            .sidebar
            .active()
            .and_then(|c| c.feed.filter.active_preset());
        let sidebar_el = self.sidebar.view(
            Message::Sidebar,
            &palette,
            self.sidebar_width,
            active_preset,
        );

        let resize_handle = mouse_area(
            container(iced::widget::Space::new(4.0, Length::Fill)).style(move |_| {
                container::Style {
                    background: Some(iced::Background::Color(border_color)),
                    ..Default::default()
                }
            }),
        )
        .on_press(Message::SidebarResizeStart)
        .on_release(Message::SidebarResizeEnd)
        .interaction(mouse::Interaction::ResizingHorizontally);

        let feed_el: Element<Message> = if let Some(conn) = self.sidebar.active() {
            conn.feed.view(Message::Feed, palette, self.density)
        } else {
            iced::widget::Space::new(Length::Fill, Length::Fill).into()
        };

        let selected_entry = self.sidebar.active().and_then(|c| {
            c.feed
                .selected
                .and_then(|id| c.feed.entries.iter().find(|e| e.id == id))
        });

        let maximized = matches!(self.inspector_panel, InspectorPanel::Maximized { .. });
        let active_conn = self.sidebar.active();
        let cluster_label = active_conn.map(|c| c.item.label.as_str()).unwrap_or("—");
        let shell_version = active_conn
            .map(|c| c.item.shell_version.as_str())
            .unwrap_or("mongosh 2.4.0");
        let inspector_el = self.inspector.view(
            selected_entry,
            maximized,
            Message::Inspector,
            palette,
            fs,
            cluster_label,
            shell_version,
        );

        let main_pane: Element<Message> = match &self.inspector_panel {
            InspectorPanel::Closed => container(feed_el)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            InspectorPanel::Open { height, .. } => {
                let resize_handle_h = mouse_area(
                    container(iced::widget::Space::new(Length::Fill, 4.0)).style(move |_| {
                        container::Style {
                            background: Some(iced::Background::Color(border_color)),
                            ..Default::default()
                        }
                    }),
                )
                .on_press(Message::InspectorResizeStart)
                .on_release(Message::InspectorResizeEnd)
                .interaction(mouse::Interaction::ResizingVertically);

                column![
                    container(feed_el).width(Length::Fill).height(Length::Fill),
                    resize_handle_h,
                    container(inspector_el).width(Length::Fill).height(*height),
                ]
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            InspectorPanel::Maximized { .. } => container(inspector_el)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
        };

        let body_row = row![sidebar_el, resize_handle, main_pane,]
            .spacing(0)
            .height(Length::Fill);

        let body: Element<Message> = if self.mcp_panel.open {
            iced::widget::stack![
                body_row,
                ui::mcp_panel::overlay_view(&self.mcp_panel, Message::Mcp, &palette,),
            ]
            .into()
        } else {
            body_row.into()
        };

        let base: Element<Message> = container(column![top, body, status].spacing(0))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| iced::widget::container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    color: border_color,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into();

        if let Some(dialog_state) = &self.sidebar.dialog {
            let dialog_palette = self.theme.palette();
            stack![
                base,
                dialog_view(
                    dialog_state,
                    |m| Message::Sidebar(SidebarMsg::Connections(m)),
                    &dialog_palette,
                )
            ]
            .into()
        } else {
            base
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let data_subs: Vec<Subscription<Message>> = self
            .sidebar
            .connections
            .iter()
            .filter(|c| c.item.live)
            .map(|c| {
                let id = c.item.id;
                Subscription::run_with_id(
                    id,
                    iced::stream::channel(256, move |mut output| async move {
                        use iced::futures::SinkExt;
                        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
                        Box::new(MockSource).start(tx);
                        loop {
                            let first = rx.recv().await;
                            let Some(first) = first else { break };
                            let mut batch = vec![first];
                            let deadline =
                                tokio::time::Instant::now() + std::time::Duration::from_millis(300);
                            while let Ok(Some(e)) =
                                tokio::time::timeout_at(deadline, rx.recv()).await
                            {
                                batch.push(e);
                                if batch.len() >= 8 {
                                    break;
                                }
                            }
                            let _ = output.send(Message::QueriesReceived(id, batch)).await;
                        }
                    }),
                )
            })
            .collect();

        let inspector_dragging = matches!(
            self.inspector_panel,
            InspectorPanel::Open {
                drag_start: Some(_),
                ..
            }
        );

        let mut all = data_subs;

        if self.sidebar_dragging {
            let drag_sub = iced::event::listen_with(|event, _status, _window| match event {
                iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    Some(Message::SidebarResizeMove(position.x))
                }
                iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Message::SidebarResizeEnd)
                }
                _ => None,
            });
            all.push(drag_sub);
        }

        if inspector_dragging {
            let drag_sub = iced::event::listen_with(|event, _status, _window| match event {
                iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    Some(Message::InspectorResizeMove(position.y))
                }
                iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Message::InspectorResizeEnd)
                }
                _ => None,
            });
            all.push(drag_sub);
        }

        Subscription::batch(all)
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
