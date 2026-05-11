pub mod clients;
pub mod collections;
pub mod connections;
pub mod databases;
pub mod saved_views;

pub use clients::{clients_panel, ClientItem, ClientsMsg};
pub use collections::{collections_panel, CollectionItem, CollectionsMsg};
pub use connections::{connections_panel, ConnectionItem, ConnectionsMsg};
pub use databases::{DatabaseItem, DatabasesMsg, databases_panel, apply_toggle_db, apply_toggle_collection};
pub use saved_views::{saved_views_panel, SavedView, SavedViewsMsg};

use iced::{widget::{column, container, row, scrollable, text}, Border, Element, Length, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone)]
pub enum SidebarMsg {
    Connections(ConnectionsMsg),
    Collections(CollectionsMsg),
    Clients(ClientsMsg),
    SavedViews(SavedViewsMsg),
}

pub struct SidebarState {
    pub db_name: String,
    pub connections: Vec<ConnectionItem>,
    pub collections: Vec<CollectionItem>,
    pub clients: Vec<ClientItem>,
    pub saved_views: Vec<SavedView>,
}

impl SidebarState {
    pub fn new() -> Self {
        Self {
            db_name: "shop".into(),
            connections: vec![
                ConnectionItem {
                    id: 0,
                    label: "localhost".into(),
                    topology: "direct".into(),
                    active: true,
                    live: true,
                },
            ],
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
            clients: vec![],
            saved_views: vec![
                SavedView { id: 0, label: "slow queries (>500ms)".into() },
                SavedView { id: 1, label: "COLLSCANs only".into() },
                SavedView { id: 2, label: "writes to orders".into() },
            ],
        }
    }

    pub fn register_entries(&mut self, entries: &[crate::data::model::QueryEntry]) {
        for entry in entries {
            let name = entry.app.to_string();
            if !self.clients.iter().any(|c| c.name == name) {
                let color = clients::app_color_for(&name);
                self.clients.push(ClientItem { name, color, active: false });
            }
        }
    }

    pub fn update(&mut self, msg: SidebarMsg) {
        match msg {
            SidebarMsg::Connections(m) => match m {
                ConnectionsMsg::Select(id) => {
                    for c in &mut self.connections { c.active = c.id == id; }
                }
                ConnectionsMsg::Add => {}
            },
            SidebarMsg::Collections(m) => match m {
                CollectionsMsg::Select(name) => {
                    for c in &mut self.collections {
                        if c.name == name { c.active = !c.active; } else { c.active = false; }
                    }
                }
            },
            SidebarMsg::Clients(m) => match m {
                ClientsMsg::Toggle(name) => {
                    for c in &mut self.clients {
                        if c.name == name { c.active = !c.active; }
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
    ) -> Element<'static, Msg> {
        let bg  = palette.bg;
        let bg1 = palette.bg1;
        let border_color = palette.border;
        let fg_dim  = palette.fg_dim;
        let fg_dim2 = palette.fg_dim2;
        let db_name = self.db_name.clone();
        let coll_count = self.collections.len();

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
                ].into()
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

        let db_header = format!("DATABASE · {}", db_name);
        let coll_right = format!("{} colls", coll_count);

        let content = column![
            section_header("CONNECTIONS".into(), None),
            connections_panel(
                &self.connections,
                move |m| on_msg(SidebarMsg::Connections(m)),
                palette,
            ),
            section_header(db_header, Some(coll_right)),
            collections_panel(
                &self.collections,
                move |m| on_msg(SidebarMsg::Collections(m)),
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
        )
        .width(200)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: border_color, width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }
}
