//! The playing table: turns gestures and buttons into game moves.
//!
//! `solitaire_core::Game` is the source of truth; this only tracks what the
//! player is doing with it (a drag in progress) and cached flags for the UI.

use macroquad::math::{Rect, Vec2};
use solitaire_core::{Card, Game, Move, PileId};

use crate::gesture::Gesture;
use crate::hit::{Spot, drop_targets, spot_at};
use crate::layout::{Button, Insets, Layout};

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

pub struct Table {
    game: Game,
    layout: Layout,
    drag: Option<Drag>,
    next_seed: u64,
    can_auto_complete: bool,
    stuck: bool,
}

impl Table {
    /// `next_seed` seeds the games started with "New".
    #[must_use]
    pub fn new(game: Game, screen: Vec2, insets: Insets, next_seed: u64) -> Self {
        let mut table = Self {
            game,
            layout: Layout::new(screen, insets),
            drag: None,
            next_seed,
            can_auto_complete: false,
            stuck: false,
        };
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
        self.stuck
    }

    pub fn resize(&mut self, screen: Vec2, insets: Insets) {
        if self.layout.screen != screen {
            self.layout = Layout::new(screen, insets);
        }
    }

    #[must_use]
    pub fn is_enabled(&self, button: Button) -> bool {
        match button {
            Button::New => true,
            Button::Undo => self.game.can_undo(),
            Button::Redo => self.game.can_redo(),
            Button::AutoComplete => self.can_auto_complete,
        }
    }

    pub fn press(&mut self, button: Button) {
        if !self.is_enabled(button) {
            return;
        }
        match button {
            Button::New => self.new_game(),
            Button::Undo => {
                let _ = self.game.undo();
            }
            Button::Redo => {
                let _ = self.game.redo();
            }
            Button::AutoComplete => {
                for mv in self.game.auto_complete_plan().unwrap_or_default() {
                    if self.game.apply(mv).is_err() {
                        break;
                    }
                }
            }
        }
        self.refresh();
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
            Some(Spot::Card { pile, index }) => self
                .face_up_card(pile, index)
                .and_then(|card| self.game.best_move_for(card)),
            Some(Spot::Empty(_)) | None => None,
        };
        if let Some(mv) = mv
            && self.game.apply(mv).is_ok()
        {
            self.refresh();
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
                self.refresh();
                return;
            }
        }
    }

    fn new_game(&mut self) {
        let seed = self.next_seed;
        self.next_seed = splitmix64(seed);
        self.game = Game::new(seed, self.game.draw_mode());
        self.drag = None;
        self.refresh();
    }

    /// Recomputes the cached flags after the game changed.
    fn refresh(&mut self) {
        self.can_auto_complete = self.game.can_auto_complete();
        self.stuck = !self.can_auto_complete && self.game.is_stuck();
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
