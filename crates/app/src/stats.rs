//! Local statistics (SPEC §6.3). Never leaves the device.
//!
//! Totals, best score and streaks are kept as running counters, so they stay
//! right after old games drop out of the 200-game history.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use solitaire_core::DrawMode;

pub const HISTORY_LIMIT: usize = 200;

/// One finished or abandoned game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameRecord {
    /// Seconds since the Unix epoch when the game ended.
    pub date: u64,
    pub draw: DrawMode,
    pub score: u32,
    pub won: bool,
    pub moves: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub played: u32,
    pub won: u32,
}

impl Tally {
    /// Rounded percentage of games won; `None` before the first game.
    #[must_use]
    pub fn win_percent(self) -> Option<u32> {
        (self.played > 0).then(|| (self.won * 100 + self.played / 2) / self.played)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    history: VecDeque<GameRecord>,
    draw_one: Tally,
    draw_three: Tally,
    best_score: u32,
    current_streak: u32,
    best_streak: u32,
}

impl Stats {
    pub fn record(&mut self, game: GameRecord) {
        let tally = match game.draw {
            DrawMode::One => &mut self.draw_one,
            DrawMode::Three => &mut self.draw_three,
        };
        tally.played += 1;
        if game.won {
            tally.won += 1;
            self.current_streak += 1;
            self.best_streak = self.best_streak.max(self.current_streak);
        } else {
            self.current_streak = 0;
        }
        self.best_score = self.best_score.max(game.score);
        if self.history.len() == HISTORY_LIMIT {
            self.history.pop_front();
        }
        self.history.push_back(game);
    }

    /// Games for one draw mode, or all of them.
    #[must_use]
    pub fn tally(&self, draw: Option<DrawMode>) -> Tally {
        match draw {
            Some(DrawMode::One) => self.draw_one,
            Some(DrawMode::Three) => self.draw_three,
            None => Tally {
                played: self.draw_one.played + self.draw_three.played,
                won: self.draw_one.won + self.draw_three.won,
            },
        }
    }

    #[must_use]
    pub const fn best_score(&self) -> u32 {
        self.best_score
    }

    #[must_use]
    pub const fn current_streak(&self) -> u32 {
        self.current_streak
    }

    #[must_use]
    pub const fn best_streak(&self) -> u32 {
        self.best_streak
    }

    /// Oldest first.
    #[must_use]
    pub const fn history(&self) -> &VecDeque<GameRecord> {
        &self.history
    }

    /// The last `n` games, oldest first.
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &GameRecord> {
        self.history
            .iter()
            .skip(self.history.len().saturating_sub(n))
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// `YYYY-MM-DD` in UTC (no time zone data on the device side).
#[must_use]
pub fn utc_date(unix_seconds: u64) -> String {
    // Howard Hinnant's civil_from_days, for days since 1970-01-01.
    let days = (unix_seconds / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
#[path = "tests/stats.rs"]
mod tests;
