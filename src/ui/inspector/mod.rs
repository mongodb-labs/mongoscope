pub mod header;
pub mod tabs;

use crate::{data::model::QueryEntry, theme::Palette};
use header::inspector_header;
use iced::{
    widget::{column, container, row},
    Border, Color, Element, Length, Padding,
};
use tabs::{
    explain_tab, overview_tab, request_tab, response_tab, ComposeMsg, ComposeState, ExplainMsg,
    ExplainState, Rule, RuleAction, RulesMsg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorTab {
    Overview,
    Request,
    Response,
    Explain,
    Compose,
    Rules,
    Schema,
}

impl InspectorTab {
    pub fn label(self) -> &'static str {
        match self {
            InspectorTab::Overview => "Overview",
            InspectorTab::Request => "Request",
            InspectorTab::Response => "Response",
            InspectorTab::Explain => "Explain",
            InspectorTab::Compose => "Compose",
            InspectorTab::Rules => "Rules",
            InspectorTab::Schema => "Schema",
        }
    }
    pub fn all() -> &'static [InspectorTab] {
        // Compose (#33), Rules (#32), Schema (#28) hidden until implemented.
        &[
            InspectorTab::Overview,
            InspectorTab::Request,
            InspectorTab::Response,
            InspectorTab::Explain,
        ]
    }
}

#[derive(Debug, Clone)]
pub enum InspectorMsg {
    TabSelect(InspectorTab),
    Compose(ComposeMsg),
    Rules(RulesMsg),
    Explain(ExplainMsg),
    SuggestIndex,
    Close,
    ToggleMaximize,
}

pub struct InspectorState {
    pub tab: InspectorTab,
    pub compose: ComposeState,
    pub rules: Vec<Rule>,
    pub explain: ExplainState,
}

impl InspectorState {
    pub fn new() -> Self {
        Self {
            tab: InspectorTab::Overview,
            compose: ComposeState::new(),
            explain: ExplainState::default(),
            rules: vec![
                Rule {
                    id: 0,
                    pattern: "coll == 'orders' && plan == 'COLLSCAN'".into(),
                    action: RuleAction::Warn,
                    enabled: true,
                    hits: 3,
                },
                Rule {
                    id: 1,
                    pattern: "latency > 1000ms".into(),
                    action: RuleAction::Warn,
                    enabled: true,
                    hits: 12,
                },
                Rule {
                    id: 2,
                    pattern: "client == 'admin-portal' && op == 'deleteMany'".into(),
                    action: RuleAction::Block,
                    enabled: false,
                    hits: 0,
                },
                Rule {
                    id: 3,
                    pattern: "coll == 'users' && filter.email".into(),
                    action: RuleAction::Highlight,
                    enabled: true,
                    hits: 87,
                },
            ],
        }
    }

    pub fn seed_compose(&mut self, entry: &QueryEntry) {
        use crate::data::model::{BsonVal, Op};

        fn fmt_val(v: &BsonVal) -> String {
            match v {
                BsonVal::Str(s) => format!("\"{}\"", s),
                BsonVal::Int(n) => n.to_string(),
                BsonVal::Float(f) => format!("{:.2}", f),
                BsonVal::Bool(b) => b.to_string(),
                BsonVal::ObjectId(s) => format!("ObjectId(\"{}\")", s),
                BsonVal::IsoDate(s) => format!("ISODate(\"{}\")", s),
                BsonVal::Null => "null".into(),
                _ => "\"…\"".into(),
            }
        }

        let coll = entry.coll.as_str();
        let query = match &entry.op {
            Op::Find | Op::FindOne => {
                let filter = entry
                    .filter
                    .as_ref()
                    .map(|f| {
                        let pairs: Vec<String> = f
                            .iter()
                            .map(|(k, v)| format!("  \"{}\": {}", k, fmt_val(v)))
                            .collect();
                        format!("{{\n{}\n}}", pairs.join(",\n"))
                    })
                    .unwrap_or_else(|| "{}".into());
                let method = if matches!(entry.op, Op::FindOne) {
                    "findOne"
                } else {
                    "find"
                };
                format!("db.{}.{}(\n{}\n)", coll, method, filter)
            }
            Op::Aggregate => {
                let stages = entry
                    .pipeline
                    .as_ref()
                    .map(|p| {
                        let s: Vec<String> = p
                            .iter()
                            .map(|stage| {
                                let key = stage.keys().next().cloned().unwrap_or_default();
                                format!("  {{ {}: {{}} }}", key)
                            })
                            .collect();
                        format!("[\n{}\n]", s.join(",\n"))
                    })
                    .unwrap_or_else(|| "[]".into());
                format!("db.{}.aggregate(\n{}\n)", coll, stages)
            }
            Op::CountDocuments => {
                let filter = entry
                    .filter
                    .as_ref()
                    .map(|f| {
                        let pairs: Vec<String> = f
                            .iter()
                            .map(|(k, v)| format!("  \"{}\": {}", k, fmt_val(v)))
                            .collect();
                        format!("{{\n{}\n}}", pairs.join(",\n"))
                    })
                    .unwrap_or_else(|| "{}".into());
                format!("db.{}.countDocuments(\n{}\n)", coll, filter)
            }
            _ => format!("db.{}.find({{}})", coll),
        };

        self.compose = ComposeState::with_query(&query);
    }

