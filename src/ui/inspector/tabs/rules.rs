use iced::{widget::{button, column, container, row, scrollable, text, text_input}, Border, Element, Length, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: usize,
    pub pattern: String,
    pub action: RuleAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleAction {
    Highlight,
    Warn,
    Block,
}

impl RuleAction {
    pub fn label(&self) -> &'static str {
        match self {
            RuleAction::Highlight => "highlight",
            RuleAction::Warn => "warn",
            RuleAction::Block => "block",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RulesMsg {
    Toggle(usize),
    Delete(usize),
    AddNew,
    PatternChanged(usize, String),
}

pub fn rules_tab<Msg: Clone + 'static>(
    rules: &[Rule],
    on_msg: impl Fn(RulesMsg) -> Msg + 'static + Copy,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let bg1 = palette.bg1;
    let bg2 = palette.bg2;
    let border_color = palette.border;
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let accent = palette.accent;
    let danger = palette.danger;
    let warn = palette.warn;

    let mut rows: Vec<Element<Msg>> = rules.iter().map(|r| {
        let id = r.id;
        let enabled = r.enabled;
        let action_color = match r.action {
            RuleAction::Highlight => accent,
            RuleAction::Warn => warn,
            RuleAction::Block => danger,
        };
        let action_label = r.action.label();
        let pattern = r.pattern.clone();

        container(
            row![
                container(iced::widget::Space::new(4, 4))
                    .width(4).height(24)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(action_color)),
                        ..Default::default()
                    }),
                column![
                    text(action_label).size(9).color(action_color).font(iced::Font::MONOSPACE),
                    text(pattern).size(fs).color(if enabled { fg } else { fg_dim2 })
                        .font(iced::Font::MONOSPACE),
                ].spacing(1).width(Length::Fill),
                button(text("×").size(11).color(fg_dim).font(iced::Font::MONOSPACE))
                    .padding(Padding { top: 2.0, bottom: 2.0, left: 6.0, right: 6.0 })
                    .on_press(on_msg(RulesMsg::Delete(id)))
                    .style(|_, _| button::Style { background: None, border: Border::default(), ..Default::default() }),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center)
            .padding(Padding { top: 6.0, bottom: 6.0, left: 0.0, right: 8.0 })
        )
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg1)),
            border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        })
        .into()
    }).collect();

    rows.push(
        button(text("+ Add rule").size(fs).color(accent).font(iced::Font::MONOSPACE))
            .padding(Padding { top: 6.0, bottom: 6.0, left: 8.0, right: 8.0 })
            .width(Length::Fill)
            .on_press(on_msg(RulesMsg::AddNew))
            .style(move |_, _| button::Style {
                background: Some(iced::Background::Color(bg1)),
                border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
                ..Default::default()
            })
            .into()
    );

    scrollable(column(rows).spacing(6).padding(12))
        .height(Length::Fill)
        .into()
}
