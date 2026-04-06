use futures::{SinkExt, StreamExt, stream::BoxStream};
use iced::{
    Alignment, Color, Element, Font, Length, Padding, Renderer, Subscription, Task,
    border::{self, Radius},
    stream,
    theme::{self, Theme},
    widget::{Container, Space, button::Status, container, mouse_area, row, space, svg, text},
    window,
};

use iced::widget::button;

use iced_layershell::{
    daemon,
    reexport::{Anchor, NewLayerShellSettings},
    settings::{LayerShellSettings, Settings, StartMode},
    to_layer_message,
};

use chrono::Local;
use mpris_client_async::{
    Mpris, Player,
    player_types::Playback,
    properties::{PlaybackStatus, Property},
    streams::{mpris::MprisEvent, player::PropertyYield},
};
use std::{collections::HashMap, sync::Arc, time::Duration};

// Weather backend
mod weather;
use crate::weather::{CurrentWeather, HourlyWeather};
use weather::prelude::*;

// The notification of rusty bar to the user (things like errrors, notices, and other messages)
mod notification;
use crate::notification::Notification;

// Contains svgs and other small assets
mod assets;
use crate::assets::get_svg;

mod windows;
use windows::weather_window;

mod graph;

#[derive(Debug, Clone)]
#[non_exhaustive]
enum WindowType {
    Main,
    Weather,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Message {
    /// Does absolutely nothing
    Nothing,

    //  INTERNAL STUFF   \\
    NewNotif(Notification),
    NotifRetry(Notification),

    SecondTrigger,

    //      WEATHER     \\
    ParseWeather,

    ParseCurrentWeather,
    CurrentWeatherParsed(Result<CurrentWeather, ParsingError>),

    ParseHourlyWeather,
    HourlyWeatherParsed(Result<Vec<HourlyWeather>, ParsingError>),

    WeatherWindowMessage(weather_window::Message),
    WeatherWindowToggle,

    //      MPRIS       \\
    MprisEvent(MprisEvent),
    PlayerPlaybackChange((Arc<Player>, Playback)),
}

#[derive(Default)]
struct State {
    //  GLOBAL VARS \\
    window_ids: HashMap<window::Id, WindowType>,
    theme: Option<Theme>,
    radius: i32,
    time_fmt: &'static str,
    spacing: u32,
    hpadding: u32,
    notifications: Vec<Notification>,

    //  LEFT SIDE   \\
    clock: String,
    clock_widget_width: u32, // TODO: ELLIMINATE!!

    // Parse initial values, it will only
    // be run once, when the program starts
    first_parse: bool,

    //      WEATHER      \\
    units: Units,
    weather_current: Option<CurrentWeather>,
    // None if the location should be parsed from ip address, not a specified position
    tracked_location: Option<Coordinates>,

    weather_hourly: Vec<HourlyWeather>,
    weather_hours_to_parse: Option<u8>,

    weather_window_id: Option<window::Id>,
    weather_window_state: weather_window::State,

