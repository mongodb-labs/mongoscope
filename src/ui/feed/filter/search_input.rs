use crate::{theme::Palette, ui::feed::filter::parser::FilterExpr};
use iced::{
    widget::{button, container, row, text, text_input},
    Border, Element, Length, Padding,
};

pub fn search_input<Msg: Clone + 'static>(
    value: String,
    placeholder: &'static str,
    on_change: impl Fn(String) -> Msg + 'static + Copy,
    on_submit: Msg,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg2 = palette.bg2;
    let fg = palette.fg;
    let fg_dim = palette.fg_dim;
    let fg_dim2 = palette.fg_dim2;
    let border = palette.border;
    let accent = palette.accent;
    let bg_sel = palette.bg_sel;

    let chips = FilterExpr::chip_tokens(&value);
    let remaining = FilterExpr::non_chip_text(&value);
    let chips_prefix = chips.join(" ");

    let chip_els: Vec<Element<'static, Msg>> = chips
        .into_iter()
        .map(|tok| {
            let value_clone = value.clone();
            let tok_label = tok.clone();
            button(
                row![
                    text(tok_label.clone())
                        .size(11)
                        .color(fg)
                        .font(iced::Font::MONOSPACE),
                    text("×").size(11).color(fg_dim).font(iced::Font::MONOSPACE),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 6.0,
                right: 6.0,
            })
            .on_press(on_change(FilterExpr::remove_token(&value_clone, &tok)))
            .style(move |_, _| button::Style {
                background: Some(iced::Background::Color(bg_sel)),
                border: Border {
                    color: accent,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .into()
        })
        .collect();

    let input_el = container(
        text_input(placeholder, &remaining)
            .size(12)
            .padding(Padding {
                top: 5.0,
                bottom: 5.0,
                left: 10.0,
                right: 10.0,
            })
            .on_input(move |new_remaining: String| {
                let new_full = if chips_prefix.is_empty() {
                    new_remaining
                } else if new_remaining.is_empty() {
                    chips_prefix.clone()
                } else {
                    format!("{} {}", chips_prefix, new_remaining)
                };
                on_change(new_full)
            })
            .on_submit(on_submit)
            .font(iced::Font::MONOSPACE)
            .style(move |_, _| text_input::Style {
                background: iced::Background::Color(bg2),
                border: Border {
                    color: border,
                    width: 1.0,
                    radius: 5.0.into(),
                },
                icon: fg,
                placeholder: fg_dim2,
                value: fg,
                selection: accent,
            }),
    )
    .width(Length::Fill);

    let mut contents: Vec<Element<'static, Msg>> = chip_els;
    contents.push(input_el.into());

    container(row(contents).spacing(4).align_y(iced::Alignment::Center))
        .width(Length::Fill)
        .into()
}
