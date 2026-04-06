use std::str::FromStr;

use iced::widget::text::LineHeight;
use iced::widget::{Button, button, scrollable, text};
use iced::{
    Alignment, Border, Color, Element, Length, Renderer, Task, Theme,
    border::{self, radius, rounded},
    widget::{
        Row,
        button::Status,
        column, container, row,
        scrollable::{Direction, Scrollbar},
        space, svg, tooltip,
    },
};
use iced_layershell::to_layer_message;

use crate::{
    get_svg,
    graph::{Series, graph},
};

/// The display mode of some data
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum DisplayMode {
    Graph,
    #[default]
    Cards,
}

/// The type of graph to be rendered
#[derive(Debug, Clone, Copy, PartialEq)]
enum GraphType {
    Temp,
    PrecProb,
    Prec,
    Wind,
}

fn get_button_color(theme: &Theme, status: Status) -> Color {
    let palette = theme.extended_palette();

    use iced::widget::button::Status::*;
    match status {
        Active => palette.background.strong.color,
        Hovered => palette.primary.strong.color,
        _ => palette.background.weak.color,
    }
}

/// Builds a new button with a svg to the given parameters
fn svg_button_builder<'a>(
    svg_handle: svg::Handle,
    button_size: f32,
    svg_size: f32,
    style: impl Fn(&Theme, Status) -> button::Style + 'a,
    message: crate::Message,
) -> Button<'a, crate::Message, Theme, Renderer> {
    button(svg(svg_handle).height(svg_size).width(svg_size))
        .padding(0.0)
        .height(button_size)
        .width(button_size)
        .on_press(message)
        .style(style)
}

/// This is only intended to be used internally
#[to_layer_message(multi)]
#[derive(Debug, Clone)]
#[allow(private_interfaces)]
pub enum Message {
    DisplayModeChange(DisplayMode),
    GraphTypeChange(Option<GraphType>),
}

#[derive(Debug, Default)]
pub struct State {
    display_mode: DisplayMode,
    graph_type: Option<GraphType>,
}

