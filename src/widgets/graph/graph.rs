use iced::{
    Color, Font, Point, Rectangle, Renderer, mouse, widget::{
        canvas::{
            self, Frame, Geometry, Path, Program, Stroke 
        }, 
        text
    }
};

// How much the cursor can move before it's considered move
const CURSOR_THRESHOLD: f32 = 4.0;

/// A group of data to be displayed
#[derive(Debug, Clone)]
pub struct Series {
    pub color: Color,

    /// (value between min and max of a graph, horizontal position percentage)
    pub values: Vec<(f32, f32)>
}

impl Series {
    /// Evenly distribute a collection from 0% to 100%
    pub fn evenly_distribute(color: Color, values: Vec<impl Into<f32>>) -> Self {
        let values: Vec<f32> = values.into_iter().map(Into::into).collect::<Vec<f32>>();
        let step = 100.0 / (values.len() as f32 - 1.0).max(1.0);

        Self {
            color,
            values: {
                if values.is_empty() {
                    Vec::new()
                } else if values.len() == 1 {
                    vec![(values[0], 0.0)]
                } else {
                    values.iter().enumerate().map(|(id, value)| (value.clone(), id as f32 * step)).collect()
                }
            }
        }
    }
}

impl<T, I> From<T> for Series 
where 
    T: IntoIterator<Item = I>,
    I: Into<f32>
{
    fn from(value: T) -> Self {
        let values: Vec<f32> = value.into_iter().map(Into::into).collect();
        Self::evenly_distribute(Color::BLACK, values)
    }
}



/// The state of the cursor hover
#[derive(Debug, Default)]
pub struct HoverState {
    point: Option<Point>,
    
    values: Vec<(X, Vec<Y>)>,
    closest_index: Option<(X, Vec<Y>)>,
}

type X = f32;
type Y = f32;

/// A graph to display bunch of data
pub struct Graph {
    pub scale_hmargin: f32,
    pub scale_line_color: Color,
    pub scale_line_width: f32,

    /// Usually worth settings, as the top line may not render correctly without this
    pub top_padding: f32,
    /// Height of the labels on the bottom
    pub bottom_label_height: f32,
    /// The margin between the side of the graph, and first/last label on the bottom
    pub label_hmargin: f32,

    pub font: Font,
    pub font_color: Color,

    pub min_value: f32,
    pub max_value: f32,
    pub value_steps: i32,

    /// The labels at the bottom of the graph
    pub labels: Vec<String>,
    /// The series of data to be represented
    pub series: Vec<Series>,
    /// The width of the line representing the series
    pub series_line_width: f32,

    pub hover_radius: f32, 
    pub hover_color: Color
}

impl<Message> Program<Message> for Graph {
    type State = HoverState;

    fn update(
            &self,
            state: &mut Self::State,
            _event: &iced::Event,
            bounds: Rectangle,
            cursor: mouse::Cursor,
        ) -> Option<canvas::Action<Message>> {
        
        if let Some(point) = cursor.position_in(bounds) {
            if let Some(previous_point) = state.point {

                let range = (self.max_value - self.min_value) as i64;
                let vertical_step = (bounds.height - self.bottom_label_height - self.top_padding) / range as f32;
                let horizontal_area = bounds.width - self.scale_hmargin - self.label_hmargin * 2.0;

                if (point - previous_point).x.abs() >= CURSOR_THRESHOLD {
                    state.values.clear();
                    for serie in &self.series {
                        let markers: Vec<Point> = serie.values
                            .iter()
                            .map(|(value, horizontal_percentage)| {
                                Point::new(
                                    self.label_hmargin + self.scale_hmargin + ((horizontal_percentage / 100.0) * horizontal_area),
                                    (self.max_value - value) * vertical_step)
                            })
                            .collect();

                        for marker in markers {
                            match state.values.iter_mut().find(|value| {value.0 == marker.x}) {
                                Some(value) => value.1.push(marker.y),
                                None => {
                                    state.values.push((
                                        marker.x,
                                        vec![marker.y]
                                    ));
                                }
                            }
                        }
                    };
                    state.point = Some(point);

                    let closest = state.values
                        .iter()
                        .min_by(|a, b| {
                            let da = (point.x - a.0).abs();
                            let db = (point.x - b.0).abs();
                            da.partial_cmp(&db).unwrap()
                        });

                    state.closest_index = closest.cloned();
                    return Some(canvas::Action::request_redraw())
                };
            } else {
                state.point = Some(point);
            }
        }        
        None
    }

