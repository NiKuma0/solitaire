//! Saving and loading the game, settings and statistics.
//!
//! A value that fails to load is copied to `<key>.bad` and treated as
//! missing, so one damaged entry never takes the others down with it.

use serde::Serialize;
use serde::de::DeserializeOwned;
use solitaire_core::Game;

use crate::settings::Settings;
use crate::stats::Stats;
use crate::storage::{Storage, StorageError};

pub const GAME_KEY: &str = "solitaire/v1/game";
pub const SETTINGS_KEY: &str = "solitaire/v1/settings";
pub const STATS_KEY: &str = "solitaire/v1/stats";

#[derive(Debug, Default)]
pub struct Saved {
    pub game: Option<Game>,
    /// `None` on first launch, so platform defaults can apply.
    pub settings: Option<Settings>,
    pub stats: Stats,
}

pub fn load(storage: &mut dyn Storage) -> Saved {
    Saved {
        game: read(storage, GAME_KEY),
        settings: read(storage, SETTINGS_KEY),
        stats: read(storage, STATS_KEY).unwrap_or_default(),
    }
}

pub fn save_game(storage: &mut dyn Storage, game: &Game) -> Result<(), StorageError> {
    write(storage, GAME_KEY, game)
}

pub fn save_settings(storage: &mut dyn Storage, settings: &Settings) -> Result<(), StorageError> {
    write(storage, SETTINGS_KEY, settings)
}

pub fn save_stats(storage: &mut dyn Storage, stats: &Stats) -> Result<(), StorageError> {
    write(storage, STATS_KEY, stats)
}

fn read<T: DeserializeOwned>(storage: &mut dyn Storage, key: &str) -> Option<T> {
    let raw = storage.get(key)?;
    let value = serde_json::from_str(&raw).ok();
    if value.is_none() {
        // Keep the damaged copy for a later look; best effort only.
        let _ = storage.set(&format!("{key}.bad"), &raw);
    }
    value
}

fn write<T: Serialize>(
    storage: &mut dyn Storage,
    key: &str,
    value: &T,
) -> Result<(), StorageError> {
    let json = serde_json::to_string(value).map_err(|e| StorageError::Io(e.into()))?;
    storage.set(key, &json)
}

#[cfg(test)]
#[path = "tests/persist.rs"]
mod tests;
