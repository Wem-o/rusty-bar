use std::{cmp::Ordering, fmt::Display, sync::Arc, time::Duration};

use mpris_client_async::{
    Player,
    player_types::{Loop, Metadata, Playback},
};
use zbus::names::OwnedBusName;

/// A type that bundles a player and its current metadata
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerBundle {
    pub inner: Arc<Player>,
    pub bus_name: OwnedBusName,

    pub name: PlayerName,

    pub metadata: Option<Metadata>,
    pub playback_status: Playback,
    pub loop_status: Loop,
    pub is_shuffle: bool,
    pub position: Duration,
    pub can_go_next: bool,
    pub can_go_back: bool,
    pub can_play: bool,
    pub can_pause: bool,
    pub can_seek: bool,

    pub can_control: bool,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A name of a player.
///
/// A player should display the one with the
/// "highest value" which you can check by
/// comparing two value, as PartialOrd is
/// implemented.
///
/// The order is BusEntry < DesktopEntry < DisplayEntry
pub enum PlayerName {
    BusEntry(String),
    DesktopEntry(String),
    DisplayName(String),
}

impl PlayerName {
    fn to_string(&self) -> String {
        match self {
            Self::BusEntry(s) => s.clone(),
            Self::DesktopEntry(s) => s.clone(),
            Self::DisplayName(s) => s.clone(),
        }
    }
}

impl Display for PlayerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialOrd for PlayerName {
    /// Follows BusEntry < DesktopEntry < DisplayEntry
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::BusEntry(_), Self::BusEntry(_)) => Some(Ordering::Equal),
            (Self::DesktopEntry(_), Self::DesktopEntry(_)) => Some(Ordering::Equal),
            (Self::DisplayName(_), Self::DisplayName(_)) => Some(Ordering::Equal),

            (Self::BusEntry(_), Self::DesktopEntry(_))
            | (Self::BusEntry(_), Self::DisplayName(_)) => Some(Ordering::Less),

            (Self::DesktopEntry(_), Self::DisplayName(_)) => Some(Ordering::Less),
            (Self::DesktopEntry(_), Self::BusEntry(_)) => Some(Ordering::Greater),

            (Self::DisplayName(_), Self::DesktopEntry(_))
            | (Self::DisplayName(_), Self::BusEntry(_)) => Some(std::cmp::Ordering::Greater),
        }
    }
}
