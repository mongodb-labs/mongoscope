use iced::{widget::{button, column, row, text}, Border, Element, Length, Padding};
use crate::theme::Palette;
use crate::ui::sidebar::collections::CollectionItem;

#[derive(Debug, Clone)]
pub struct DatabaseItem {
    pub name: String,
    pub expanded: bool,
    pub active: bool,
    pub collections: Vec<CollectionItem>,
}

#[derive(Debug, Clone)]
pub enum DatabasesMsg {
    ToggleDb(String),
    ToggleCollection(String, String),
}

pub fn apply_toggle_db(databases: &mut Vec<DatabaseItem>, name: &str) {
    for db in databases.iter_mut() {
        if db.name == name {
            db.active = !db.active;
            if db.active {
                db.expanded = true;
            } else {
                db.expanded = false;
            }
            for c in &mut db.collections {
                c.active = false;
            }
        } else {
            db.active = false;
            for c in &mut db.collections {
                c.active = false;
            }
        }
    }
}

pub fn apply_toggle_collection(databases: &mut Vec<DatabaseItem>, db_name: &str, coll_name: &str) {
    for db in databases.iter_mut() {
        if db.name == db_name {
            db.active = true;
            db.expanded = true;
            for c in &mut db.collections {
                if c.name == coll_name {
                    c.active = !c.active;
                } else {
                    c.active = false;
                }
            }
        } else {
            db.active = false;
            for c in &mut db.collections {
                c.active = false;
            }
        }
    }
}

pub fn databases_panel<Msg: Clone + 'static>(
    databases: &[DatabaseItem],
    on_msg: impl Fn(DatabasesMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0 = palette.bg;
    let bg_sel = palette.bg_sel;
    let bg_hover = palette.bg_hover;
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;

    let rows: Vec<Element<Msg>> = databases
        .iter()
        .flat_map(|db| {
            let db_name = db.name.clone();
            let is_db_active = db.active;
            let chevron = if db.expanded { "▾" } else { "▸" };
            let db_bg = if is_db_active { bg_sel } else { bg0 };
            let db_name_click = db_name.clone();

            let db_row: Element<Msg> = button(
                row![
                    text(chevron)
                        .size(10)
                        .color(fg_dim)
                        .font(iced::Font::MONOSPACE),
                    text(db_name.clone())
                        .size(11)
                        .color(fg)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .padding(Padding { top: 5.0, bottom: 5.0, left: 8.0, right: 8.0 })
            .width(Length::Fill)
            .on_press(on_msg(DatabasesMsg::ToggleDb(db_name_click)))
            .style(move |_, status| button::Style {
                background: Some(iced::Background::Color(match status {
                    iced::widget::button::Status::Hovered if !is_db_active => bg_hover,
                    _ => db_bg,
                })),
                border: Border::default(),
                ..Default::default()
            })
            .into();

            let mut items: Vec<Element<Msg>> = vec![db_row];

            if db.expanded {
                for coll in &db.collections {
                    let is_coll_active = coll.active;
                    let coll_bg = if is_coll_active { bg_sel } else { bg0 };
                    let coll_name = coll.name.clone();
                    let sub = coll.requests_str();
                    let db_for_coll = db.name.clone();
                    let coll_name_click = coll_name.clone();

                    let coll_row: Element<Msg> = button(
                        row![
                            text("◧")
                                .size(11)
                                .color(fg_dim2)
                                .font(iced::Font::MONOSPACE),
                            column![
                                text(coll_name.clone())
                                    .size(11)
                                    .color(fg)
                                    .font(iced::Font::MONOSPACE),
                                text(sub)
                                    .size(9)
                                    .color(fg_dim2)
                                    .font(iced::Font::MONOSPACE),
                            ]
                            .spacing(1)
                            .width(Length::Fill),
                        ]
                        .spacing(5)
                        .align_y(iced::Alignment::Center),
                    )
                    .padding(Padding { top: 5.0, bottom: 5.0, left: 20.0, right: 8.0 })
                    .width(Length::Fill)
                    .on_press(on_msg(DatabasesMsg::ToggleCollection(
                        db_for_coll,
                        coll_name_click,
                    )))
                    .style(move |_, status| button::Style {
                        background: Some(iced::Background::Color(match status {
                            iced::widget::button::Status::Hovered if !is_coll_active => bg_hover,
                            _ => coll_bg,
                        })),
                        border: Border::default(),
                        ..Default::default()
                    })
                    .into();

                    items.push(coll_row);
                }
            }

            items
        })
        .collect();

    column(rows)
        .spacing(1)
        .padding(Padding { top: 4.0, bottom: 4.0, left: 0.0, right: 0.0 })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db(name: &str, expanded: bool, colls: &[&str]) -> DatabaseItem {
        DatabaseItem {
            name: name.into(),
            expanded,
            active: false,
            collections: colls
                .iter()
                .map(|c| CollectionItem {
                    name: c.to_string(),
                    requests: 0,
                    active: false,
                })
                .collect(),
        }
    }

    #[test]
    fn toggle_db_activates_and_expands() {
        let mut dbs = vec![make_db("shop", false, &["orders"]), make_db("auth", false, &["tokens"])];
        apply_toggle_db(&mut dbs, "shop");
        assert!(dbs[0].active);
        assert!(dbs[0].expanded);
        assert!(!dbs[1].active);
    }

    #[test]
    fn toggle_db_deactivates_when_already_active() {
        let mut dbs = vec![make_db("shop", true, &["orders"])];
        dbs[0].active = true;
        apply_toggle_db(&mut dbs, "shop");
        assert!(!dbs[0].active);
    }

    #[test]
    fn toggle_db_collapses_on_deactivation() {
        let mut dbs = vec![make_db("shop", true, &["orders"])];
        dbs[0].active = true;
        apply_toggle_db(&mut dbs, "shop");
        assert!(!dbs[0].active);
        assert!(!dbs[0].expanded);
    }

    #[test]
    fn toggle_db_deactivates_other_dbs() {
        let mut dbs = vec![make_db("shop", true, &[]), make_db("auth", false, &[])];
        dbs[0].active = true;
        apply_toggle_db(&mut dbs, "auth");
        assert!(!dbs[0].active);
        assert!(dbs[1].active);
    }

    #[test]
    fn toggle_collection_activates_parent_db() {
        let mut dbs = vec![make_db("shop", true, &["orders", "products"])];
        apply_toggle_collection(&mut dbs, "shop", "orders");
        assert!(dbs[0].active);
        assert!(dbs[0].collections[0].active);
        assert!(!dbs[0].collections[1].active);
    }

    #[test]
    fn toggle_collection_deactivates_when_already_active() {
        let mut dbs = vec![make_db("shop", true, &["orders"])];
        dbs[0].active = true;
        dbs[0].collections[0].active = true;
        apply_toggle_collection(&mut dbs, "shop", "orders");
        assert!(dbs[0].active);
        assert!(!dbs[0].collections[0].active);
    }

    #[test]
    fn toggle_collection_clears_other_dbs() {
        let mut dbs = vec![
            make_db("shop", true, &["orders"]),
            make_db("auth", true, &["tokens"]),
        ];
        dbs[0].active = true;
        apply_toggle_collection(&mut dbs, "auth", "tokens");
        assert!(!dbs[0].active);
        assert!(dbs[1].active);
        assert!(dbs[1].collections[0].active);
    }
}