impl State {
    pub fn update(&mut self, message: Message) -> Task<crate::Message> {
        match message {
            Message::DisplayModeChange(mode) => {
                // println!("DisplayModeChanged to {:#?}", mode);
                self.display_mode = mode;
                Task::none()
            }
            Message::GraphTypeChange(graph_type) => {
                // println!("{:?}", graph_type);
                self.graph_type = graph_type;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view<'a>(&'a self, state: &'a crate::State) -> Element<'a, crate::Message> {
        let padding = 5;

        // Common svgs
        let wind = svg::Handle::from_memory(get_svg("weather", "wind").as_bytes());

        let droplet = svg::Handle::from_memory(get_svg("weather", "droplet").as_bytes());

        let humidity = svg::Handle::from_memory(get_svg("weather", "humidity").as_bytes());

        let refresh = svg::Handle::from_memory(get_svg("commons", "refresh").as_bytes());

        let temp_svg = svg::Handle::from_memory(get_svg("weather", "temperature").as_bytes());

        // Current weather
        let current_height = 120;
        let current = {
            container(match &state.weather_current {
                Some(weather) => {
                    let code_svg_handle = svg::Handle::from_memory(
                        get_svg(
                            if weather.is_day.unwrap() {
                                "day"
                            } else {
                                "night"
                            },
                            weather.code.as_ref().unwrap().get_svg_name().as_str(),
                        )
                        .as_bytes(),
                    );

                    let code_string_size =
                        if weather.code.as_ref().unwrap().to_string().chars().count() < 15 {
                            32
                        } else if weather.code.as_ref().unwrap().to_string().chars().count() > 23 {
                            28
                        } else {
                            24
                        };

                    container(
                        row![
                            svg(code_svg_handle)
                                .height(current_height - 2 * padding)
                                .width(current_height - 2 * padding)
                                .content_fit(iced::ContentFit::Fill),
                            column![
                                row![
                                    column![
                                        text(weather.temperature.as_ref().unwrap().stringify())
                                            .size(60)
                                            .style(text::primary)
                                            .align_y(Alignment::Start)
                                            .line_height(LineHeight::Relative(0.8)),
                                        text(format!(
                                            "Feels like {}",
                                            weather
                                                .apparent_temperature
                                                .as_ref()
                                                .unwrap()
                                                .stringify()
                                        ))
                                        .size(18)
                                        .style(text::secondary)
                                        .line_height(LineHeight::Relative(0.9)),
                                    ],
                                    space::horizontal(),
                                    column![
                                        row![
                                            text(
                                                weather
                                                    .precipitation
                                                    .as_ref()
                                                    .unwrap()
                                                    .combined_to_string()
                                            )
                                            .size(18)
                                            .style(text::primary)
                                            .align_y(Alignment::Center),
                                            svg(droplet.clone()).width(18).height(18)
                                        ],
                                        row![
                                            text(weather.humidity.as_ref().unwrap().stringify())
                                                .size(18)
                                                .style(text::primary)
                                                .align_y(Alignment::Center),
                                            svg(humidity.clone()).width(18).height(18)
                                        ],
                                        row![
                                            text(weather.wind.as_ref().unwrap().stringify())
                                                .size(18)
                                                .style(text::primary)
                                                .align_y(Alignment::Center),
                                            svg(wind.clone()).width(18).height(18)
                                        ],
                                    ]
                                    .align_x(Alignment::End)
                                ],
                                space::vertical(),
                                // Lower text
                                text(weather.code.as_ref().unwrap().to_string())
                                    .size(code_string_size)
                                    .style(text::primary)
                            ]
                            .width(Length::Fill)
                            .height(Length::Fill)
                        ]
                        .align_y(Alignment::Start)
                        .spacing(5),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Start)
                    .align_y(Alignment::Center)
                }
                None => container(
                    button(
                        column![
                            text("Click here to parse current weather"),
                            svg(svg::Handle::from_memory(
                                get_svg("commons", "refresh").as_bytes()
                            ))
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                        .width(Length::Fill)
                        .height(Length::Fill),
                    )
                    .padding(0)
                    .width(Length::Fill)
                    .height(Length::Fill),
                )
                .width(Length::Fill)
                .height(Length::Fill),
            })
            .width(Length::Fill)
            .height(current_height)
            .align_left(Length::Fill)
            .padding(padding as u16)
            .style(|theme: &Theme| {
                container::Style::default()
                    .background(theme.extended_palette().background.strong.color)
                    .border(rounded(state.radius))
            })
        };

        // Hourly weather
        let navbar_height = 25;
        let navbar = {
            let mut elements: Vec<Element<'_, crate::Message, Theme, Renderer>> = vec![
                // Cards
                {
                    button(text("Cards").center().size(20).style(|theme: &Theme| {
                        let palette = theme.extended_palette();

                        text::Style {
                            color: palette.primary.strong.text.into(),
                        }
                    }))
                    .width(55)
                    .height(navbar_height)
                    .style(|theme: &Theme, _status: button::Status| {
                        let palette = theme.extended_palette();

                        button::Style {
                            background: if self.display_mode == DisplayMode::Cards {
                                Some(palette.primary.strong.color.into())
                            } else {
                                Some(palette.secondary.base.color.into())
                            },
                            border: iced::Border {
                                width: 5.0,
                                radius: border::Radius {
                                    top_left: state.radius as f32,
                                    top_right: 0.0,
                                    bottom_right: 0.0,
                                    bottom_left: state.radius as f32,
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .on_press_maybe({
                        if self.display_mode != DisplayMode::Cards {
                            Some(crate::Message::WeatherWindowMessage(
                                Message::DisplayModeChange(DisplayMode::Cards),
                            ))
                        } else {
                            None
                        }
                    })
                    .into()
                },
                // Graph
                {
                    button(text("Graph").center().size(20).style(|theme: &Theme| {
                        let palette = theme.extended_palette();

                        text::Style {
                            color: palette.primary.strong.text.into(),
                        }
                    }))
                    .width(60)
                    .height(navbar_height)
                    .style(|theme: &Theme, _status: button::Status| {
                        let palette = theme.extended_palette();

                        button::Style {
                            background: if self.display_mode == DisplayMode::Graph {
                                Some(palette.primary.strong.color.into())
                            } else {
                                Some(palette.secondary.base.color.into())
                            },
                            border: iced::Border {
                                width: 5.0,
                                radius: border::Radius {
                                    top_left: 0.0,
                                    top_right: state.radius as f32,
                                    bottom_right: state.radius as f32,
                                    bottom_left: 0.0,
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .on_press_maybe({
                        if self.display_mode != DisplayMode::Graph {
                            Some(crate::Message::WeatherWindowMessage(
                                Message::DisplayModeChange(DisplayMode::Graph),
                            ))
                        } else {
                            None
                        }
                    })
                    .into()
                },
                space::horizontal().into(),
            ];

            if self.display_mode == DisplayMode::Graph {
                let middle_button_style = move |theme: &Theme, status: Status| button::Style {
                    background: Some(get_button_color(theme, status).into()),
                    ..Default::default()
                };

                let right_button_style = move |theme: &Theme, status: Status| button::Style {
                    background: Some(get_button_color(theme, status).into()),
                    border: Border {
                        radius: border::Radius::new(0.0).right(state.radius as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let left_button_style = move |theme: &Theme, status: Status| button::Style {
                    background: Some(get_button_color(theme, status).into()),
                    border: Border {
                        radius: border::Radius::new(0.0).left(state.radius as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                elements.push(
                    container(match self.graph_type {
                        None => {
                            row![
                                // Temp
                                svg_button_builder(
                                    temp_svg.clone(),
                                    navbar_height as f32,
                                    (navbar_height - 4) as f32,
                                    left_button_style,
                                    crate::Message::WeatherWindowMessage(Message::GraphTypeChange(
                                        Some(GraphType::Temp)
                                    ))
                                ),
                                // PrecProb
                                svg_button_builder(
                                    humidity.clone(),
                                    navbar_height as f32,
                                    (navbar_height - 4) as f32,
                                    middle_button_style,
                                    crate::Message::WeatherWindowMessage(Message::GraphTypeChange(
                                        Some(GraphType::PrecProb)
                                    ))
                                ),
                                // Prec
                                svg_button_builder(
                                    droplet.clone(),
                                    navbar_height as f32,
                                    (navbar_height - 4) as f32,
                                    middle_button_style,
                                    crate::Message::WeatherWindowMessage(Message::GraphTypeChange(
                                        Some(GraphType::Prec)
                                    ))
                                ),
                                // Wind
                                svg_button_builder(
                                    wind.clone(),
                                    navbar_height as f32,
                                    (navbar_height - 4) as f32,
                                    right_button_style,
                                    crate::Message::WeatherWindowMessage(Message::GraphTypeChange(
                                        Some(GraphType::Wind)
                                    ))
                                )
                            ]
                            .align_y(Alignment::Center)
                            .width(Length::Fill)
                            .height(Length::Fill)
                        }
                        Some(graph_type) => {
                            let mut layout = Vec::new();

                            let back = svg_button_builder(
                                svg::Handle::from_memory(get_svg("commons", "back").as_bytes()),
                                navbar_height as f32,
                                (navbar_height - 4) as f32,
                                right_button_style,
                                crate::Message::WeatherWindowMessage(Message::GraphTypeChange(
                                    None,
                                )),
                            );
                            use GraphType::*;
                            match graph_type {
                                Temp => {
                                    // temp
                                    layout.push(
                                        tooltip(
                                            svg_button_builder(
                                                svg::Handle::from_memory(
                                                    get_svg("weather", "temperature").as_bytes(),
                                                ),
                                                navbar_height as f32,
                                                (navbar_height - 6) as f32,
                                                |theme: &Theme, status: Status| {
                                                    let palette = theme.extended_palette();

                                                    button::Style {
                                                        background: Some(
                                                            if status == Status::Hovered {
                                                                palette.success.strong.color
                                                            } else {
                                                                palette.success.base.color
                                                            }
                                                            .into(),
                                                        ),
                                                        border: Border {
                                                            radius: radius(0)
                                                                .left(state.radius as f32),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }
                                                },
                                                crate::Message::Nothing,
                                            ),
                                            "Temperature",
                                            tooltip::Position::Bottom,
                                        )
                                        .into(),
                                    );

                                    // apparent temperature
                                    layout.push(
                                        tooltip(
                                            svg_button_builder(
                                                svg::Handle::from_memory(
                                                    get_svg("weather", "apparent_temperature")
                                                        .as_bytes(),
                                                ),
                                                navbar_height as f32,
                                                (navbar_height - 6) as f32,
                                                |theme: &Theme, status: Status| {
                                                    let palette = theme.extended_palette();

                                                    button::Style {
                                                        background: Some(
                                                            if status == Status::Hovered {
                                                                palette.warning.strong.color
                                                            } else {
                                                                palette.warning.base.color
                                                            }
                                                            .into(),
                                                        ),
                                                        border: Border {
                                                            radius: radius(0),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }
                                                },
                                                crate::Message::Nothing,
                                            ),
                                            "Apparent temperature",
                                            tooltip::Position::Bottom,
                                        )
                                        .into(),
                                    );
                                    layout.push(back.into());
                                }
                                Prec => {
                                    // Combined
                                    layout.push(
                                        tooltip(
                                            svg_button_builder(
                                                svg::Handle::from_memory(
                                                    get_svg("prec", "combined").as_bytes(),
                                                ),
                                                navbar_height as f32,
                                                (navbar_height - 8) as f32,
                                                |theme: &Theme, status: Status| {
                                                    let palette = theme.extended_palette();

                                                    button::Style {
                                                        background: Some(
                                                            if status == Status::Hovered {
                                                                palette.success.strong.color
                                                            } else {
                                                                palette.success.base.color
                                                            }
                                                            .into(),
                                                        ),
                                                        border: Border {
                                                            radius: radius(0)
                                                                .left(state.radius as f32),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }
                                                },
                                                crate::Message::Nothing,
                                            ),
                                            "Combined",
                                            tooltip::Position::Bottom,
                                        )
                                        .into(),
                                    );

                                    // Rain
                                    layout.push(
                                        tooltip(
                                            svg_button_builder(
                                                svg::Handle::from_memory(
                                                    get_svg("prec", "rain").as_bytes(),
                                                ),
                                                navbar_height as f32,
                                                (navbar_height - 8) as f32,
                                                |theme: &Theme, status: Status| {
                                                    let palette = theme.extended_palette();

                                                    button::Style {
                                                        background: Some(
                                                            if status == Status::Hovered {
                                                                palette.warning.strong.color
                                                            } else {
                                                                palette.warning.base.color
                                                            }
                                                            .into(),
                                                        ),
                                                        border: Border {
                                                            radius: radius(0),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }
                                                },
                                                crate::Message::Nothing,
                                            ),
                                            "Rain",
                                            tooltip::Position::Bottom,
                                        )
                                        .into(),
                                    );

                                    // Showers
                                    layout.push(
                                        tooltip(
                                            svg_button_builder(
                                                svg::Handle::from_memory(
                                                    get_svg("prec", "showers").as_bytes(),
                                                ),
                                                navbar_height as f32,
                                                (navbar_height - 8) as f32,
                                                |theme: &Theme, status: Status| {
                                                    let palette = theme.extended_palette();

                                                    button::Style {
                                                        background: Some(
                                                            if status == Status::Hovered {
                                                                palette.danger.strong.color
                                                            } else {
                                                                palette.danger.base.color
                                                            }
                                                            .into(),
                                                        ),
                                                        border: Border {
                                                            radius: radius(0),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }
                                                },
                                                crate::Message::Nothing,
                                            ),
                                            "Showers",
                                            tooltip::Position::Bottom,
                                        )
                                        .into(),
                                    );

                                    // Snowfalls
                                    layout.push(
                                        tooltip(
                                            svg_button_builder(
                                                svg::Handle::from_memory(
                                                    get_svg("prec", "snow").as_bytes(),
                                                ),
                                                navbar_height as f32,
                                                (navbar_height - 8) as f32,
                                                |theme: &Theme, status: Status| {
                                                    let palette = theme.extended_palette();

                                                    button::Style {
                                                        background: Some(
                                                            if status == Status::Hovered {
                                                                palette.primary.strong.color
                                                            } else {
                                                                palette.primary.base.color
                                                            }
                                                            .into(),
                                                        ),
                                                        border: Border {
                                                            radius: radius(0),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    }
                                                },
                                                crate::Message::Nothing,
                                            ),
                                            "Snow",
                                            tooltip::Position::Bottom,
                                        )
                                        .into(),
                                    );

                                    layout.push(back.into());
                                }
                                PrecProb | Wind => layout.push(
                                    svg_button_builder(
                                        svg::Handle::from_memory(
                                            get_svg("commons", "back").as_bytes(),
                                        ),
                                        navbar_height as f32,
                                        (navbar_height - 4) as f32,
                                        |theme: &Theme, status: Status| button::Style {
                                            background: Some(
                                                get_button_color(theme, status).into(),
                                            ),
                                            border: rounded(state.radius),
                                            ..Default::default()
                                        },
                                        crate::Message::WeatherWindowMessage(
                                            Message::GraphTypeChange(None),
                                        ),
                                    )
                                    .into(),
                                ),
                            }
                            Row::from_vec(layout)
                                .height(navbar_height)
                                .width(Length::Shrink)
                                .align_y(Alignment::Center)
                        }
                    })
                    .height(25)
                    .width(Length::Shrink)
                    .into(),
                )
            }

            container(
                Row::from_vec(elements)
                    .height(navbar_height)
                    .width(Length::Fill),
            )
        };

        let hourly_height = 150;
        let hourly_body: container::Container<'_, crate::Message, Theme, Renderer> = {
            container(if state.weather_hourly.is_empty() {
                let row_thingy: Element<'_, crate::Message, Theme, Renderer> = row![
                    space::horizontal(),
                    text("Refresh hourly weather data")
                        .center()
                        .style(text::primary),
                    svg(refresh.clone()).width(20).height(20),
                    space::horizontal()
                ]
                .spacing(5)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(Alignment::Center)
                .into();

                button(row_thingy)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .on_press(crate::Message::ParseHourlyWeather)
                    .style(|_, _| button::Style {
                        background: Some(Color::TRANSPARENT.into()),
                        ..Default::default()
                    })
                    .into()
            } else {
                match self.display_mode {
                    DisplayMode::Graph => match self.graph_type {
                        None => {
                            let select_type: Element<'_, crate::Message> = {
                                container(
                                    text("Please select a graph type from above")
                                        .align_x(Alignment::Center)
                                        .align_y(Alignment::Center)
                                        .style(text::primary),
                                )
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center)
                                .into()
                            };
                            select_type
                        }
                        Some(graph_type) => {
                            let mut series: Vec<Series> = Vec::new();

                            let mut min: Option<f32> = None;
                            let mut max: Option<f32> = None;
                            let mut steps: Option<i32> = None;

                            use GraphType::*;
                            match graph_type {
                                Temp => {
                                    series.push(Series::evenly_distribute(
                                        state
                                            .theme
                                            .as_ref()
                                            .unwrap()
                                            .extended_palette()
                                            .success
                                            .base
                                            .color,
                                        state
                                            .weather_hourly
                                            .iter()
                                            .map(|hour| hour.temperature.as_ref().unwrap().temp)
                                            .collect(),
                                    ));

                                    series.push(Series::evenly_distribute(
                                        state
                                            .theme
                                            .as_ref()
                                            .unwrap()
                                            .extended_palette()
                                            .warning
                                            .base
                                            .color,
                                        state
                                            .weather_hourly
                                            .iter()
                                            .map(|hour| {
                                                hour.apparent_temperature.as_ref().unwrap().temp
                                            })
                                            .collect(),
                                    ));
                                }

                                PrecProb => {
                                    min = Some(0.0);
                                    max = Some(100.0);
                                    steps = Some(20);

                                    series.push(Series::evenly_distribute(
                                        state
                                            .theme
                                            .as_ref()
                                            .unwrap()
                                            .extended_palette()
                                            .success
                                            .base
                                            .color,
                                        state
                                            .weather_hourly
                                            .iter()
                                            .map(|hour| {
                                                hour.precipitation
                                                    .as_ref()
                                                    .unwrap()
                                                    .probability
                                                    .unwrap()
                                            })
                                            .collect(),
                                    ))
                                }

                                Prec => {
                                    series.push(Series::evenly_distribute(
                                        state
                                            .theme
                                            .as_ref()
                                            .unwrap()
                                            .extended_palette()
                                            .success
                                            .base
                                            .color,
                                        state
                                            .weather_hourly
                                            .iter()
                                            .map(|hour| {
                                                hour.precipitation
                                                    .as_ref()
                                                    .unwrap()
                                                    .combined
                                                    .unwrap()
                                            })
                                            .collect(),
                                    ));

                                    series.push(Series::evenly_distribute(
                                        state
                                            .theme
                                            .as_ref()
                                            .unwrap()
                                            .extended_palette()
                                            .warning
                                            .base
                                            .color,
                                        state
                                            .weather_hourly
                                            .iter()
                                            .map(|hour| {
                                                hour.precipitation.as_ref().unwrap().rain.unwrap()
                                            })
                                            .collect(),
                                    ));

                                    series.push(Series::evenly_distribute(
                                        state
                                            .theme
                                            .as_ref()
                                            .unwrap()
                                            .extended_palette()
                                            .danger
                                            .base
                                            .color,
                                        state
                                            .weather_hourly
                                            .iter()
                                            .map(|hour| {
                                                hour.precipitation
                                                    .as_ref()
                                                    .unwrap()
                                                    .showers
                                                    .unwrap()
                                            })
                                            .collect(),
                                    ));

                                    series.push(Series::evenly_distribute(
                                        state
                                            .theme
                                            .as_ref()
                                            .unwrap()
                                            .extended_palette()
                                            .primary
                                            .base
                                            .color,
                                        state
                                            .weather_hourly
                                            .iter()
                                            .map(|hour| {
                                                hour.precipitation
                                                    .as_ref()
                                                    .unwrap()
                                                    .snowfall
                                                    .unwrap()
                                            })
                                            .collect(),
                                    ));
                                }

                                Wind => {
                                    min = Some(0.0);

                                    series.push(Series::evenly_distribute(
                                        state
                                            .theme
                                            .as_ref()
                                            .unwrap()
                                            .extended_palette()
                                            .success
                                            .base
                                            .color,
                                        state
                                            .weather_hourly
                                            .iter()
                                            .map(|hour| hour.wind.as_ref().unwrap().speed.unwrap())
                                            .collect(),
                                    ))
                                }
                            }

                            let labels = state
                                .weather_hourly
                                .iter()
                                .step_by(2)
                                .map(|hour| hour.time.format("%H").to_string())
                                .collect::<Vec<String>>();

                            graph(
                                state
                                    .theme
                                    .as_ref()
                                    .unwrap()
                                    .extended_palette()
                                    .background
                                    .strongest
                                    .color,
                                2.0,
                                15.0,
                                Color::from_str("#cba6f7").unwrap(),
                                labels,
                                35.0,
                                series,
                                min,
                                max,
                                steps,
                                3.0,
                                None,
                            )
                            .width(Length::Fill)
                            .height(hourly_height as f32)
                            .into()
                        }
                    },
                    DisplayMode::Cards => {
                        let mut cards = Vec::new();

                        for hour in &state.weather_hourly {
                            let code_svg_handle = svg::Handle::from_memory(
                                get_svg(
                                    if hour.is_day.unwrap() { "day" } else { "night" },
                                    hour.code.as_ref().unwrap().get_svg_name().as_str(),
                                )
                                .as_bytes(),
                            );

                            cards.push(
                                container(
                                    column![
                                        text(hour.time.format("%H:00").to_string())
                                            .align_x(Alignment::Center)
                                            .style(text::secondary)
                                            .size(12),
                                        svg(code_svg_handle).width(58).height(58),
                                        text(hour.temperature.as_ref().unwrap().stringify())
                                            .align_x(Alignment::Center)
                                            .style(text::primary)
                                            .line_height(LineHeight::Relative(0.95))
                                            .size(18),
                                        row![
                                            svg(droplet.clone()).width(10).height(10),
                                            text(
                                                hour.precipitation
                                                    .as_ref()
                                                    .unwrap()
                                                    .combined_to_string()
                                            )
                                            .align_x(Alignment::Center)
                                            .style(text::secondary)
                                            .line_height(LineHeight::Relative(0.9))
                                            .size(12)
                                        ],
                                        row![
                                            svg(wind.clone()).width(10).height(10),
                                            text(hour.wind.as_ref().unwrap().speed_stringify())
                                                .align_x(Alignment::Center)
                                                .style(text::secondary)
                                                .line_height(LineHeight::Relative(0.9))
                                                .size(12)
                                        ]
                                    ]
                                    .align_x(Alignment::Center)
                                    .spacing(2),
                                )
                                .width(65)
                                .height(Length::Fill)
                                .align_x(Alignment::Center)
                                .style(|theme: &Theme| {
                                    container::Style::default()
                                        .background(
                                            theme.extended_palette().background.strong.color,
                                        )
                                        .border(border::rounded(state.radius))
                                })
                                .into(),
                            );
                        }

                        scrollable(
                            Row::from_vec(cards)
                                .height(hourly_height)
                                .spacing(state.spacing),
                        )
                        .height(Length::Fill)
                        .width(Length::Fill)
                        .auto_scroll(true)
                        .direction(Direction::Horizontal(Scrollbar::new().spacing(5)))
                        .into()
                    }
                }
            })
            .height(hourly_height)
            .width(Length::Fill)
        };

        let bottom_bar_height = 20;
        let bottom_bar = {
            container(
                row![
                    button(
                        svg(svg::Handle::from_memory(
                            get_svg("commons", "refresh").as_bytes()
                        ))
                        .width(bottom_bar_height)
                        .height(bottom_bar_height)
                    )
                    .padding(0)
                    .width(bottom_bar_height)
                    .height(bottom_bar_height)
                    .style(button::primary)
                    .on_press(crate::Message::ParseWeather),
                    space::horizontal(),
                    text(match &state.weather_current {
                        Some(weather) => format!(
                            "Parsed for: {}; {}",
                            weather.coordinates.lng, weather.coordinates.lat
                        ),
                        None => String::new(),
                    }) // Maybe add location settings and stuff?
                ]
                .align_y(Alignment::Center)
                .width(Length::Fill)
                .height(bottom_bar_height),
            )
            .width(Length::Fill)
            .height(bottom_bar_height)
        };

        container(column![current, navbar, hourly_body, space::vertical(), bottom_bar].spacing(10))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(5)
            .style(|theme: &Theme| {
                container::Style::default()
                    .background(theme.extended_palette().background.weakest.color)
                    .border(border::rounded(state.radius))
            })
            .into()
    }
}
