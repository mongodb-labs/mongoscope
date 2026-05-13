use crate::{theme::Palette, ui::feed::filter::parser::Preset};
use iced::{
    widget::{button, column, row, text},
    Border, Element, Length, Padding,
};

#[derive(Debug, Clone)]
pub enum FilterPanelMsg {
    Toggle(Preset),
}

pub fn filters_panel<Msg: Clone + 'static>(
    active: Option<Preset>,
    on_msg: impl Fn(FilterPanelMsg) -> Msg + 'static + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg0 = palette.bg;
    let bg_hover = palette.bg_hover;
    let fg = palette.fg;
    let fg_dim2 = palette.fg_dim2;
    let accent = palette.accent;

    let rows: Vec<Element<Msg>> = Preset::all()
        .iter()
        .map(|&preset| {
            let is_active = active == Some(preset);
            let star_color = if is_active { accent } else { fg_dim2 };
            let label_color = if is_active { fg } else { fg_dim2 };

            button(
                row![
                    text("★")
                        .size(11)
                        .color(star_color)
                        .font(iced::Font::MONOSPACE),
                    text(preset.label())
                        .size(11)
                        .color(label_color)
                        .font(iced::Font::MONOSPACE),
                ]
                .spacing(5)
                .align_y(iced::Alignment::Center),
            )
            .padding(Padding {
                top: 5.0,
                bottom: 5.0,
                left: 8.0,
                right: 8.0,
            })
            .width(Length::Fill)
            .on_press(on_msg(FilterPanelMsg::Toggle(preset)))
            .style(move |_, status| button::Style {
                background: Some(iced::Background::Color(match status {
                    iced::widget::button::Status::Hovered => bg_hover,
                    _ => bg0,
                })),
                border: Border::default(),
                ..Default::default()
            })
            .into()
        })
        .collect();

    column(rows)
        .spacing(2)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 0.0,
            right: 0.0,
        })
        .into()
}
