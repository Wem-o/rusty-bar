use iced::{Color, widget::Canvas};

mod circular_indicator;
use circular_indicator::CircularIndicator;
pub mod graph;

pub fn circular_indicator<'a, T>(
    progress: f32,
    progress_color: Color,
    remaining_color: Color,
    line_width: f32,
    size: f32,
    gap: f32,
) -> Canvas<CircularIndicator, T> {
    Canvas::new(CircularIndicator {
        progress: progress,
        progress_color,
        remaining_color,
        line_width,
        gap,
    })
    .width(size)
    .height(size)
}
