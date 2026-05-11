pub mod header;
pub mod tabs;

use iced::{widget::{button, column, container, row}, Border, Element, Length, Padding};
use crate::{
    data::model::QueryEntry,
    theme::Palette,
};
use tabs::{
    ComposeMsg, ComposeState, Rule, RulesMsg,
    explain_tab, overview_tab, request_tab, response_tab,
    rules_tab, schema_tab, timeline_tab,
};
use header::inspector_header;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorTab {
    Overview,
    Request,
    Response,
    Explain,
    Timeline,
    Compose,
    Rules,
    Schema,
}

impl InspectorTab {
    pub fn label(self) -> &'static str {
        match self {
            InspectorTab::Overview  => "Overview",
            InspectorTab::Request   => "Request",
            InspectorTab::Response  => "Response",
            InspectorTab::Explain   => "Explain",
            InspectorTab::Timeline  => "Timeline",
            InspectorTab::Compose   => "Compose",
            InspectorTab::Rules     => "Rules",
            InspectorTab::Schema    => "Schema",
        }
    }
    pub fn all() -> &'static [InspectorTab] {
        &[
            InspectorTab::Overview,
            InspectorTab::Request,
            InspectorTab::Response,
            InspectorTab::Explain,
            InspectorTab::Timeline,
            InspectorTab::Compose,
            InspectorTab::Rules,
            InspectorTab::Schema,
        ]
    }
}

#[derive(Debug, Clone)]
pub enum InspectorMsg {
    TabSelect(InspectorTab),
    Compose(ComposeMsg),
    Rules(RulesMsg),
}

pub struct InspectorState {
    pub tab: InspectorTab,
    pub compose: ComposeState,
    pub rules: Vec<Rule>,
}

impl InspectorState {
    pub fn new() -> Self {
        Self {
            tab: InspectorTab::Overview,
            compose: ComposeState::new(),
            rules: vec![],
        }
    }

    pub fn update(&mut self, msg: InspectorMsg) {
        match msg {
            InspectorMsg::TabSelect(tab) => {
                self.tab = tab;
            }
            InspectorMsg::Compose(m) => self.compose.update(m),
            InspectorMsg::Rules(RulesMsg::Delete(id)) => self.rules.retain(|r| r.id != id),
            InspectorMsg::Rules(RulesMsg::AddNew) => {
                let id = self.rules.len();
                self.rules.push(Rule {
                    id,
                    pattern: String::new(),
                    action: tabs::RuleAction::Highlight,
                    enabled: true,
                });
            }
            InspectorMsg::Rules(_) => {}
        }
    }

    pub fn view<'a, Msg: Clone + 'static>(
        &'a self,
        entry: Option<&'a QueryEntry>,
        on_msg: impl Fn(InspectorMsg) -> Msg + 'static + Copy,
        palette: Palette,
        fs: f32,
    ) -> Element<'a, Msg> {
        let bg = palette.bg1;
        let bg2 = palette.bg2;
        let border_color = palette.border;
        let accent = palette.accent;
        let bg_sel = palette.bg_sel;
        let fg = palette.fg;
        let fg_dim = palette.fg_dim;
        let active_tab = self.tab;

        // Tab bar
        let tab_items: Vec<Element<Msg>> = InspectorTab::all().iter().map(|&tab| {
            let is_active = tab == active_tab;
            let bg_tab = if is_active { bg_sel } else { bg2 };
            let fg_tab = if is_active { fg } else { fg_dim };
            let accent_c = accent;

            button(
                iced::widget::text(tab.label()).size(10).color(fg_tab).font(iced::Font::MONOSPACE)
            )
            .padding(Padding { top: 4.0, bottom: 4.0, left: 8.0, right: 8.0 })
            .on_press(on_msg(InspectorMsg::TabSelect(tab)))
            .style(move |_, _| button::Style {
                background: Some(iced::Background::Color(bg_tab)),
                border: if is_active {
                    Border { color: accent_c, width: 0.0, radius: 0.0.into() }
                } else {
                    Border::default()
                },
                ..Default::default()
            })
            .into()
        }).collect();

        let tab_bar = container(row(tab_items).spacing(1))
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg2)),
                border: Border { color: border_color, width: 1.0, radius: 0.0.into() },
                ..Default::default()
            });

        let fg_dim2 = palette.fg_dim2;
        let content: Element<'a, Msg> = match entry {
            None => iced::widget::text("Select a query to inspect")
                .size(fs).color(fg_dim2).font(iced::Font::MONOSPACE).into(),
            Some(e) => match self.tab {
                InspectorTab::Overview  => overview_tab(e, &palette, fs),
                InspectorTab::Request   => request_tab(e, &palette, fs),
                InspectorTab::Response  => response_tab(e, &palette, fs),
                InspectorTab::Explain   => explain_tab(e, &palette, fs),
                InspectorTab::Timeline  => timeline_tab(e, &palette, fs),
                InspectorTab::Compose   => self.compose.view(
                    move |m| on_msg(InspectorMsg::Compose(m)), palette, fs
                ),
                InspectorTab::Rules     => rules_tab(
                    &self.rules,
                    move |m| on_msg(InspectorMsg::Rules(m)),
                    &palette, fs
                ),
                InspectorTab::Schema    => schema_tab(e, &palette, fs),
            },
        };

        let header_el: Element<'a, Msg> = match entry {
            Some(e) => inspector_header(e, &palette, fs),
            None => iced::widget::Space::new(0, 0).into(),
        };

        container(
            column![header_el, tab_bar, content].spacing(0)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: border_color, width: 1.0, radius: 0.0.into() },
            ..Default::default()
        })
        .into()
    }
}
