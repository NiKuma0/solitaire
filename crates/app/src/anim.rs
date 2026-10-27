//! Card animations (SPEC §7.3). `core::Game` stays the truth: the animator
//! only eases each card from where it was shown towards its new placement.
//!
//! Time is passed in (seconds), so everything here is deterministic.

use std::collections::HashMap;
use std::f32::consts::PI;

use macroquad::math::Rect;
use solitaire_core::Card;

use crate::scene::Placement;
use crate::settings::AnimSpeed;

/// Why the table changed, when that matters for how it animates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cue {
    None,
    /// A new game: cards fly out of the stock one after another.
    Deal,
    /// Auto-complete: cards cascade to the foundations, low ranks first.
    Cascade,
}

/// How a card should be drawn this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shown {
    pub rect: Rect,
    pub face_up: bool,
    /// Horizontal squeeze while turning over (1 = flat on the table).
    pub scale_x: f32,
}

#[derive(Debug, Clone, Copy)]
struct Tween {
    from: Rect,
    from_face: bool,
    start: f64,
    duration: f32,
}

impl Tween {
    fn progress(&self, now: f64) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (((now - self.start) as f32) / self.duration).clamp(0.0, 1.0)
    }

    fn end(&self) -> f64 {
        self.start + f64::from(self.duration)
    }
}

/// Deal and cascade spacing between cards, as a fraction of a move.
const DEAL_STAGGER: f32 = 0.15;
const CASCADE_STAGGER: f32 = 0.5;
/// Shake length and width, relative to a move and to the card width.
const SHAKE_MOVES: f32 = 1.5;
const SHAKE_AMPLITUDE: f32 = 0.08;
/// The win wave lasts this many moves.
const WIN_MOVES: f32 = 12.0;

#[derive(Debug, Default)]
pub struct Animator {
    last: HashMap<Card, Placement>,
    tweens: HashMap<Card, Tween>,
    shakes: HashMap<Card, (f64, f32)>,
    deal_origin: Option<Rect>,
    win: Option<(f64, f32)>,
}

#[must_use]
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

impl Animator {
    /// Where cards fly from when a game is dealt (the stock).
    pub fn set_deal_origin(&mut self, stock: Rect) {
        self.deal_origin = Some(stock);
    }

    /// Compares the new placements with the last ones and starts animations
    /// for cards that moved or turned. Any real change first finishes what is
    /// still running, so input is never blocked by an animation.
    pub fn sync(&mut self, placements: &[Placement], now: f64, speed: AnimSpeed, cue: Cue) {
        let duration = speed.move_seconds();
        let changed: Vec<&Placement> = placements
            .iter()
            .filter(|p| {
                !p.dragged
                    && self
                        .last
                        .get(&p.card)
                        .is_some_and(|prev| prev.rect != p.rect || prev.face_up != p.face_up)
            })
            .collect();

        for p in placements.iter().filter(|p| p.dragged) {
            self.tweens.remove(&p.card);
        }
        if !changed.is_empty() || cue != Cue::None {
            self.tweens.clear();
        }

        if duration > 0.0 {
            match cue {
                Cue::Deal => self.deal(placements, now, duration),
                Cue::Cascade => {
                    let mut moving = changed;
                    moving.sort_by_key(|p| (p.card.rank, p.card.suit));
                    for (k, p) in moving.into_iter().enumerate() {
                        let start = now + f64::from(k as f32 * CASCADE_STAGGER * duration);
                        self.tween_from_last(p, start, duration);
                    }
                }
                Cue::None => {
                    for p in changed {
                        self.tween_from_last(p, now, duration);
                    }
                }
            }
        }

        self.tweens.retain(|_, t| t.end() > now);
        self.last = placements.iter().map(|p| (p.card, *p)).collect();
    }

    fn tween_from_last(&mut self, p: &Placement, start: f64, duration: f32) {
        if let Some(prev) = self.last.get(&p.card) {
            let tween = Tween {
                from: prev.rect,
                from_face: prev.face_up,
                start,
                duration,
            };
            self.tweens.insert(p.card, tween);
        }
    }