    pub fn update(&mut self, msg: InspectorMsg) {
        match msg {
            InspectorMsg::TabSelect(tab) => {
                if InspectorTab::all().contains(&tab) {
                    self.tab = tab;
                }
            }
            InspectorMsg::Compose(m) => self.compose.update(m),
            InspectorMsg::Rules(RulesMsg::Toggle(id)) => {
                if let Some(r) = self.rules.iter_mut().find(|r| r.id == id) {
                    r.enabled = !r.enabled;
                }
            }
            InspectorMsg::Rules(RulesMsg::Delete(id)) => self.rules.retain(|r| r.id != id),
            InspectorMsg::Rules(RulesMsg::AddNew) => {
                let id = self.rules.iter().map(|r| r.id).max().unwrap_or(0) + 1;
                self.rules.push(Rule {
                    id,
                    pattern: String::new(),
                    action: RuleAction::Highlight,
                    enabled: true,
                    hits: 0,
                });
            }
            InspectorMsg::Rules(_) => {}
            InspectorMsg::SuggestIndex => {
                self.tab = InspectorTab::Explain;
                self.explain = ExplainState::default();
            }
            InspectorMsg::Explain(ExplainMsg::RunIndex) => {
                self.explain.index_applied = true;
            }
            InspectorMsg::Explain(ExplainMsg::CopyIndex) => {
                // No state change; clipboard write is handled in main.rs.
            }
            // Handled in main.rs before reaching InspectorState::update.
            InspectorMsg::Close | InspectorMsg::ToggleMaximize => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn view<'a, Msg: Clone + 'static>(
        &'a self,
        entry: Option<&'a QueryEntry>,
        maximized: bool,
        on_msg: impl Fn(InspectorMsg) -> Msg + 'static + Copy,
        palette: Palette,
        fs: f32,
        cluster_label: &str,
        shell_version: &str,
    ) -> Element<'a, Msg> {
        let bg1 = palette.bg1;
        let bg = palette.bg;
        let border_color = palette.border;
        let accent = palette.accent;
        let fg = palette.fg;
        let fg_dim = palette.fg_dim;
        let active_tab = self.tab;
        let fs_small = (fs - 1.0).max(9.0);

        // ── Tab bar: transparent background, 2px bottom indicator on active
        let tab_items: Vec<Element<Msg>> = InspectorTab::all()
            .iter()
            .map(|&tab| {
                let is_active = tab == active_tab;
                let fg_tab = if is_active { fg } else { fg_dim };
                let indicator_color = if is_active {
                    accent
                } else {
                    Color::TRANSPARENT
                };

                let label_area =
                    container(iced::widget::text(tab.label()).size(fs_small).color(fg_tab))
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center);

                let indicator = container(iced::widget::Space::new(Length::Fill, 0))
                    .height(2)
                    .width(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(indicator_color)),
                        ..Default::default()
                    });

                iced::widget::button(column![label_area, indicator].height(Length::Fixed(30.0)))
                    .padding(Padding {
                        top: 0.0,
                        bottom: 0.0,
                        left: 14.0,
                        right: 14.0,
                    })
                    .on_press(on_msg(InspectorMsg::TabSelect(tab)))
                    .style(move |_, _| iced::widget::button::Style {
                        background: Some(iced::Background::Color(Color::TRANSPARENT)),
                        border: Border::default(),
                        ..Default::default()
                    })
                    .into()
            })
            .collect();

        let tab_row = container(row(tab_items))
            .width(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg1)),
                ..Default::default()
            });

        let tab_border = container(iced::widget::Space::new(Length::Fill, 0))
            .height(1)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(border_color)),
                ..Default::default()
            });

        let tab_bar = column![tab_row, tab_border];

        let fg_dim2 = palette.fg_dim2;

        let content: Element<'a, Msg> = match entry {
            None => container(
                iced::widget::text("Select a query to inspect")
                    .size(fs)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .into(),
            Some(e) => match self.tab {
                InspectorTab::Overview => {
                    overview_tab(e, move || on_msg(InspectorMsg::SuggestIndex), &palette, fs)
                }
                InspectorTab::Request => request_tab(e, &palette, fs),
                InspectorTab::Response => response_tab(e, &palette, fs),
                InspectorTab::Explain => explain_tab(
                    e,
                    &self.explain,
                    move |m| on_msg(InspectorMsg::Explain(m)),
                    &palette,
                    fs,
                ),
                InspectorTab::Compose | InspectorTab::Rules | InspectorTab::Schema => {
                    // Hidden tabs (#28, #32, #33) — fall through to Overview.
                    overview_tab(e, move || on_msg(InspectorMsg::SuggestIndex), &palette, fs)
                }
            },
        };

        let header_el: Element<'a, Msg> = match entry {
            Some(e) => inspector_header(
                e,
                maximized,
                on_msg(InspectorMsg::Close),
                on_msg(InspectorMsg::ToggleMaximize),
                &palette,
                fs,
            ),
            None => iced::widget::Space::new(0, 0).into(),
        };

        container(column![header_el, tab_bar, content].spacing(0))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: Border {
                    color: border_color,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}
