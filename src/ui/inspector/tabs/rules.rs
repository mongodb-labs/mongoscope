use iced::{widget::{button, column, container, row, scrollable, text}, Border, Color, Element, Length, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: usize,
    pub pattern: String,
    pub action: RuleAction,
    pub enabled: bool,
    pub hits: u32,
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
    pub fn do_label(&self) -> &'static str {
        match self {
            RuleAction::Highlight => "tag + highlight match",
            RuleAction::Warn => "log + tag :slow",
            RuleAction::Block => "block & alert",
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

fn pill_toggle<Msg: Clone + 'static>(enabled: bool, msg: Msg, palette: &Palette) -> Element<'static, Msg> {
    let track_bg = if enabled { palette.accent } else { palette.border2 };
    let thumb_bg = palette.bg;

    let thumb = container(iced::widget::Space::new(10, 10))
        .width(10)
        .height(10)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(thumb_bg)),
            border: Border { radius: 5.0.into(), ..Default::default() },
            ..Default::default()
        });

    let inner = if enabled {
        row![iced::widget::Space::new(Length::Fill, 0), thumb]
            .padding(Padding { top: 2.0, bottom: 2.0, left: 2.0, right: 2.0 })
    } else {
        row![thumb, iced::widget::Space::new(Length::Fill, 0)]
            .padding(Padding { top: 2.0, bottom: 2.0, left: 2.0, right: 2.0 })
    };

    button(inner)
        .width(24)
        .height(14)
        .on_press(msg)
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(track_bg)),
            border: Border { radius: 7.0.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

pub fn rules_tab<Msg: Clone + 'static>(
    rules: &[Rule],
    on_msg: impl Fn(RulesMsg) -> Msg + 'static + Copy,
    palette: &Palette,
    fs: f32,
) -> Element<'static, Msg> {
    let bg1 = palette.bg1;
    let border_color = palette.border;
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let accent = palette.accent;
    let danger = palette.danger;
    let warn = palette.warn;
    let fs_small = (fs - 1.0).max(9.0);

    let total_hits: u32 = rules.iter().filter(|r| r.enabled).map(|r| r.hits).sum();
    let active_count = rules.iter().filter(|r| r.enabled).count();

    // ── header
    let rulehd = row![
        text(format!("{} active rules · matched {}× this session", active_count, total_hits))
            .size(fs_small).color(fg_dim).font(iced::Font::MONOSPACE),
        iced::widget::Space::new(Length::Fill, 0),
        button(text("+ New rule").size(fs_small).color(accent).font(iced::Font::MONOSPACE))
            .padding(Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 })
            .on_press(on_msg(RulesMsg::AddNew))
            .style(move |_, _| button::Style {
                background: Some(iced::Background::Color(bg1)),
                border: Border { color: border_color, width: 1.0, radius: 5.0.into() },
                ..Default::default()
            }),
    ]
    .align_y(iced::Alignment::Center)
    .spacing(8);

    // ── rule rows
    let mut rule_rows: Vec<Element<Msg>> = rules.iter().map(|r| {
        let id = r.id;
        let enabled = r.enabled;
        let action_color = match r.action {
            RuleAction::Highlight => accent,
            RuleAction::Warn => warn,
            RuleAction::Block => danger,
        };
        let pattern = r.pattern.clone();
        let do_label = r.action.do_label().to_string();
        let hits = r.hits;
        let opacity = if enabled { 1.0 } else { 0.55 };
        let text_col = Color { r: fg.r, g: fg.g, b: fg.b, a: opacity };
        let dim_col = Color { r: fg_dim2.r, g: fg_dim2.g, b: fg_dim2.b, a: opacity };
        let act_col = Color { r: action_color.r, g: action_color.g, b: action_color.b, a: opacity };

        container(
            row![
                pill_toggle(enabled, on_msg(RulesMsg::Toggle(id)), palette),
                column![
                    row![
                        text("WHEN ").size(fs_small).color(dim_col).font(iced::Font::MONOSPACE),
                        text(pattern).size(fs_small).color(text_col).font(iced::Font::MONOSPACE),
                    ],
                    row![
                        text("DO ").size(fs_small).color(dim_col).font(iced::Font::MONOSPACE),
                        text(do_label).size(fs_small).color(act_col).font(iced::Font::MONOSPACE),
                    ],
                ]
                .spacing(1)
                .width(Length::Fill),
                text(format!("{}×", hits)).size(10).color(dim_col).font(iced::Font::MONOSPACE),
                button(text("⋯").size(11).color(Color { r: fg_dim.r, g: fg_dim.g, b: fg_dim.b, a: opacity }).font(iced::Font::MONOSPACE))
                    .padding(Padding { top: 2.0, bottom: 2.0, left: 6.0, right: 6.0 })
                    .on_press(on_msg(RulesMsg::Delete(id)))
                    .style(move |_, _| button::Style {
                        background: None,
                        border: Border::default(),
                        ..Default::default()
                    }),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center)
            .padding(Padding { top: 8.0, bottom: 8.0, left: 10.0, right: 8.0 })
        )
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg1)),
            border: Border { color: border_color, width: 1.0, radius: 6.0.into() },
            ..Default::default()
        })
        .into()
    }).collect();

    // ── interception card (mock)
    let interception = container(
        column![
            text("PENDING INTERCEPTION").size(9).color(fg_dim2).font(iced::Font::MONOSPACE),
            iced::widget::Space::new(0, 4),
            text("paused → orders.aggregate · analytics-worker")
                .size(fs_small).color(fg).font(iced::Font::MONOSPACE),
            text("Rule matched: plan == 'COLLSCAN'")
                .size(fs_small).color(fg_dim).font(iced::Font::MONOSPACE),
            iced::widget::Space::new(0, 4),
            row![
                ghost_label("Edit request", fg, bg1, border_color),
                ghost_label("Step over", fg, bg1, border_color),
                ghost_label_solid("▶ Continue", palette.accent_fg, palette.accent),
                ghost_label_danger("✕ Abort", danger, bg1, border_color),
            ].spacing(4),
        ]
        .spacing(2)
    )
    .width(Length::Fill)
    .padding(Padding { top: 10.0, bottom: 10.0, left: 12.0, right: 12.0 })
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg1)),
        border: Border { color: border_color, width: 1.0, radius: 6.0.into() },
        ..Default::default()
    });

    let mut children: Vec<Element<Msg>> = vec![rulehd.into()];
    children.append(&mut rule_rows);
    children.push(interception.into());

    scrollable(
        column(children)
            .spacing(6)
            .padding(Padding { top: 14.0, bottom: 14.0, left: 16.0, right: 16.0 })
    )
    .height(Length::Fill)
    .into()
}

