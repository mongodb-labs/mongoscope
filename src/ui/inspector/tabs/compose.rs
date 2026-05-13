use crate::theme::Palette;
use iced::{
    widget::{button, column, container, row, text, text_editor},
    Border, Color, Element, Length, Padding,
};

#[derive(Debug, Clone)]
pub enum ComposeMsg {
    EditorAction(text_editor::Action),
    RunQuery,
    // TODO: remove when real backend is wired up — currently all mock data
    #[allow(dead_code)]
    CopyToClipboard,
}

pub struct ComposeState {
    pub content: text_editor::Content,
}

impl ComposeState {
    pub fn new() -> Self {
        Self {
            content: text_editor::Content::new(),
        }
    }

    // TODO: remove when real backend is wired up — currently all mock data
    #[allow(dead_code)]
    pub fn with_query(query: &str) -> Self {
        Self {
            content: text_editor::Content::with_text(query),
        }
    }

    pub fn update(&mut self, msg: ComposeMsg) {
        if let ComposeMsg::EditorAction(action) = msg {
            self.content.perform(action);
        }
    }

    pub fn view<'a, Msg: Clone + 'static>(
        &'a self,
        on_msg: impl Fn(ComposeMsg) -> Msg + 'static + Copy,
        palette: Palette,
        fs: f32,
        cluster_label: &str,
        shell_version: &str,
    ) -> Element<'a, Msg> {
        let cluster_label = cluster_label.to_owned();
        let shell_version = shell_version.to_owned();
        let bg = palette.bg;
        let bg1 = palette.bg1;
        let _bg2 = palette.bg2;
        let border_color = palette.border;
        let accent = palette.accent;
        let accent_fg = palette.accent_fg;
        let fg_dim = palette.fg_dim;
        let fg_dim2 = palette.fg_dim2;
        let fg = palette.fg;
        let fs_small = (fs - 1.0).max(9.0);

        fn ghost_act<Msg: 'static>(
            label: &str,
            fg: Color,
            bg: Color,
            border: Color,
            fs: f32,
        ) -> Element<'static, Msg> {
            container(
                text(label.to_string())
                    .size(fs)
                    .color(fg)
                    .font(iced::Font::MONOSPACE),
            )
            .padding(Padding {
                top: 3.0,
                bottom: 3.0,
                left: 10.0,
                right: 10.0,
            })
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: Border {
                    color: border,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                ..Default::default()
            })
            .into()
        }

        // ── compose header
        let composehd = container(
            row![
                row![
                    text("replay on ")
                        .size(fs_small)
                        .color(fg_dim)
                        .font(iced::Font::MONOSPACE),
                    text(cluster_label)
                        .size(fs_small)
                        .color(accent)
                        .font(iced::Font::MONOSPACE),
                    text(" · or switch →")
                        .size(fs_small)
                        .color(fg_dim)
                        .font(iced::Font::MONOSPACE),
                ],
                iced::widget::Space::new(Length::Fill, 0),
                row![
                    ghost_act::<Msg>("↻ Replay", fg_dim, bg, border_color, fs_small),
                    ghost_act::<Msg>("◇ Dry-run", fg_dim, bg, border_color, fs_small),
                    button(
                        text("▶ Run (⌘↵)")
                            .size(fs_small)
                            .color(accent_fg)
                            .font(iced::Font::MONOSPACE)
                    )
                    .padding(Padding {
                        top: 3.0,
                        bottom: 3.0,
                        left: 10.0,
                        right: 10.0
                    })
                    .on_press(on_msg(ComposeMsg::RunQuery))
                    .style(move |_, _| button::Style {
                        background: Some(iced::Background::Color(accent)),
                        border: Border {
                            color: accent,
                            width: 1.0,
                            radius: 5.0.into()
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            ]
            .align_y(iced::Alignment::Center)
            .padding(Padding {
                top: 8.0,
                bottom: 8.0,
                left: 14.0,
                right: 8.0,
            }),
        )
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg1)),
            border: Border {
                color: border_color,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        // ── editor
        let editor = text_editor(&self.content)
            .size(fs)
            .font(iced::Font::MONOSPACE)
            .on_action(move |a| on_msg(ComposeMsg::EditorAction(a)))
            .style(move |_, _| text_editor::Style {
                background: iced::Background::Color(bg),
                border: Border {
                    color: border_color,
                    width: 0.0,
                    radius: 0.0.into(),
                },
                icon: fg,
                placeholder: fg_dim2,
                value: fg,
                selection: accent,
            });

        // ── compose footer
        let composefoot = container(
            row![
                text(format!("shell · {}", shell_version))
                    .size(10)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
                iced::widget::Space::new(Length::Fill, 0),
                text("↑↓ history · ⌘K palette · ⌘↵ run")
                    .size(10)
                    .color(fg_dim2)
                    .font(iced::Font::MONOSPACE),
            ]
            .padding(Padding {
                top: 6.0,
                bottom: 6.0,
                left: 14.0,
                right: 14.0,
            }),
        )
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(bg1)),
            border: Border {
                color: border_color,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

        column![composehd, editor.height(Length::Fill), composefoot,]
            .spacing(0)
            .height(Length::Fill)
            .into()
    }
}
