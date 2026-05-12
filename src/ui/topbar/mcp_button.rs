use iced::{
    widget::{button, container, row, text},
    Alignment, Background, Border, Color, Element, Padding,
};
use crate::theme::Palette;
use crate::ui::mcp_panel::McpServerState;

fn dot<Msg: 'static>(color: Color) -> Element<'static, Msg> {
    container(iced::widget::Space::new(0, 0))
        .width(7)
        .height(7)
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            border: Border { radius: 3.5.into(), ..Default::default() },
            ..Default::default()
        })
        .into()
}

pub fn mcp_button<Msg: Clone + 'static>(
    server: &McpServerState,
    on_press: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let (dot_color, label_color, border_color) = match server {
        McpServerState::Stopped  => (palette.fg_dim2, palette.fg_dim,  palette.border2),
        McpServerState::Starting => (palette.warn,    palette.warn,    palette.warn),
        McpServerState::Running { .. } => (palette.ok, palette.ok,     palette.ok),
    };
    let bg1 = palette.bg1;

    button(
        row![
            dot(dot_color),
            text("MCP").size(10).color(label_color).font(iced::Font::MONOSPACE),
        ]
        .spacing(5)
        .align_y(Alignment::Center),
    )
    .on_press(on_press)
    .padding(Padding { top: 2.0, bottom: 2.0, left: 9.0, right: 9.0 })
    .style(move |_, _| button::Style {
        background: Some(Background::Color(bg1)),
        border: Border { color: border_color, width: 1.0, radius: 3.0.into() },
        text_color: label_color,
        ..Default::default()
    })
    .into()
}
