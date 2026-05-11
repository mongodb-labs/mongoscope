use iced::{widget::{button, column, container, row, scrollable, text, text_editor}, Border, Element, Length, Padding};
use crate::theme::Palette;

#[derive(Debug, Clone)]
pub enum ComposeMsg {
    EditorAction(text_editor::Action),
    RunQuery,
    CopyToClipboard,
}

pub struct ComposeState {
    pub content: text_editor::Content,
}

impl ComposeState {
    pub fn new() -> Self {
        Self { content: text_editor::Content::new() }
    }

    pub fn with_query(query: &str) -> Self {
        Self { content: text_editor::Content::with_text(query) }
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
    ) -> Element<'a, Msg> {
        let bg2 = palette.bg2;
        let border_color = palette.border;
        let accent = palette.accent;
        let accent_fg = palette.accent_fg;
        let fg_dim2 = palette.fg_dim2;
        let fg = palette.fg;

        let editor = text_editor(&self.content)
            .size(fs)
            .font(iced::Font::MONOSPACE)
            .on_action(move |a| on_msg(ComposeMsg::EditorAction(a)))
            .style(move |_, _| text_editor::Style {
                background: iced::Background::Color(bg2),
                border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
                icon: fg,
                placeholder: fg_dim2,
                value: fg,
                selection: accent,
            });

        let run_btn = button(
            text("▶ Run").size(fs).color(accent_fg).font(iced::Font::MONOSPACE)
        )
        .padding(Padding { top: 5.0, bottom: 5.0, left: 12.0, right: 12.0 })
        .on_press(on_msg(ComposeMsg::RunQuery))
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(accent)),
            border: Border { radius: 4.0.into(), ..Default::default() },
            ..Default::default()
        });

        let copy_btn = button(
            text("Copy").size(fs).color(palette.fg_dim).font(iced::Font::MONOSPACE)
        )
        .padding(Padding { top: 5.0, bottom: 5.0, left: 10.0, right: 10.0 })
        .on_press(on_msg(ComposeMsg::CopyToClipboard))
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(bg2)),
            border: Border { color: border_color, width: 1.0, radius: 4.0.into() },
            ..Default::default()
        });

        column![
            editor.height(Length::Fill),
            row![copy_btn, run_btn].spacing(8),
        ]
        .spacing(8)
        .padding(12)
        .height(Length::Fill)
        .into()
    }
}
