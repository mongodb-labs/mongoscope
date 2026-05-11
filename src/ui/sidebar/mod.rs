pub mod clients;
pub mod collections;
pub mod connections;
pub mod databases;
pub mod saved_views;

pub use clients::{clients_panel, ClientItem, ClientsMsg};
pub use collections::CollectionItem;
pub use connections::{connections_panel, ConnectionItem, ConnectionsMsg};
pub use databases::{
    apply_toggle_collection, apply_toggle_db, databases_panel, DatabaseItem, DatabasesMsg,
};
pub use saved_views::{saved_views_panel, SavedView, SavedViewsMsg};

use iced::{
    widget::{column, container, row, scrollable, text},
    Border, Element, Length, Padding,
};
use crate::{data::model::QueryEntry, theme::Palette};

#[derive(Debug, Clone)]
pub enum SidebarMsg {
    Connections(ConnectionsMsg),
    Databases(DatabasesMsg),
    Clients(ClientsMsg),
    SavedViews(SavedViewsMsg),
}

pub struct SidebarState {
    pub databases: Vec<DatabaseItem>,
    pub connections: Vec<ConnectionItem>,
    pub clients: Vec<ClientItem>,
    pub saved_views: Vec<SavedView>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            connections: vec![ConnectionItem {
                id: 0,
                label: "localhost".into(),
                topology: "direct".into(),
                active: true,
                live: true,
            }],
            databases: vec![
                DatabaseItem {
                    name: "shop".into(),
                    expanded: true,
                    active: false,
                    collections: vec![
                        CollectionItem { name: "orders".into(),    docs: 2_413_882,  size: "8.4 GB".into(),   idx: 7, active: false },
                        CollectionItem { name: "products".into(),  docs: 184_302,    size: "412 MB".into(),   idx: 5, active: false },
                        CollectionItem { name: "users".into(),     docs: 892_014,    size: "1.8 GB".into(),   idx: 6, active: false },
                        CollectionItem { name: "carts".into(),     docs: 71_205,     size: "98 MB".into(),    idx: 3, active: false },
                        CollectionItem { name: "sessions".into(),  docs: 12_044_119, size: "4.2 GB".into(),   idx: 4, active: false },
                        CollectionItem { name: "reviews".into(),   docs: 3_201_885,  size: "2.1 GB".into(),   idx: 5, active: false },
                        CollectionItem { name: "inventory".into(), docs: 48_112,     size: "64 MB".into(),    idx: 4, active: false },
                        CollectionItem { name: "events".into(),    docs: 88_912_004, size: "41.2 GB".into(),  idx: 2, active: false },
                    ],
                },
                DatabaseItem {
                    name: "analytics".into(),
                    expanded: false,
                    active: false,
                    collections: vec![
                        CollectionItem { name: "pageviews".into(), docs: 12_500_000, size: "8.2 GB".into(),  idx: 3, active: false },
                        CollectionItem { name: "funnels".into(),   docs: 420_100,   size: "312 MB".into(),  idx: 2, active: false },
                    ],
                },
                DatabaseItem {
                    name: "auth".into(),
                    expanded: false,
                    active: false,
                    collections: vec![
                        CollectionItem { name: "tokens".into(), docs: 2_100_000, size: "1.4 GB".into(), idx: 2, active: false },
                    ],
                },
            ],
            clients: vec![],
            saved_views: vec![
                SavedView { id: 0, label: "slow queries (>500ms)".into() },
                SavedView { id: 1, label: "COLLSCANs only".into() },
                SavedView { id: 2, label: "writes to orders".into() },
            ],
        }
    }

    pub fn active_db(&self) -> Option<String> {
        self.databases.iter().find(|d| d.active).map(|d| d.name.clone())
    }

    pub fn active_coll(&self) -> Option<String> {
        self.databases
            .iter()
            .find(|d| d.active)
            .and_then(|d| d.collections.iter().find(|c| c.active))
            .map(|c| c.name.clone())
    }

    pub fn register_entries(&mut self, entries: &[QueryEntry]) {
        for entry in entries {
            // Register client app
            let app_name = entry.app.to_string();
            if !self.clients.iter().any(|c| c.name == app_name) {
                let color = clients::app_color_for(&app_name);
                self.clients.push(ClientItem { name: app_name, color, active: false });
            }
            // Register database/collection (in case live traffic reveals new ones)
            let db_name = entry.db.to_string();
            let coll_name = entry.coll.to_string();
            if let Some(db) = self.databases.iter_mut().find(|d| d.name == db_name) {
                if !db.collections.iter().any(|c| c.name == coll_name) {
                    db.collections.push(CollectionItem {
                        name: coll_name,
                        docs: 0,
                        size: "".into(),
                        idx: 0,
                        active: false,
                    });
                }
            } else {
                self.databases.push(DatabaseItem {
                    name: db_name,
                    expanded: true,
                    active: false,
                    collections: vec![CollectionItem {
                        name: coll_name,
                        docs: 0,
                        size: "".into(),
                        idx: 0,
                        active: false,
                    }],
                });
            }
        }
    }

    pub fn update(&mut self, msg: SidebarMsg) {
        match msg {
            SidebarMsg::Connections(m) => match m {
                ConnectionsMsg::Select(id) => {
                    for c in &mut self.connections {
                        c.active = c.id == id;
                    }
                }
                ConnectionsMsg::Add => {}
            },
            SidebarMsg::Databases(m) => match m {
                DatabasesMsg::ToggleDb(name) => apply_toggle_db(&mut self.databases, &name),
                DatabasesMsg::ToggleCollection(db, coll) => {
                    apply_toggle_collection(&mut self.databases, &db, &coll)
                }
            },
            SidebarMsg::Clients(m) => match m {
                ClientsMsg::Toggle(name) => {
                    for c in &mut self.clients {
                        if c.name == name {
                            c.active = !c.active;
                        }
                    }
                }
            },
            SidebarMsg::SavedViews(m) => match m {
                SavedViewsMsg::Delete(id) => self.saved_views.retain(|v| v.id != id),
                SavedViewsMsg::Load(_) | SavedViewsMsg::Save => {}
            },
        }
    }

    pub fn view<Msg: Clone + 'static>(
        &self,
        on_msg: impl Fn(SidebarMsg) -> Msg + 'static + Copy,
        palette: &Palette,
        width: f32,
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
                .padding(Padding { top: 8.0, bottom: 3.0, left: 8.0, right: 8.0 })
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(bg1)),
                    border: Border { color: border_color, width: 0.0, radius: 0.0.into() },
                    ..Default::default()
                })
                .into()
        };

        let db_count = self.databases.len();

        let content = column![
            section_header("CONNECTIONS".into(), None),
            connections_panel(
                &self.connections,
                move |m| on_msg(SidebarMsg::Connections(m)),
                palette,
            ),
            section_header("DATABASES".into(), Some(format!("{} dbs", db_count))),
            databases_panel(
                &self.databases,
                move |m| on_msg(SidebarMsg::Databases(m)),
                palette,
            ),
            section_header("CLIENTS".into(), None),
            clients_panel(
                &self.clients,
                move |m| on_msg(SidebarMsg::Clients(m)),
                palette,
            ),
            section_header("SAVED VIEWS".into(), None),
            saved_views_panel(
                &self.saved_views,
                move |m| on_msg(SidebarMsg::SavedViews(m)),
                palette,
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
                        scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => 1.0,
                    };
                    scrollable::Style {
                        container: iced::widget::container::Style::default(),
                        vertical_rail: scrollable::Rail {
                            background: Some(iced::Background::Color(iced::Color { a: a * 0.5, ..bg })),
                            border: Border::default(),
                            scroller: scrollable::Scroller {
                                color: iced::Color { a, ..fg_dim2 },
                                border: Border { radius: 4.0.into(), ..Default::default() },
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
            border: Border { color: border_color, width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }
}