    //      MPRIS      \\
    players: Vec<Arc<Player>>,
    tracked_player: Option<Arc<Player>>,
}

impl State {
    fn new(
        theme: Theme,
        radius: i32,
        spacing: u32,
        time_fmt: &'static str,
        clock_widget_width: u32,
        hpadding: u32,
        units: Units,
    ) -> Self {
        Self {
            theme: Some(theme),
            radius,
            time_fmt,
            spacing,
            clock_widget_width,
            hpadding,
            units,
            first_parse: true,
            ..Default::default()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        use Message::*;
        match message {
            Nothing => Task::none(),

            RemoveWindow(id) => {
                self.window_ids.remove(&id);
                println!("@ICED Removing window id {id}");
                Task::none()
            }

            NewNotif(notif) => {
                println!("@N New notification: {:#?}", notif);
                self.notifications.push(notif);
                Task::none()
            }
            NotifRetry(notif) => notif.retry().expect("{0}"),

            SecondTrigger => {
                let now = Local::now();
                self.clock = now.format(self.time_fmt).to_string();

                if self.first_parse {
                    self.first_parse = false;
                    Task::batch([Task::done(Message::ParseCurrentWeather)])
                } else {
                    Task::none()
                }
            }

            //      WEATHER     \\
            ParseWeather => Task::batch(vec![
                Task::done(Message::ParseCurrentWeather),
                Task::done(Message::ParseHourlyWeather),
            ]),

            ParseCurrentWeather => {
                println!("@W Parsing current weather");

                use argument::Current;
                Task::perform(
                    get_current(
                        self.tracked_location.clone(),
                        self.units.clone(),
                        vec![
                            Current::Temperature,
                            Current::IsDay,
                            Current::ApparentTemp,
                            Current::Humidity,
                            Current::WeatherCode,
                            Current::WindDirection,
                            Current::WindSpeed,
                            Current::Precipitation(argument::PrecipitationType::Combined),
                            Current::Precipitation(argument::PrecipitationType::Rain),
                            Current::Precipitation(argument::PrecipitationType::Showers),
                            Current::Precipitation(argument::PrecipitationType::Snowfall),
                        ],
                    ),
                    Message::CurrentWeatherParsed,
                )
            }
            CurrentWeatherParsed(result) => match result {
                Ok(weather) => {
                    self.weather_current = Some(weather);
                    Task::none()
                }
                Err(e) => Task::done(NewNotif(Notification::new_with_retry(
                    notification::Level::Error,
                    e.to_string(),
                    Local::now(),
                    &Message::ParseCurrentWeather,
                ))),
            },

            ParseHourlyWeather => {
                let hours: u8 = self.weather_hours_to_parse.unwrap_or(24);
                println!("@W Parsing {} hours of hourly weather!", hours);

                use argument::Hourly;
                Task::perform(
                    get_hourly(
                        self.tracked_location.clone(),
                        self.units.clone(),
                        vec![
                            Hourly::WeatherCode,
                            Hourly::Temperature,
                            Hourly::ApparentTemp,
                            Hourly::IsDay,
                            Hourly::PrecipitationProbability,
                            Hourly::WindSpeed,
                            Hourly::Precipitation(argument::PrecipitationType::Combined),
                            Hourly::Precipitation(argument::PrecipitationType::Rain),
                            Hourly::Precipitation(argument::PrecipitationType::Showers),
                            Hourly::Precipitation(argument::PrecipitationType::Snowfall),
                        ],
                        hours,
                    ),
                    Message::HourlyWeatherParsed,
                )
            }
            HourlyWeatherParsed(result) => match result {
                Ok(result) => {
                    self.weather_hourly = result;
                    Task::none()
                }
                Err(e) => Task::done(NewNotif(Notification::new_with_retry(
                    notification::Level::Error,
                    e,
                    Local::now(),
                    &ParseHourlyWeather,
                ))),
            },

            WeatherWindowToggle => {
                if let Some(id) = self.weather_window_id {
                    // println!("Closing weather_window with id: {}", id);
                    self.weather_window_id = None;
                    window::close(id)
                } else {
                    let id = window::Id::unique();
                    self.window_ids.insert(id, WindowType::Weather);
                    self.weather_window_id = Some(id);

                    // println!("Opening new weather_window with id: {}", id);

                    let mut tasks = Vec::new();
                    // if self.weather_hourly.is_empty() {
                    //     println!("Opening weather popup, but hourly_weather is not parsed, requesting parsing!");
                    //     tasks.push(Task::done(ParseHourlyWeather));
                    // }

                    tasks.push(Task::done(Message::NewLayerShell {
                        settings: NewLayerShellSettings {
                            size: Some((450, 400)),
                            layer: iced_layershell::reexport::Layer::Top,
                            anchor: Anchor::Top | Anchor::Left,
                            margin: Some((10, 0, 0, 30)),
                            keyboard_interactivity:
                                iced_layershell::reexport::KeyboardInteractivity::OnDemand,
                            output_option: iced_layershell::reexport::OutputOption::None,
                            ..Default::default()
                        },
                        id,
                    }));

                    Task::batch(tasks)
                }
            }
            WeatherWindowMessage(msg) => self.weather_window_state.update(msg),

            //      MPRIS       \\
            MprisEvent(event) => {
                use mpris_client_async::streams::mpris::MprisEvent::*;
                match event {
                    Add(player) => {
                        println!("@M Player added with name \"{}\"", player.dbus_name());
                        self.players.push(player);

                        Task::none()
                    }
                    Remove(name) => {
                        println!("@M Player removed with name \"{}\"", name);
                        self.players.retain(|player| player.dbus_name() != name);

                        if let Some(player) = &self.tracked_player
                            && player.dbus_name() == name
                        {
                            println!("@M Tracked player removed => tracked player is None");
                            self.tracked_player = None
                        }

                        Task::none()
                    }
                    PropertyChange(any_change) => {
                        if let Some(playback) = any_change
                            .value
                            .downcast_ref::<PropertyYield<PlaybackStatus>>()
                        {
                            let player = self
                                .players
                                .iter()
                                .find(|other| other.dbus_name() == playback.player_name);

                            println!(
                                "@M Player {} is {}. Found object? {}",
                                playback.player_name,
                                playback.value,
                                player.is_some()
                            );

                            match player {
                                None => Task::none(),
                                Some(player) => Task::done(PlayerPlaybackChange((
                                    player.clone(),
                                    playback.value.clone(),
                                ))),
                            }
                        } else {
                            Task::none()
                        }
                    }
                    _ => {
                        println!("@@ MPRIS EVENT");
                        Task::none()
                    }
                }
            }
            PlayerPlaybackChange((player, playback)) => {
                println!(
                    "@M PlayerPlaybackChanged called, {}, {}",
                    player.dbus_name(),
                    playback
                );

                if playback != Playback::Stopped {
                    self.tracked_player = Some(player)
                }

                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn separator<'a>() -> Container<'a, Message, Theme, Renderer> {
        container(Space::new())
            .width(2)
            .height(30)
            .align_y(Alignment::Center)
            .align_x(Alignment::Center)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();

                container::Style::default().background(palette.background.weakest.color)
            })
    }

    /// Returns the type of the window, if it's not found in
    /// `self.window_ids` it's assumed to be Main.
    fn match_id(&self, id: &window::Id) -> &WindowType {
        match self.window_ids.get(&id) {
            Some(window) => window,
            None => &WindowType::Main,
        }
    }

    fn view(&self, id: window::Id) -> Element<'_, Message> {
        use WindowType::*;
        match *self.match_id(&id) {
            Main => self.main_view(),
            Weather => self.weather_window_state.view(&self),
        }
    }

    fn main_view(&self) -> Element<'_, Message> {
        let clock = {
            container(
                text(&self.clock)
                    .size(36)
                    .center()
                    .width(self.clock_widget_width)
                    .style(text::primary),
            )
            .padding(Padding::default().horizontal(self.hpadding))
            .width(Length::Shrink)
            .height(Length::Fill)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();

                container::Style::default()
                    .background(palette.background.weak.color)
                    .border(border::rounded(self.radius))
            })
        };

