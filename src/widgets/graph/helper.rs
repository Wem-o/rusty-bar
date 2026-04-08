use crate::widgets::graph::graph::*;
use core::f32;
use iced::{Color, Font, widget::Canvas};
use std::str::FromStr;

pub fn graph<'a>(
    scale_line_color: Color,
    scale_line_width: impl Into<f32>,
    scale_hmargin: impl Into<f32>,

    font_color: Color,

    // The labels on the bottom
    labels: impl IntoIterator<Item = impl Into<String>>,
    label_height: impl Into<f32>,

    // The data displayed
    series: impl IntoIterator<Item = impl Into<Series>>,
    min: Option<f32>,
    max: Option<f32>,
    value_steps: Option<i32>,
    serires_line_width: impl Into<f32>,

    hover_color: Option<Color>,
) -> Canvas<Graph, crate::Message> {
    let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
    let series: Vec<Series> = series.into_iter().map(Into::into).collect();

    let min: f32 = match min {
        Some(v) => v.into(),
        None => {
            let overal_min = series
                .iter()
                .flat_map(|s| s.values.iter().map(|(val, _)| *val))
                .fold(f32::INFINITY, |acc, val| acc.min(val));

            if overal_min.is_infinite() || overal_min.is_nan() {
                0.0
            } else {
                overal_min - 1.0
            }
            .round()
        }
    };

    let max: f32 = match max {
        Some(v) => {
            let max = v.into();
            if max <= min { min + 1.0 } else { max }
        }
        None => {
            let overal_max = series
                .iter()
                .flat_map(|s| s.values.iter().map(|(val, _)| *val))
                .fold(f32::NEG_INFINITY, |max, val| max.max(val));

            if overal_max.is_infinite() || overal_max.is_nan() || overal_max <= min {
                min + 1.0
            } else {
                overal_max + 1.0
            }
            .round()
        }
    };

    let step = match value_steps {
        Some(v) => v.max(1),
        None => {
            let range = (max - min).abs() as i32;

            if range >= 25 {
                10
            } else if range >= 15 {
                5
            } else if range >= 6 {
                2
            } else {
                1
            }
        }
    };

    let graph = Graph {
        scale_line_color,
        scale_line_width: scale_line_width.into(),
        scale_hmargin: scale_hmargin.into(),
        top_padding: 0.0,

        labels,
        label_hmargin: 8.0,
        bottom_label_height: label_height.into(),

        series,

        font: Font::DEFAULT,
        font_color,

        max_value: max as f32,
        min_value: min as f32,

        value_steps: step,

        series_line_width: serires_line_width.into(),

        hover_radius: 9.0,
        hover_color: hover_color
            .unwrap_or(Color::from_str("#cba6f7").expect("Failed to convert '#cba6f7' to color")),
    };

    Canvas::new(graph)
}
