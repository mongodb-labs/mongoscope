use iced::{widget::{button, row, text}, Border, Element};
use crate::{data::model::Op, theme::Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindFilter {
    All,
    Find,
    Aggregate,
    Write,
    Count,
    Unknown,
}

impl KindFilter {
    pub fn label(self) -> &'static str {
        match self {
            KindFilter::All       => "All",
            KindFilter::Find      => "Find",
            KindFilter::Aggregate => "Agg",
            KindFilter::Write     => "Write",
            KindFilter::Count     => "Count",
            KindFilter::Unknown   => "Unknown",
        }
    }

    pub fn all() -> &'static [KindFilter] {
        &[
            KindFilter::All,
            KindFilter::Find,
            KindFilter::Aggregate,
            KindFilter::Write,
            KindFilter::Count,
            KindFilter::Unknown,
        ]
    }

    pub fn matches(self, op: &Op) -> bool {
        match self {
            KindFilter::All => true,
            KindFilter::Find => matches!(op, Op::Find | Op::FindOne),
            KindFilter::Aggregate => matches!(op, Op::Aggregate),
            KindFilter::Write => matches!(op, Op::InsertOne | Op::UpdateOne | Op::UpdateMany | Op::DeleteOne | Op::DeleteMany),
            KindFilter::Count => matches!(op, Op::CountDocuments),
            KindFilter::Unknown => matches!(op, Op::Unknown(_)),
        }
    }
}

pub fn kind_chips<Msg: Clone + 'static>(
    active: KindFilter,
    on_select: impl Fn(KindFilter) -> Msg + Copy,
    palette: &Palette,
) -> Element<'static, Msg> {
    let bg_sel  = palette.bg_sel;
    let bg1     = palette.bg1;
    let fg      = palette.fg;
    let fg_dim  = palette.fg_dim;
    let accent  = palette.accent;
    let border  = palette.border;

    let chips: Vec<Element<Msg>> = KindFilter::all().iter().map(|&kind| {
        let is_active = kind == active;
        let bg = if is_active { bg_sel } else { bg1 };
        let fg_color = if is_active { fg } else { fg_dim };
        let border_color = if is_active { accent } else { border };

        button(
            text(kind.label()).size(11).color(fg_color).font(iced::Font::MONOSPACE)
        )
        .padding(iced::Padding { top: 3.0, bottom: 3.0, left: 8.0, right: 8.0 })
        .on_press(on_select(kind))
        .style(move |_, _| button::Style {
            background: Some(iced::Background::Color(bg)),
            border: Border { color: border_color, width: 1.0, radius: 12.0.into() },
            ..Default::default()
        })
        .into()
    }).collect();

    row(chips).spacing(4).into()
}
