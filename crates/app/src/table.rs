//! The playing table: turns gestures and buttons into game moves.
//!
//! `solitaire_core::Game` is the source of truth; this only tracks what the
//! player is doing with it (a drag in progress) and cached flags for the UI.

use macroquad::math::{Rect, Vec2};
use solitaire_core::{Card, DrawMode, Game, Move, PileId};

use crate::gesture::Gesture;
use crate::hit::{Spot, drop_targets, spot_at};
use crate::layout::{Button, Insets, Layout, LayoutOptions};
use crate::settings::Settings;

/// A stack being dragged: card `index` of `pile` and everything above it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drag {
    pub pile: PileId,
    pub index: usize,
    /// Pointer position relative to the bottom card's top-left corner.
    pub grab: Vec2,
    pub pointer: Vec2,
}

impl Drag {
    /// Where the bottom card of the stack is drawn.
    #[must_use]
    pub fn rect(&self, card: Vec2) -> Rect {
        let p = self.pointer - self.grab;
        Rect::new(p.x, p.y, card.x, card.y)
    }
}

/// Things the rest of the app reacts to, drained once per frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    OpenSettings,
    /// A tapped card had nowhere to go.
    Rejected(Card),
    NewGame,
    AutoComplete,
    /// The game changed and should be saved.
    Changed,
    /// A game ended, won or abandoned, for the statistics.
    GameOver {
        draw: DrawMode,
        score: u32,
        won: bool,
        moves: u32,
    },
}

/// Two taps on the same card within this many seconds make a double tap.
const DOUBLE_TAP: f64 = 0.4;

/// Expensive facts about the game, recomputed after each change.
#[derive(Debug, Clone, Copy, Default)]
struct Hints {
    can_auto_complete: bool,
    stuck: bool,
}

pub struct Table {
    game: Game,
    events: Vec<Event>,
    layout: Layout,
    drag: Option<Drag>,
    next_seed: u64,
    hints: Hints,
    insets: Insets,
    options: LayoutOptions,
    tap_to_move: bool,
    /// Draw mode for the next new game.
    next_draw: DrawMode,
    /// Seconds, set by the caller each frame; only used to spot double taps.
    clock: f64,
    last_tap: Option<(Card, f64)>,
    /// This game's win is already in the statistics.
    recorded_win: bool,
}

impl Table {
    /// `next_seed` seeds the games started with "New".
    #[must_use]
    pub fn new(game: Game, screen: Vec2, insets: Insets, next_seed: u64) -> Self {
        let mut table = Self {
            game,
            events: Vec::new(),
            layout: Layout::new(screen, insets),
            drag: None,
            next_seed,
            hints: Hints::default(),
            insets,
            options: LayoutOptions::default(),
            tap_to_move: true,
            next_draw: DrawMode::One,
            clock: 0.0,
            last_tap: None,
            recorded_win: false,
        };
        table.recorded_win = table.game.is_won();
        table.next_draw = table.game.draw_mode();
        table.refresh();
        table
    }

    #[must_use]
    pub const fn game(&self) -> &Game {
        &self.game
    }

    #[must_use]
    pub const fn layout(&self) -> &Layout {
        &self.layout
    }

    #[must_use]
    pub const fn drag(&self) -> Option<&Drag> {
        self.drag.as_ref()
    }

    #[must_use]
    pub const fn is_stuck(&self) -> bool {
        self.hints.stuck
    }

    /// Events since the last call.
    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn resize(&mut self, screen: Vec2, insets: Insets) {
        if self.layout.screen != screen || self.insets != insets {
            self.insets = insets;
            self.layout = Layout::with_options(screen, insets, self.options);
        }
    }

    /// Applies the player's settings. Draw mode waits for the next game.
    pub fn configure(&mut self, settings: &Settings) {
        self.tap_to_move = settings.tap_to_move;
        self.next_draw = settings.draw;
        let options = settings.layout_options();
        if options != self.options {
            self.options = options;
            self.layout = Layout::with_options(self.layout.screen, self.insets, options);
        }
    }

    pub fn set_clock(&mut self, now: f64) {
        self.clock = now;
    }

    #[must_use]
    pub fn is_enabled(&self, button: Button) -> bool {
        match button {
            Button::New | Button::Settings => true,
            // A win is final: no taking it back.
            Button::Undo => self.game.can_undo() && !self.game.is_won(),
            Button::Redo => self.game.can_redo() && !self.game.is_won(),
            Button::AutoComplete => self.hints.can_auto_complete,
        }
    }

    pub fn press(&mut self, button: Button) {
        if !self.is_enabled(button) {
            return;
        }
        match button {
            Button::Settings => {
                self.events.push(Event::OpenSettings);
                return;
            }
            Button::New => {
                self.new_game();
                return;
            }
            Button::Undo => {
                let _ = self.game.undo();
            }
            Button::Redo => {
                let _ = self.game.redo();
            }
            Button::AutoComplete => {
                self.events.push(Event::AutoComplete);
                for mv in self.game.auto_complete_plan().unwrap_or_default() {
                    if self.game.apply(mv).is_err() {
                        break;
                    }
                }
            }
        }
        self.changed();
    }

    /// Deals the current game's cards again from the start.
    pub fn restart(&mut self) {
        self.abandon();
        self.game = Game::new(self.game.seed(), self.game.draw_mode());
        self.start_game();
    }

    pub fn handle(&mut self, gesture: Gesture) {
        match gesture {
            Gesture::Tap(p) => self.tap(p),
            Gesture::DragStart { origin, at } => self.start_drag(origin, at),
            Gesture::DragMove(p) => {
                if let Some(drag) = &mut self.drag {
                    drag.pointer = p;
                }
            }
            Gesture::Drop(p) => self.drop_at(p),
        }
    }