fn ghost_label<Msg: 'static>(label: &str, fg: Color, bg: Color, border: Color) -> Element<'static, Msg> {
    container(text(label.to_string()).size(10).color(fg).font(iced::Font::MONOSPACE))
        .padding(Padding { top: 3.0, bottom: 3.0, left: 10.0, right: 10.0 })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: border, width: 1.0, radius: 5.0.into() },
            ..Default::default()
        })
        .into()
}

fn ghost_label_solid<Msg: 'static>(label: &str, fg: Color, bg: Color) -> Element<'static, Msg> {
    container(text(label.to_string()).size(10).color(fg).font(iced::Font::MONOSPACE))
        .padding(Padding { top: 3.0, bottom: 3.0, left: 10.0, right: 10.0 })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: bg, width: 1.0, radius: 5.0.into() },
            ..Default::default()
        })
        .into()
}

fn ghost_label_danger<Msg: 'static>(label: &str, fg: Color, bg: Color, border: Color) -> Element<'static, Msg> {
    let border_c = Color { r: fg.r, g: fg.g, b: fg.b, a: 0.4 };
    container(text(label.to_string()).size(10).color(fg).font(iced::Font::MONOSPACE))
        .padding(Padding { top: 3.0, bottom: 3.0, left: 10.0, right: 10.0 })
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: border_c, width: 1.0, radius: 5.0.into() },
            ..Default::default()
        })
        .into()
}
