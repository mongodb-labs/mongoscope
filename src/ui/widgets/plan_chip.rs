use crate::{data::model::Plan, theme::Palette};
use iced::{
    widget::{container, text},
    Border, Color, Element, Padding,
};

pub fn plan_chip<Msg: 'static>(plan: &Plan, palette: &Palette) -> Element<'static, Msg> {
    let label = plan.label();
    let color = plan_color(plan, palette);
    let bg = match plan {
        Plan::CollScan => Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 0.15,
        },
        _ => Color {
            r: color.r,
            g: color.g,
            b: color.b,
            a: 0.10,
        },
    };

    container(
        text(label)
            .size(9.5)
            .color(color)
            .font(iced::Font::MONOSPACE),
    )
    .padding(Padding {
        top: 1.0,
        bottom: 1.0,
        left: 5.0,
        right: 5.0,
    })
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            radius: 3.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

pub fn plan_color(plan: &Plan, p: &Palette) -> Color {
    match plan {
        Plan::CollScan => p.danger,
        Plan::IdHack => p.ok,
        Plan::IxScan(_) | Plan::IxScanLookup(_) => p.op_read,
        Plan::Unknown(_) => p.fg_dim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn collscan_is_danger() {
        let p = Theme::Dark.palette();
        assert_eq!(plan_color(&Plan::CollScan, &p).r, p.danger.r);
    }

    #[test]
    fn idhack_is_ok() {
        let p = Theme::Dark.palette();
        assert_eq!(plan_color(&Plan::IdHack, &p).r, p.ok.r);
    }
}