    fn tap(&mut self, p: Vec2) {
        if let Some(&(button, _)) = self.layout.buttons.iter().find(|(_, r)| r.contains(p)) {
            self.press(button);
            return;
        }
        if self.game.is_won() {
            if self.layout.win_button.contains(p) {
                self.new_game();
            }
            return;
        }
        let board = self.game.board();
        let mv = match spot_at(&self.layout, board, self.game.draw_mode(), p) {
            Some(Spot::Stock) if board.stock.is_empty() => Some(Move::Recycle),
            Some(Spot::Stock) => Some(Move::Draw),
            Some(Spot::Card { pile, index }) => {
                let Some(card) = self.face_up_card(pile, index) else {
                    return;
                };
                if !self.tap_to_move && !self.is_double_tap(card) {
                    return;
                }
                let mv = self.game.best_move_for(card);
                if mv.is_none() {
                    self.events.push(Event::Rejected(card));
                }
                mv
            }
            Some(Spot::Empty(_)) | None => None,
        };
        if let Some(mv) = mv
            && self.game.apply(mv).is_ok()
        {
            self.changed();
        }
    }

    fn start_drag(&mut self, origin: Vec2, at: Vec2) {
        self.drag = None;
        if self.game.is_won() {
            return;
        }
        let draw = self.game.draw_mode();
        let Some(Spot::Card { pile, index }) =
            spot_at(&self.layout, self.game.board(), draw, origin)
        else {
            return;
        };
        if self.face_up_card(pile, index).is_none() {
            return;
        }
        let base = self.layout.card_rect(self.game.board(), draw, pile, index);
        self.drag = Some(Drag {
            pile,
            index,
            grab: origin - base.point(),
            pointer: at,
        });
    }

    fn drop_at(&mut self, p: Vec2) {
        let Some(mut drag) = self.drag.take() else {
            return;
        };
        drag.pointer = p;
        let Ok(count) = u8::try_from(self.pile_len(drag.pile) - drag.index) else {
            return;
        };
        let rect = drag.rect(self.layout.card);
        let targets = drop_targets(&self.layout, self.game.board(), self.game.draw_mode(), rect);
        for to in targets {
            let mv = Move::Transfer {
                from: drag.pile,
                to,
                count,
            };
            if to != drag.pile && self.game.apply(mv).is_ok() {
                self.changed();
                return;
            }
        }
    }

    /// Remembers this tap; true if it completes a double tap on `card`.
    fn is_double_tap(&mut self, card: Card) -> bool {
        let double = self
            .last_tap
            .is_some_and(|(c, t)| c == card && self.clock - t <= DOUBLE_TAP);
        self.last_tap = if double {
            None
        } else {
            Some((card, self.clock))
        };
        double
    }

    fn new_game(&mut self) {
        self.abandon();
        let seed = self.next_seed;
        self.next_seed = splitmix64(seed);
        self.game = Game::new(seed, self.next_draw);
        self.start_game();
    }

    fn start_game(&mut self) {
        self.drag = None;
        self.recorded_win = false;
        self.events.push(Event::NewGame);
        self.changed();
    }

    /// Records the current game as lost if the player made any move in it.
    fn abandon(&mut self) {
        let board = self.game.board();
        if !self.recorded_win && board.moves > 0 {
            self.events.push(Event::GameOver {
                draw: self.game.draw_mode(),
                score: board.score,
                won: false,
                moves: board.moves,
            });
        }
    }

    /// After any change to the game: refresh flags, ask for a save, record a win.
    fn changed(&mut self) {
        self.refresh();
        self.events.push(Event::Changed);
        if self.game.is_won() && !self.recorded_win {
            self.recorded_win = true;
            let board = self.game.board();
            self.events.push(Event::GameOver {
                draw: self.game.draw_mode(),
                score: board.score,
                won: true,
                moves: board.moves,
            });
        }
    }

    /// Recomputes the cached flags after the game changed.
    fn refresh(&mut self) {
        self.hints.can_auto_complete = self.game.can_auto_complete();
        self.hints.stuck = !self.hints.can_auto_complete && self.game.is_stuck();
    }

    fn pile_len(&self, pile: PileId) -> usize {
        let b = self.game.board();
        match pile {
            PileId::Stock => b.stock.len(),
            PileId::Waste => b.waste.len(),
            PileId::Foundation(i) => b.foundations.get(usize::from(i)).map_or(0, Vec::len),
            PileId::Tableau(i) => b
                .tableau
                .get(usize::from(i))
                .map_or(0, solitaire_core::Column::len),
        }
    }

    /// The card at `index` of `pile` if the player may pick it up.
    fn face_up_card(&self, pile: PileId, index: usize) -> Option<Card> {
        let b = self.game.board();
        match pile {
            PileId::Waste => (index + 1 == b.waste.len()).then(|| b.waste.get(index).copied())?,
            PileId::Foundation(i) => {
                let cards = b.foundations.get(usize::from(i))?;
                (index + 1 == cards.len()).then(|| cards.get(index).copied())?
            }
            PileId::Tableau(i) => {
                let col = b.tableau.get(usize::from(i))?;
                (index >= col.face_down_count()).then(|| col.cards().get(index).copied())?
            }
            PileId::Stock => None,
        }
    }
}

/// Steps the seed for the next new game (`SplitMix64`).
const fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
#[path = "tests/table.rs"]
mod tests;
