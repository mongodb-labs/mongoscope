pub mod capture_indicator;
pub mod conn_bar;
pub mod logo;
pub mod mcp_button;
pub mod menu_bar;

pub use capture_indicator::capture_indicator;
pub use conn_bar::{conn_bar, ConnInfo};
pub use logo::logo;
pub use mcp_button::mcp_button;
pub use menu_bar::{menu_bar, MenuMsg};

use crate::theme::Palette;
use crate::ui::mcp_panel::McpServerState;
use iced::{
    widget::{container, row},
    Border, Element, Length, Padding,
};

#[allow(clippy::too_many_arguments)]
pub fn topbar<Msg: Clone + 'static>(
    conn: &ConnInfo,
    capturing: bool,
    on_menu: impl Fn(MenuMsg) -> Msg + 'static,
    on_capture_toggle: Msg,
    mcp_server: &McpServerState,
    on_mcp_toggle: Msg,
    on_copy_uri: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg = palette.bg1;
    let border_color = palette.border;

    container(
        row![
            logo(palette),
            menu_bar(on_menu, palette),
            iced::widget::Space::new(Length::Fill, 0),
            conn_bar(conn, on_copy_uri, palette),
            iced::widget::Space::new(8, 0),
            mcp_button(mcp_server, on_mcp_toggle, palette),
            iced::widget::Space::new(8, 0),
            capture_indicator(capturing, on_capture_toggle, palette),
            iced::widget::Space::new(8, 0),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center)
        .height(36),
    )
    .width(Length::Fill)
    .height(36)
    .padding(Padding {
        top: 0.0,
        bottom: 0.0,
        left: 0.0,
        right: 8.0,
    })
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    })
    .into()
}