    fn deal(&mut self, placements: &[Placement], now: f64, duration: f32) {
        let Some(origin) = self.deal_origin else {
            return;
        };
        let mut k = 0.0;
        for p in placements.iter().filter(|p| !p.dragged && p.rect != origin) {
            let start = now + f64::from(k * DEAL_STAGGER * duration);
            self.tweens.insert(
                p.card,
                Tween {
                    from: origin,
                    from_face: false,
                    start,
                    duration,
                },
            );
            k += 1.0;
        }
    }

    /// Wiggles `card` sideways to say "no move".
    pub fn shake(&mut self, card: Card, now: f64, speed: AnimSpeed) {
        let duration = speed.move_seconds() * SHAKE_MOVES;
        if duration > 0.0 {
            self.shakes.insert(card, (now, duration));
        }
    }

    pub fn start_win(&mut self, now: f64, speed: AnimSpeed) {
        let duration = speed.move_seconds() * WIN_MOVES;
        self.win = (duration > 0.0).then_some((now, duration));
    }

    pub fn stop_win(&mut self) {
        self.win = None;
    }

    /// How far foundation `index`'s top card is lifted, as a fraction of its
    /// height: a wave travelling across the foundations that fades out.
    #[must_use]
    pub fn win_lift(&self, index: usize, now: f64) -> f32 {
        let Some((start, duration)) = self.win else {
            return 0.0;
        };
        let u = ((now - start) as f32) / duration;
        if !(0.0..1.0).contains(&u) {
            return 0.0;
        }
        let phase = u * 4.0 * 2.0 * PI - index as f32 * 0.9;
        phase.sin().max(0.0) * (1.0 - u) * 0.12
    }

    #[must_use]
    pub fn shown(&self, p: &Placement, now: f64) -> Shown {
        let mut shown = Shown {
            rect: p.rect,
            face_up: p.face_up,
            scale_x: 1.0,
        };
        if p.dragged {
            return shown;
        }
        if let Some(t) = self.tweens.get(&p.card) {
            let u = t.progress(now);
            if u < 1.0 {
                let e = ease_out_cubic(u);
                shown.rect = Rect::new(
                    t.from.x + (p.rect.x - t.from.x) * e,
                    t.from.y + (p.rect.y - t.from.y) * e,
                    p.rect.w,
                    p.rect.h,
                );
                if t.from_face != p.face_up {
                    // Squeeze to an edge showing the old side, then open on the new one.
                    if u < 0.5 {
                        shown.face_up = t.from_face;
                        shown.scale_x = 1.0 - 2.0 * u;
                    } else {
                        shown.scale_x = 2.0 * u - 1.0;
                    }
                }
            }
        }
        if let Some(&(start, duration)) = self.shakes.get(&p.card) {
            let u = ((now - start) as f32) / duration;
            if (0.0..1.0).contains(&u) {
                let dx = (u * 6.0 * PI).sin() * (1.0 - u) * SHAKE_AMPLITUDE * p.rect.w;
                shown.rect.x += dx;
            }
        }
        shown
    }

    /// Placements in drawing order: cards in flight above resting ones.
    #[must_use]
    pub fn draw_order(&self, placements: &[Placement], now: f64) -> Vec<Placement> {
        let flying = |p: &Placement| {
            self.tweens
                .get(&p.card)
                .is_some_and(|t| t.progress(now) < 1.0)
        };
        let mut order = placements.to_vec();
        order.sort_by_key(|p| (p.dragged, flying(p), p.z));
        order
    }

    #[must_use]
    pub fn is_animating(&self, now: f64) -> bool {
        self.tweens.values().any(|t| t.end() > now)
            || self.shakes.values().any(|&(s, d)| now < s + f64::from(d))
            || self.win.is_some_and(|(s, d)| now < s + f64::from(d))
    }
}

#[cfg(test)]
#[path = "tests/anim.rs"]
mod tests;