        let weather_widget: Element<'_, Message> = {
            match &self.weather_current {
                Some(weather) => {
                    let svg_handle = svg::Handle::from_memory(
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

                    container(
                        mouse_area(
                            row![
                                svg(svg_handle)
                                    .width(36)
                                    .height(36)
                                    .content_fit(iced::ContentFit::Fill),
                                text(weather.temperature.as_ref().unwrap().stringify())
                                    .align_y(Alignment::Center)
                                    .size(36)
                                    .style(text::primary)
                            ]
                            .spacing(5)
                            .align_y(Alignment::Center),
                        )
                        .on_press(Message::WeatherWindowToggle),
                    )
                    .padding(Padding::default().horizontal(self.hpadding))
                    .width(Length::Shrink)
                    .height(Length::Fill)
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();

                        container::Style::default()
                            .background(palette.background.weak.color)
                            .border(border::rounded(self.radius))
                    })
                    .into()
                }
                None => button(
                    svg(svg::Handle::from_memory(
                        get_svg("commons", "question_mark").as_bytes(),
                    ))
                    .width(36)
                    .height(36),
                )
                .on_press(Message::ParseCurrentWeather)
                .style(|theme: &Theme, state: Status| button::Style {
                    background: Some(
                        if state == Status::Hovered {
                            theme.extended_palette().background.stronger.color
                        } else {
                            theme.extended_palette().background.weak.color
                        }
                        .into(),
                    ),
                    border: iced::Border {
                        radius: Radius::new(self.radius as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .padding(Padding::default().horizontal(self.hpadding))
                .width(Length::Shrink)
                .height(Length::Fill)
                .into(),
            }
        };

        let media: Element<'_, Message> = {
            if let Some(player) = &self.tracked_player {
                container(row![text(format!(
                    "Media is playing :+1:, by player {}",
                    player.dbus_name()
                ))])
                .into()
            } else {
                container(space()).into()
            }
        };

        let media_group: Element<'_, Message> = {
            match &self.tracked_player {
                Some(_) => row![Self::separator(), media]
                    .align_y(Alignment::Center)
                    .spacing(self.spacing)
                    .into(),
                None => space().into(),
            }
        };

