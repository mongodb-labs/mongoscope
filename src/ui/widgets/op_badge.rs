use crate::{data::model::Op, theme::Palette};
use iced::{
    widget::{container, text},
    Border, Color, Element, Padding,
};

pub fn op_badge<Msg: 'static>(op: &Op, palette: &Palette) -> Element<'static, Msg> {
    let label = op.label();
    let color = op_color(op, palette);
    let bg = Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: 0.12,
    };
    let border_color = Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: 0.20,
    };

    container(
        text(label)
            .size(9.5)
            .color(color)
            .font(iced::Font::MONOSPACE),
    )
    .padding(Padding {
        top: 2.0,
        bottom: 2.0,
        left: 6.0,
        right: 6.0,
    })
    .style(move |_| container::Style {
        background: Some(iced::Background::Color(bg)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    })
    .into()
}

pub fn op_color(op: &Op, p: &Palette) -> Color {
    match op {
        Op::Find | Op::Aggregate | Op::CountDocuments => p.op_read,
        Op::InsertOne | Op::UpdateOne | Op::UpdateMany => p.op_write,
        Op::DeleteMany => p.op_delete,
        Op::Unknown(_) => p.fg_dim,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn unknown_op_uses_dim_color() {
        let p = Theme::Dark.palette();
        let color = op_color(&Op::Unknown("getMore".into()), &p);
        assert_eq!(color.r, p.fg_dim.r);
    }

    #[test]
    fn aggregate_is_read_color() {
        let p = Theme::Dark.palette();
        let color = op_color(&Op::Aggregate, &p);
        assert_eq!(color.r, p.op_read.r);
    }
}
