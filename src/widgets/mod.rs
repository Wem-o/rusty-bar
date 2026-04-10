use iced::{Color, widget::Canvas};

mod circular_indicator;
use circular_indicator::CircularIndicator;

pub mod graph;

// Animated play / pause button
mod play_pause;
use play_pause::AnimatedPlayPause;

/// A play pause button that takes a number, as the
/// progress of the animation, and animates between and
/// a play and a pause button.
///
/// Worth putting it in a container with padding, as
/// it doesnt have padding internally!
pub fn play_pause_animated<T>(
    progress: f32,
    color: Color,
    size: f32,
) -> Canvas<AnimatedPlayPause, T> {
    Canvas::new(AnimatedPlayPause { progress, color })
        .width(size)
        .height(size)
}

pub fn circular_indicator<T>(
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