        let left = row![clock, Self::separator(), weather_widget, media_group]
            .align_y(Alignment::Center)
            .spacing(self.spacing);

        //
        let middle = row![].align_y(Alignment::Center).spacing(self.spacing);

        //
        let right = row![].align_y(Alignment::Center).spacing(self.spacing);

        container(
            row![
                left,
                space::horizontal(),
                middle,
                space::horizontal(),
                right
            ]
            .align_y(Alignment::Center)
            .padding(5),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(Alignment::Center)
        .align_x(Alignment::Center)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();

            container::Style::default()
                .background(palette.background.base.color)
                .border(border::rounded(self.radius))
        })
        .into()
    }

    fn subscription(_state: &State) -> Subscription<Message> {
        let subs = vec![
            iced::time::every(Duration::from_secs(1)).map(|_| Message::SecondTrigger),
            iced::time::every(Duration::from_mins(15)).map(|_| Message::ParseCurrentWeather),
            iced::Subscription::run_with(MprisPlug, mpris_stream_builder)
                .map(|event| Message::MprisEvent(event)),
        ];

        Subscription::batch(subs)
    }

    fn style(_: &State, theme: &Theme) -> theme::Style {
        theme::Style {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }
}

fn main() -> iced_layershell::Result {
    assets::load_assets();

    // Setting ICED_BACKEND to software will panic, for some reason...
    unsafe {
        std::env::set_var("ICED_BACKEND", "tiny-skia");
    }

    let theme = Theme::CatppuccinMocha;
    let radius = 10;
    let time_fmt = "%H:%M:%S";
    let spacing = 4;
    let clock_widget_width = 140;
    let hpadding = 4;
    let units = Units::default();
    // let units = Units::new(Speed::Mph, TempUnit::Fahrenheit, weather::prelude::Length::Inch);

    daemon(
        move || {
            State::new(
                theme.clone(),
                radius,
                spacing,
                time_fmt,
                clock_widget_width,
                hpadding,
                units.clone(),
            )
        },
        "Rusty Bar",
        State::update,
        State::view,
    )
    .settings(Settings {
        layer_settings: LayerShellSettings {
            layer: iced_layershell::reexport::Layer::Top,
            size: Some((0, 50)),
            start_mode: StartMode::AllScreens,
            anchor: Anchor::Top | Anchor::Left | Anchor::Right,
            exclusive_zone: 50,
            margin: (5, 10, 5, 10),
            keyboard_interactivity: iced_layershell::reexport::KeyboardInteractivity::OnDemand,
            ..Default::default()
        },
        fonts: vec![std::include_bytes!("assets/fonts/Itim-Regular.ttf").into()],
        default_font: Font::with_name("Itim"),
        ..Default::default()
    })
    .style(State::style)
    .theme(|state: &State, _| state.theme.clone())
    .subscription(State::subscription)
    .run()
}

// Zero sized type to id the mpirs stream builder,
// so it doesnt get rebuilt every frame
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct MprisPlug;

fn mpris_stream_builder(_id: &MprisPlug) -> BoxStream<'static, MprisEvent> {
    stream::channel(32, async |mut output| {
        println!("@M Creating mpris object");
        let mpris = Mpris::new()
            .await
            .expect("@M Failed to create MPRIS object: {0}");

        let mut stream = mpris
            .new_event_loop(vec![PlaybackStatus.into_any()], vec![], false)
            .await
            .expect("Failed to create event loop");

        while let Some(event) = stream.next().await {
            let _ = output.send(event).await;
        }

        println!("@M MPRIS stream ended!!!")
    })
    .boxed()
}