    fn draw(
            &self,
            state: &Self::State,
            renderer: &Renderer,
            _theme: &iced::Theme,
            bounds: Rectangle,
            _cursor: mouse::Cursor,
        ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());
        let range = (self.max_value - self.min_value).abs() as i32;

        // The "step" for 1 value on the range
        // This is used to keep the graph to the height of the graph
        let vertical_step = if range > 0 {
            (bounds.height - self.bottom_label_height - self.top_padding) / range as f32 
        } else {
            0.0
        };

        // The bottom of the graph area
        let graph_bottom_y = range as f32 * vertical_step + self.top_padding;
        
        // Numbers on the left side + the scale lines
        for value in (0..=range).step_by(self.value_steps as usize) {
            let y = value as f32 * vertical_step + self.top_padding;

            // Scale label
            let text = canvas::Text {
                content: (self.max_value - value as f32).to_string(),
                align_x: text::Alignment::Left,
                align_y: iced::alignment::Vertical::Center,
                position: Point::new(0.0, y),
                color: self.font_color,
                font: self.font,
                ..Default::default()
            };
            frame.fill_text(text);

            // Scale line
            frame.stroke(
                &Path::line(
                    Point::new(self.scale_hmargin, y),
                    Point::new(bounds.width, y)
                ),
                Stroke {
                    width: self.scale_line_width,
                    style: self.scale_line_color.into(),
                    ..Default::default()
                }
            );
        }

        // Draw the bottom line
        let y = range as f32 * vertical_step + self.top_padding;
        // Scale label
        let text = canvas::Text {
            content: (self.max_value - range as f32).to_string(),
            align_x: text::Alignment::Left,
            align_y: iced::alignment::Vertical::Center,
            position: Point::new(0.0, y),
            color: self.font_color,
            font: self.font,
            ..Default::default()
        };
        frame.fill_text(text);

        // Scale line
        frame.stroke(
            &Path::line(
                Point::new(self.scale_hmargin, y),
                Point::new(bounds.width, y)
            ),
            Stroke {
                width: self.scale_line_width,
                style: self.scale_line_color.into(),
                ..Default::default()
            }
        );


        // The horizontal area available to draw the values
        let horizontal_area = bounds.width - self.scale_hmargin - self.label_hmargin * 2.0;

        let position_count = self.labels.len();
        let horizontal_step = if position_count > 1 {
            let divisor = (position_count - 1) as f32;
            if divisor > 0.0 {
                horizontal_area / divisor
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Draw the labels
        for (id, label) in self.labels.iter().enumerate() {
            let x = (id as f32 * horizontal_step) + self.label_hmargin + self.scale_hmargin;

            let text = canvas::Text {
                content: label.to_string(),
                align_x: text::Alignment::Center,
                align_y: iced::alignment::Vertical::Top,
                position: Point::new(x, graph_bottom_y + 5.0),
                color: self.font_color,
                font: self.font,
                ..Default::default()
            };
            frame.fill_text(text);
        }

        // The the values
        for serie in &self.series {
            let markers: Vec<Point> = serie.values
                .iter()
                .map(|(value, horizontal_percentage)| {
                    Point::new(
                        self.label_hmargin + self.scale_hmargin + ((horizontal_percentage / 100.0) * horizontal_area),
                        (self.max_value - value) * vertical_step + self.top_padding)
                })
                .collect();

            if markers.len() >= 2 {
                let path = Path::new(|builder| {
                    builder.move_to(markers[0]);

                    for i in 0..markers.len() -1 {
                        let current = markers[i];
                        let next = markers[i + 1];

                        let control1 = Point::new(current.x + (next.x - current.x) * 0.3, current.y);
                        let control2 = Point::new(next.x - (next.x - current.x) * 0.3, next.y);

                        builder.bezier_curve_to(control1, control2, next);
                    }
                });

                frame.stroke(
                    &path, 
                    Stroke {
                        style: serie.color.into(),
                        width: self.series_line_width,
                        ..Default::default()
                    }
                );
            }
        }



        let mut hover_overlay = Frame::new(renderer, bounds.size());
        if let Some(point) = &state.closest_index {
            let x = &point.0;

            point.1
                .iter()
                .for_each(|y| {
                    hover_overlay.fill(
                        &Path::circle(Point::new(x.clone(), y.clone()), self.hover_radius),
                    self.hover_color
                    );
                });
        }

        vec![frame.into_geometry(), hover_overlay.into_geometry()]
    }
}