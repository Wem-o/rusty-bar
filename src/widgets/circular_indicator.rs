use std::{
    cell::Cell,
    f32::{self, consts::PI},
};

use iced::{
    Color, Radians, Rectangle, Renderer, mouse,
    widget::canvas::{self, Geometry, Path, Program, Stroke, path::Arc},
};

#[derive(Debug, Clone, Copy)]
pub struct CircularIndicator {
    /// The % of the progress, from 0.0 to 1.0
    pub progress: f32,
    /// The color of the progressed part
    pub progress_color: Color,
    /// The color of the remaining part
    pub remaining_color: Color,
    /// The width of the strokes
    pub line_width: f32,
    /// % of a radian (half a circle) to
    /// use as the gap
    pub gap: f32,
}

impl<Message> Program<Message> for CircularIndicator {
    type State = (canvas::Cache<Renderer>, Cell<f32>);

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let (cache, prev_value) = state;

        // Clear the cache if the value changed
        // otherwise do nothing
        if self.progress != prev_value.get() {
            cache.clear();
            prev_value.set(self.progress);
        }

        let geo = cache.draw(renderer, bounds.size(), |frame| {
            // Correction to start from the top, instead of the right
            let top: f32 = -PI / 2.0;

            // Progress end angle (without the top correction!!!)
            let progress_end = self.progress * 2.0 * PI;

            // Gap between progress and remaining lines
            let gap = if self.progress == 0.0 || self.progress == 100.0 {
                0.0
            } else {
                PI * self.gap
            };

            // The radius of the circle
            let radius = (bounds.height - self.line_width * 2.0) / 2.0;

            // println!(
            //     "Radius is {}, bounds are W: {}, H: {}, top left corner is x: {} y: {}, center is at: {}",
            //     radius,
            //     bounds.width,
            //     bounds.height,
            //     bounds.x,
            //     bounds.y,
            //     bounds.center()
            // );

            // Progress
            let progress = Path::new(|builder| {
                builder.arc(Arc {
                    center: frame.center(),
                    start_angle: Radians(top),
                    end_angle: Radians(progress_end + top),
                    radius: radius,
                });
            });
            frame.stroke(
                &progress,
                Stroke {
                    width: self.line_width,
                    style: self.progress_color.into(),
                    line_cap: canvas::LineCap::Round,
                    ..Default::default()
                },
            );

            let remaining_start_angle = Radians(progress_end + top + gap);
            let remaining_end_angle = Radians(PI * 2.0 + top - gap);

            if remaining_start_angle < remaining_end_angle {
                // Remaining
                let remaining = Path::new(|builder| {
                    builder.arc(Arc {
                        center: frame.center(),
                        start_angle: remaining_start_angle,
                        end_angle: remaining_end_angle,
                        radius: radius,
                    });
                });
                frame.stroke(
                    &remaining,
                    Stroke {
                        width: self.line_width,
                        style: self.remaining_color.into(),
                        line_cap: canvas::LineCap::Round,
                        ..Default::default()
                    },
                );
            }
        });

        vec![geo]
    }
}
