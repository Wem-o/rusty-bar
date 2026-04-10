use std::{
    cell::Cell,
    f32::{self},
};

use iced::{
    Color, Point, Rectangle, Renderer, Size, Theme, mouse,
    widget::canvas::{Cache, Geometry, Path, Program},
};

#[derive(Debug, Clone, Copy)]
/// A play pause button that takes a number, as the
/// progress of the animation, and animates between and
/// a play and a pause button.
///
/// Note: The animation is only correct if the aspect
/// ration of the widget it 1:1
pub struct AnimatedPlayPause {
    /// A percentage of how "paused" the object is
    ///
    /// 0 will be playing (triangle) and
    /// 1 will be paused (||)
    pub progress: f32,
    pub color: Color,
}
impl<Message> Program<Message> for AnimatedPlayPause {
    type State = (Cache<Renderer>, Cell<f32>);

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let (cache, prev_value) = state;

        // Clear cache if the value changed
        if self.progress != prev_value.get() {
            cache.clear();
            prev_value.set(self.progress);
        }

        let geo = cache.draw(renderer, bounds.size(), |frame| {
            let fill = iced::widget::canvas::Fill {
                style: iced::widget::canvas::Style::Solid(self.color),
                ..Default::default()
            };
            let width = frame.width();
            let height = frame.height();

            // Paused
            if self.progress >= 1.0 {
                let paused = Path::new(|builder| {
                    // Width of one of the vertical lines
                    let chunk_width = width * 0.35;

                    // The space between the two lines
                    let space_width = width * 0.3;

                    let chunk_size = Size {
                        height: height,
                        width: chunk_width,
                    };

                    builder.rectangle(Point::ORIGIN, chunk_size);

                    builder.rectangle(
                        Point {
                            x: chunk_width + space_width,
                            y: 0.0,
                        },
                        chunk_size,
                    );
                });

                frame.fill(&paused, fill);
            }
            // Playing
            else if self.progress <= 0.0 {
                let trianlge = Path::new(|builder| {
                    builder.move_to(Point::ORIGIN);
                    builder.line_to(Point {
                        x: width,
                        y: height / 2.0,
                    });

                    builder.line_to(Point { x: 0.0, y: height });

                    builder.line_to(Point::ORIGIN);
                });

                frame.fill(&trianlge, fill);
            }
            // Animating
            else {
                //  Left side  \\
                // The x of the two point at the top
                let left_x = (1.0 - self.progress * 0.3).clamp(0.7, 1.0) * (height / 2.0);

                let left_path = Path::new(|builder| {
                    builder.move_to(Point::ORIGIN);

                    // Top line
                    builder.line_to(Point {
                        x: left_x,
                        y: (1.0 - self.progress) * height * 0.25,
                    });

                    // Right line
                    builder.line_to(Point {
                        x: left_x,
                        y: (height * 0.75) + self.progress * height * 0.25,
                    });

                    // Bottom line
                    builder.line_to(Point { x: 0.0, y: height });

                    // Left line
                    builder.line_to(Point::ORIGIN);
                });
                frame.fill(&left_path, fill);

                //  Right side  \\
                // The progress of the two points at the top
                // They should be at their full positions
                // when the progress is 85%
                let right_triangle_progress = (self.progress * (1.0 / 0.5)).clamp(0.0, 1.0);
                // The x of the two point at the top
                let right_triangle_x = width / 2.0 + right_triangle_progress * (width * 0.15);

                let rectangle_x = width * 0.65 + ((self.progress - 0.5) * 2.0) * width * 0.35;

                let right_path = Path::new(|builder| {
                    let home = Point {
                        x: width,
                        y: height / 2.0,
                    };

                    // Right point
                    builder.move_to(home);

                    // Top of the rectangle
                    if self.progress >= 0.5 {
                        builder.line_to(Point {
                            x: rectangle_x,
                            y: 0.0,
                        });
                    }

                    // Top triangle
                    builder.line_to(Point {
                        x: right_triangle_x,
                        y: height * 0.25 - right_triangle_progress * height * 0.25,
                    });

                    // Bottom triangle
                    builder.line_to(Point {
                        x: right_triangle_x,
                        y: height * 0.75 + right_triangle_progress * height * 0.25,
                    });

                    // Bottom of the rectangle
                    if self.progress >= 0.5 {
                        builder.line_to(Point {
                            x: rectangle_x,
                            y: height,
                        });
                    }

                    builder.line_to(home);
                });
                frame.fill(&right_path, fill);
            }
        });

        vec![geo]
    }
}
