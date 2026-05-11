use iced::{
    widget::canvas::{self, Canvas, Frame, Geometry, Path},
    Color, Element, Length, Rectangle, Size,
};
use crate::{theme::Palette, ui::feed::buckets::{BucketData, BUCKET_COUNT}};

pub struct DensityLane<'a> {
    buckets: &'a [BucketData; BUCKET_COUNT],
    max_total: u32,
    palette: Palette,
}

impl<'a> DensityLane<'a> {
    pub fn new(buckets: &'a [BucketData; BUCKET_COUNT], max_total: u32, palette: Palette) -> Self {
        Self { buckets, max_total, palette }
    }
}

impl<Msg> canvas::Program<Msg> for DensityLane<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Background
        frame.fill_rectangle(
            iced::Point::ORIGIN,
            bounds.size(),
            self.palette.bg2,
        );

        let w = bounds.width;
        let h = bounds.height;
        let bar_w = (w / BUCKET_COUNT as f32).max(1.0);

        for (i, b) in self.buckets.iter().enumerate() {
            let total = (b.ok + b.warn + b.slow) as f32;
            let max = self.max_total as f32;
            let full_h = (total / max) * h;

            let x = i as f32 * bar_w;

            // Stack slow / warn / ok bottom-to-top
            let slow_h = if b.slow > 0 { (b.slow as f32 / max) * h } else { 0.0 };
            let warn_h = if b.warn > 0 { (b.warn as f32 / max) * h } else { 0.0 };
            let ok_h = full_h - slow_h - warn_h;

            let mut y = h - full_h;

            if ok_h > 0.0 {
                frame.fill_rectangle(
                    iced::Point::new(x, y),
                    Size::new(bar_w - 1.0, ok_h),
                    self.palette.ok,
                );
                y += ok_h;
            }
            if warn_h > 0.0 {
                frame.fill_rectangle(
                    iced::Point::new(x, y),
                    Size::new(bar_w - 1.0, warn_h),
                    self.palette.warn,
                );
                y += warn_h;
            }
            if slow_h > 0.0 {
                frame.fill_rectangle(
                    iced::Point::new(x, y),
                    Size::new(bar_w - 1.0, slow_h),
                    self.palette.danger,
                );
            }
        }

        vec![frame.into_geometry()]
    }
}

pub fn density_lane<'a, Msg: 'static>(
    buckets: &'a [BucketData; BUCKET_COUNT],
    max_total: u32,
    palette: Palette,
) -> Element<'a, Msg> {
    Canvas::new(DensityLane::new(buckets, max_total, palette))
        .width(Length::Fill)
        .height(48)
        .into()
}
