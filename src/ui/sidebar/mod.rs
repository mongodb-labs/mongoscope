pub mod clients;
pub mod collections;
pub mod connection_state;
pub mod connections;
pub mod databases;
pub mod filters;
pub use connection_state::ConnectionState;

pub use clients::{clients_panel, ClientsMsg};
pub use connections::{connections_panel, ConnectionItem, ConnectionsMsg};
pub use databases::{apply_toggle_collection, apply_toggle_db, databases_panel, DatabasesMsg};
pub use filters::{filters_panel, FilterPanelMsg};

use crate::{theme::Palette, ui::dialog::ConnectionDialogState};
use iced::{
    widget::{column, container, row, scrollable, text},
    Border, Element, Length, Padding,
};

#[derive(Debug, Clone)]
pub enum SidebarMsg {
    Connections(ConnectionsMsg),
    Databases(DatabasesMsg),
    Clients(ClientsMsg),
    Filters(FilterPanelMsg),
}

pub struct SidebarState {
    pub connections: Vec<ConnectionState>,
    pub active_id: Option<usize>,
    pub dialog: Option<ConnectionDialogState>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            connections: vec![],
            active_id: None,
            dialog: None,
        }
    }

    pub fn active(&self) -> Option<&ConnectionState> {
        let id = self.active_id?;
        self.connections.iter().find(|c| c.item.id == id)
    }

    pub fn active_mut(&mut self) -> Option<&mut ConnectionState> {
        let id = self.active_id?;
        self.connections.iter_mut().find(|c| c.item.id == id)
    }

    pub fn active_db(&self) -> Option<String> {
        self.active()?
            .databases
            .iter()
            .find(|d| d.active)
            .map(|d| d.name.clone())
    }

    pub fn active_coll(&self) -> Option<String> {
        let conn = self.active()?;
        conn.databases
            .iter()
            .find(|d| d.active)
            .and_then(|d| d.collections.iter().find(|c| c.active))
            .map(|c| c.name.clone())
    }

    pub fn active_client(&self) -> Option<String> {
        self.active()?
            .clients
            .iter()
            .find(|c| c.active)
            .map(|c| c.name.clone())
    }

    pub fn update(&mut self, msg: SidebarMsg) {
        match msg {
            SidebarMsg::Connections(m) => match m {
                ConnectionsMsg::Select(id) => {
                    for c in &mut self.connections {
                        c.item.active = c.item.id == id;
                    }
                    self.active_id = Some(id);
                }
                ConnectionsMsg::Add => {
                    if self.dialog.is_none() {
                        self.dialog = Some(ConnectionDialogState::new());
                    }
                }
                ConnectionsMsg::DialogUriChanged(s) => {
                    if let Some(d) = &mut self.dialog {
                        d.uri = s;
                        d.error = None;
                    }
                }
                ConnectionsMsg::DialogNameChanged(s) => {
                    if let Some(d) = &mut self.dialog {
                        d.name = s;
                    }
                }
                ConnectionsMsg::DialogColorChanged(c) => {
                    if let Some(d) = &mut self.dialog {
                        d.color = c;
                    }
                }
                ConnectionsMsg::DialogConnect => {
                    if let Some(d) = &mut self.dialog {
                        d.step = crate::ui::dialog::DialogStep::Step1 { connecting: true };
                        d.error = None;
                    }
                }
                ConnectionsMsg::DialogConnectResult(Ok(port)) => {
                    if let Some(d) = &mut self.dialog {
                        d.proxy_port = port;
                        d.step = crate::ui::dialog::DialogStep::Step2;
                    }
                }
                ConnectionsMsg::DialogConnectResult(Err(e)) => {
                    if let Some(d) = &mut self.dialog {
                        d.step = crate::ui::dialog::DialogStep::Step1 { connecting: false };
                        d.error = Some(e);
                    }
                }
                ConnectionsMsg::DialogBack => {
                    if let Some(d) = &mut self.dialog {
                        d.step = crate::ui::dialog::DialogStep::Step1 { connecting: false };
                    }
                }
                ConnectionsMsg::DialogDone => {
                    if let Some(d) = &self.dialog {
                        let next_id = self
                            .connections
                            .iter()
                            .map(|c| c.item.id)
                            .max()
                            .unwrap_or(0)
                            + 1;
                        let label = if d.name.is_empty() {
                            d.uri
                                .trim_start_matches("mongodb://")
                                .trim_start_matches("mongodb+srv://")
                                .split('/')
                                .next()
                                .unwrap_or("connection")
                                .to_owned()
                        } else {
                            d.name.clone()
                        };
                        let topology = format!("direct · proxy :{}", d.proxy_port);
                        let item = ConnectionItem {
                            id: next_id,
                            label,
                            topology,
                            uri: d.uri.clone(),
                            proxy_port: d.proxy_port,
                            color: d.color,
                            active: true,
                            live: true,
                            shell_version: "mongosh 2.4.0".into(),
                        };
                        for c in &mut self.connections {
                            c.item.active = false;
                        }
                        self.connections.push(ConnectionState::new(item));
                        self.active_id = Some(next_id);
                    }
                    self.dialog = None;
                }
                ConnectionsMsg::DialogCancel => {
                    self.dialog = None;
                }
                ConnectionsMsg::DialogCopyUri => {
                    // handled in App::update to produce clipboard Task
                }
                ConnectionsMsg::DialogNoop => {}
            },
            SidebarMsg::Databases(m) => {
                if let Some(conn) = self.active_mut() {
                    match m {
                        DatabasesMsg::ToggleDb(name) => apply_toggle_db(&mut conn.databases, &name),
                        DatabasesMsg::ToggleCollection(db, coll) => {
                            apply_toggle_collection(&mut conn.databases, &db, &coll)
                        }
                    }
                }
            }
            SidebarMsg::Clients(m) => {
                if let Some(conn) = self.active_mut() {
                    match m {
                        ClientsMsg::Toggle(name) => {
                            let was_active = conn
                                .clients
                                .iter()
                                .find(|c| c.name == name)
                                .map(|c| c.active)
                                .unwrap_or(false);
                            for c in &mut conn.clients {
                                c.active = false;
                            }
                            if !was_active {
                                if let Some(c) = conn.clients.iter_mut().find(|c| c.name == name) {
                                    c.active = true;
                                }
                            }
                        }
                    }
                }
            }
            SidebarMsg::Filters(_) => {
                // Preset toggle handled in App::update before reaching here.
            }
        }
    }

    pub fn view<Msg: Clone + 'static>(
        &self,
        on_msg: impl Fn(SidebarMsg) -> Msg + 'static + Copy,
        palette: &Palette,
        width: f32,
        active_preset: Option<crate::ui::feed::filter::parser::Preset>,
    ) -> Element<'static, Msg> {
        let bg = palette.bg;
        let bg1 = palette.bg1;
        let border_color = palette.border;
        let fg_dim2 = palette.fg_dim2;
        let fg_dim = palette.fg_dim;

        let section_header = move |label: String, right: Option<String>| -> Element<'static, Msg> {
            let label_el = text(label)
                .size(9)
                .color(fg_dim2)
                .font(iced::Font::MONOSPACE);

            let inner: Element<Msg> = if let Some(r) = right {
                row![
                    label_el,
                    iced::widget::Space::new(Length::Fill, 0),
                    text(r).size(9).color(fg_dim).font(iced::Font::MONOSPACE),
                ]
                .into()
            } else {
                label_el.into()
            };

            container(inner)
                .padding(Padding {
                    top: 8.0,
                    bottom: 3.0,
                    left: 8.0,
                    right: 8.0,
                })
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg1)),
                    border: Border {
                        color: border_color,
                        width: 0.0,
                        radius: 0.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        };

        let (dbs, clients) = self
            .active()
            .map(|c| (c.databases.as_slice(), c.clients.as_slice()))
            .unwrap_or((&[], &[]));

        let db_count = dbs.len();

        let conn_items: Vec<ConnectionItem> =
            self.connections.iter().map(|c| c.item.clone()).collect();

        let content = column![
            section_header("CONNECTIONS".into(), None),
            connections_panel(
                &conn_items,
                self.dialog.is_some(),
                move |m| on_msg(SidebarMsg::Connections(m)),
                palette,
            ),
            section_header("DATABASES".into(), Some(format!("{} dbs", db_count))),
            databases_panel(dbs, move |m| on_msg(SidebarMsg::Databases(m)), palette,),
            section_header("CLIENTS".into(), None),
            clients_panel(clients, move |m| on_msg(SidebarMsg::Clients(m)), palette,),
            section_header("FILTERS".into(), None),
            filters_panel(
                active_preset,
                move |m| on_msg(SidebarMsg::Filters(m)),
                *palette,
            ),
        ]
        .spacing(0)
        .width(Length::Fill);

        container(
            scrollable(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_theme, status| {
                    let a = match status {
                        scrollable::Status::Active => 0.0,
                        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => {
                            1.0
                        }
                    };
                    scrollable::Style {
                        container: iced::widget::container::Style::default(),
                        vertical_rail: scrollable::Rail {
                            background: Some(iced::Background::Color(iced::Color {
                                a: a * 0.5,
                                ..bg
                            })),
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
                }),
        )
        .width(width)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
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
    use crate::ui::sidebar::connections::{ConnectionColor, ConnectionItem};

    fn make_item(id: usize) -> ConnectionItem {
        ConnectionItem {
            id,
            label: "test".into(),
            topology: "direct".into(),
            uri: "mongodb://localhost:27017/".into(),
            proxy_port: 27117,
            color: ConnectionColor::None,
            active: false,
            live: true,
            shell_version: "mongosh 2.4.0".into(),
        }
    }

    impl SidebarState {
        fn default_for_test() -> Self {
            let mut s = SidebarState::new();
            s.connections.push(ConnectionState::new(make_item(0)));
            s.active_id = Some(0);
            s
        }
    }

    #[test]
    fn active_returns_none_when_empty() {
        let s = SidebarState::new();
        assert!(s.active().is_none());
    }

    #[test]
    fn active_returns_correct_connection() {
        let mut s = SidebarState::new();
        s.connections.push(ConnectionState::new(make_item(1)));
        s.connections.push(ConnectionState::new(make_item(2)));
        s.active_id = Some(2);
        assert_eq!(s.active().unwrap().item.id, 2);
    }

    #[test]
    fn dialog_done_creates_connection_state_with_capturing_true() {
        let mut s = SidebarState::new();
        s.dialog = Some(crate::ui::dialog::ConnectionDialogState::new());
        if let Some(d) = &mut s.dialog {
            d.proxy_port = 27117;
            d.step = crate::ui::dialog::DialogStep::Step2;
        }
        s.update(SidebarMsg::Connections(ConnectionsMsg::DialogDone));
        assert_eq!(s.connections.len(), 1);
        assert!(s.connections[0].capturing);
        assert!(s.dialog.is_none());
    }

    #[test]
    fn dialog_done_sets_active_id() {
        let mut s = SidebarState::new();
        s.dialog = Some(crate::ui::dialog::ConnectionDialogState::new());
        if let Some(d) = &mut s.dialog {
            d.proxy_port = 27117;
        }
        s.update(SidebarMsg::Connections(ConnectionsMsg::DialogDone));
        assert!(s.active_id.is_some());
        assert_eq!(s.active_id, Some(s.connections[0].item.id));
    }

    #[test]
    fn client_toggle_is_radio_style() {
        let mut s = SidebarState::default_for_test();
        s.connections[0].clients = vec![
            crate::ui::sidebar::clients::ClientItem {
                name: "app1".into(),
                color: [0, 0, 0],
                active: false,
            },
            crate::ui::sidebar::clients::ClientItem {
                name: "app2".into(),
                color: [0, 0, 0],
                active: false,
            },
        ];
        s.update(SidebarMsg::Clients(ClientsMsg::Toggle("app1".into())));
        assert!(s.connections[0].clients[0].active);
        assert!(!s.connections[0].clients[1].active);

        s.update(SidebarMsg::Clients(ClientsMsg::Toggle("app2".into())));
        assert!(!s.connections[0].clients[0].active);
        assert!(s.connections[0].clients[1].active);

        // toggle active one off
        s.update(SidebarMsg::Clients(ClientsMsg::Toggle("app2".into())));
        assert!(!s.connections[0].clients[0].active);
        assert!(!s.connections[0].clients[1].active);
    }

    #[test]
    fn active_client_returns_active_name() {
        let mut s = SidebarState::default_for_test();
        s.connections[0].clients = vec![crate::ui::sidebar::clients::ClientItem {
            name: "myapp".into(),
            color: [0, 0, 0],
            active: false,
        }];
        assert_eq!(s.active_client(), None);
        s.connections[0].clients[0].active = true;
        assert_eq!(s.active_client(), Some("myapp".into()));
    }
}
