//! Ties the table, settings screen, animator and theme together.

use macroquad::math::Vec2;
use solitaire_core::Game;

use crate::anim::{Animator, Cue};
use crate::gesture::Gesture;
use crate::layout::{Button, Insets};
use crate::menu::{Menu, MenuHit, Row, Tab, apply};
use crate::persist;
use crate::scene::placements;
use crate::settings::{AnimSpeed, Settings};
use crate::stats::{GameRecord, Stats};
use crate::storage::{MemoryStorage, Storage};
use crate::table::{Event, Table};
use crate::theme::{Palette, ThemeId, palette};

/// Theme crossfade length when animations are on.
const FADE_SECONDS: f32 = 0.3;

struct MenuState {
    menu: Menu,
    screen: Vec2,
    tab: Tab,
    scroll: f32,
    /// Pointer y and scroll when the current drag started.
    drag_anchor: Option<(f32, f32)>,
    /// A first tap on "Reset statistics" asks; a second one before this time resets.
    confirm_reset_until: Option<f64>,
}

/// The open menu, for drawing.
#[derive(Debug, Clone, Copy)]
pub struct MenuView<'a> {
    pub menu: &'a Menu,
    pub tab: Tab,
    pub scroll: f32,
    /// "Reset statistics" is waiting for its confirming tap.
    pub confirm_reset: bool,
}

/// How long a reset request waits for its confirming tap.
const CONFIRM_SECONDS: f64 = 3.0;

struct Fade {
    from: Palette,
    to: ThemeId,
    start: f64,
    duration: f32,
}

pub struct App {
    table: Table,
    settings: Settings,
    anim: Animator,
    menu: Option<MenuState>,
    fade: Fade,
    screen: Vec2,
    insets: Insets,
    cue: Cue,
    won: bool,
    stats: Stats,
    storage: Box<dyn Storage>,
    /// Seconds since the Unix epoch, for dating finished games.
    unix_time: fn() -> u64,
    /// The last attempt to save failed (shown to the player).
    save_failed: bool,
    /// The menu reopens on the tab it was closed on.
    last_tab: Tab,
    clock: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    /// Ctrl+Z
    Undo,
    /// Ctrl+Shift+Z
    Redo,
    /// N
    NewGame,
    /// Space: draw from the stock or turn the waste over.
    Draw,
}

/// What the app needs from the platform to start.
#[derive(Debug, Clone, Copy)]
pub struct Startup {
    pub screen: Vec2,
    pub insets: Insets,
    /// Seeds the first game when there is no saved one, and later new games.
    pub seed: u64,
    pub now: f64,
    /// The system asks for reduced motion; applies when no settings are saved yet.
    pub reduced_motion: bool,
    pub unix_time: fn() -> u64,
}

impl App {
    #[must_use]
    pub fn new(
        game: Game,
        screen: Vec2,
        insets: Insets,
        next_seed: u64,
        settings: Settings,
        now: f64,
    ) -> Self {
        let storage = Box::new(MemoryStorage::default());
        Self::assemble(
            game,
            settings,
            Stats::default(),
            storage,
            || 0,
            (screen, insets, next_seed, now),
        )
    }

    /// Resumes the saved game, settings and statistics, or starts fresh.
    #[must_use]
    pub fn start(mut storage: Box<dyn Storage>, env: Startup) -> Self {
        let saved = persist::load(storage.as_mut());
        let settings = saved.settings.unwrap_or_else(|| Settings {
            animation: if env.reduced_motion {
                AnimSpeed::Off
            } else {
                AnimSpeed::Normal
            },
            ..Settings::default()
        });
        let game = saved
            .game
            .unwrap_or_else(|| Game::new(env.seed, settings.draw));
        let next_seed = env.seed.rotate_left(32) ^ 0x5eed;
        Self::assemble(
            game,
            settings,
            saved.stats,
            storage,
            env.unix_time,
            (env.screen, env.insets, next_seed, env.now),
        )
    }

    fn assemble(
        game: Game,
        settings: Settings,
        stats: Stats,
        storage: Box<dyn Storage>,
        unix_time: fn() -> u64,
        (screen, insets, next_seed, now): (Vec2, Insets, u64, f64),
    ) -> Self {
        let mut table = Table::new(game, screen, insets, next_seed);
        table.configure(&settings);
        let fade = Fade {
            from: palette(settings.theme),
            to: settings.theme,
            start: now,
            duration: 0.0,
        };
        let mut app = Self {
            table,
            settings,
            anim: Animator::default(),
            menu: None,
            fade,
            screen,
            insets,
            cue: Cue::None,
            won: false,
            stats,
            storage,
            unix_time,
            save_failed: false,
            last_tab: Tab::Settings,
            clock: now,
        };
        app.update(screen, now);
        app
    }

    #[must_use]
    pub const fn table(&self) -> &Table {
        &self.table
    }

    #[must_use]
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    #[must_use]
    pub const fn animator(&self) -> &Animator {
        &self.anim
    }

    #[must_use]
    pub const fn stats(&self) -> &Stats {
        &self.stats
    }

    #[must_use]
    pub fn storage(&self) -> &dyn Storage {
        self.storage.as_ref()
    }

    #[must_use]
    pub const fn save_failed(&self) -> bool {
        self.save_failed
    }

    #[must_use]
    pub const fn screen(&self) -> Vec2 {
        self.screen
    }

    /// The open settings screen and how far it is scrolled.
    #[must_use]
    pub fn menu(&self) -> Option<MenuView<'_>> {
        self.menu.as_ref().map(|m| MenuView {
            menu: &m.menu,
            tab: m.tab,
            scroll: m.scroll,
            confirm_reset: m.confirm_reset_until.is_some_and(|t| self.clock < t),
        })
    }

    /// Colours at `now`, mid-crossfade after a theme change.
    #[must_use]
    pub fn palette(&self, now: f64) -> Palette {
        let target = palette(self.fade.to);
        if self.fade.duration <= 0.0 {
            return target;
        }
        let t = ((now - self.fade.start) as f32) / self.fade.duration;
        Palette::lerp(&self.fade.from, &target, t)
    }

    pub fn handle(&mut self, gesture: Gesture, now: f64) {
        if self.menu.is_some() {
            self.handle_menu(gesture, now);
        } else {
            self.table.set_clock(now);
            self.table.handle(gesture);
        }
    }

    /// Keyboard shortcuts and other direct button presses.
    /// A keyboard shortcut (SPEC §8). Ignored while the menu is open.
    pub fn shortcut(&mut self, shortcut: Shortcut) {
        if self.menu.is_some() {
            return;
        }
        match shortcut {
            Shortcut::Undo => self.table.press(Button::Undo),
            Shortcut::Redo => self.table.press(Button::Redo),
            Shortcut::NewGame => self.table.press(Button::New),
            Shortcut::Draw => self.table.draw_from_stock(),
        }
    }

    /// Once per frame, after input: reacts to what changed and syncs animations.
    pub fn update(&mut self, screen: Vec2, now: f64) {
        self.screen = screen;
        self.table.resize(screen, self.insets);
        if let Some(m) = &mut self.menu
            && m.screen != screen
        {
            m.menu = Menu::new(screen, self.insets);
            m.screen = screen;
            m.scroll = m.scroll.min(m.menu.max_scroll(m.tab));
        }
        for event in self.table.drain_events() {
            match event {
                Event::OpenSettings => {
                    let menu = Menu::new(screen, self.insets);
                    self.menu = Some(MenuState {
                        menu,
                        screen,
                        tab: self.last_tab,
                        scroll: 0.0,
                        drag_anchor: None,
                        confirm_reset_until: None,
                    });
                }
                Event::Rejected(card) => self.anim.shake(card, now, self.settings.animation),
                Event::NewGame => {
                    self.cue = Cue::Deal;
                    self.anim.stop_win();
                }
                Event::AutoComplete => self.cue = Cue::Cascade,
                Event::Changed => {
                    let result = persist::save_game(self.storage.as_mut(), self.table.game());
                    self.save_failed = result.is_err();
                }
                Event::GameOver {
                    draw,
                    score,
                    won,
                    moves,
                } => {
                    let date = (self.unix_time)();
                    self.stats.record(GameRecord {
                        date,
                        draw,
                        score,
                        won,
                        moves,
                    });
                    let result = persist::save_stats(self.storage.as_mut(), &self.stats);
                    self.save_failed = result.is_err();
                }
            }
        }

        let won = self.table.game().is_won();
        if won && !self.won {
            self.anim.start_win(now, self.settings.animation);
        }
        self.won = won;

        let layout = self.table.layout();
        self.anim.set_deal_origin(layout.stock);
        let scene = placements(layout, self.table.game(), self.table.drag());
        self.anim.sync(
            &scene,
            now,
            self.settings.animation,
            std::mem::replace(&mut self.cue, Cue::None),
        );
    }

    fn handle_menu(&mut self, gesture: Gesture, now: f64) {
        self.clock = now;
        let Some(m) = &mut self.menu else { return };
        match gesture {
            Gesture::Tap(p) => match m.menu.hit(p, m.scroll, m.tab) {
                Some(MenuHit::Done) => {
                    self.last_tab = m.tab;
                    self.menu = None;
                }
                Some(MenuHit::Tab(tab)) => {
                    m.tab = tab;
                    m.scroll = 0.0;
                }
                Some(MenuHit::Action(Row::Restart)) => {
                    self.last_tab = m.tab;
                    self.menu = None;
                    self.table.restart();
                }
                Some(MenuHit::Action(Row::ResetStats)) => {
                    if m.confirm_reset_until.is_some_and(|t| now < t) {
                        m.confirm_reset_until = None;
                        self.stats.reset();
                        let result = persist::save_stats(self.storage.as_mut(), &self.stats);
                        self.save_failed = result.is_err();
                    } else {
                        m.confirm_reset_until = Some(now + CONFIRM_SECONDS);
                    }
                }
                Some(hit) => self.change_setting(hit, now),
                None => {}
            },
            Gesture::DragStart { origin, at } => {
                if m.menu.viewport.contains(origin) {
                    m.drag_anchor = Some((origin.y, m.scroll));
                    scroll_to(m, at.y);
                }
            }
            Gesture::DragMove(p) => scroll_to(m, p.y),
            Gesture::Drop(p) => {
                scroll_to(m, p.y);
                m.drag_anchor = None;
            }
        }
    }

    fn change_setting(&mut self, hit: MenuHit, now: f64) {
        let old_theme = self.settings.theme;
        let old_palette = self.palette(now);
        if !apply(&mut self.settings, hit) {
            return;
        }
        let result = persist::save_settings(self.storage.as_mut(), &self.settings);
        self.save_failed = result.is_err();
        if self.settings.theme != old_theme {
            let duration = if self.settings.animation == AnimSpeed::Off {
                0.0
            } else {
                FADE_SECONDS
            };
            self.fade = Fade {
                from: old_palette,
                to: self.settings.theme,
                start: now,
                duration,
            };
        }
        self.table.configure(&self.settings);
    }
}

/// Drags the menu content: moving the pointer up scrolls down.
fn scroll_to(m: &mut MenuState, y: f32) {
    if let Some((start_y, start_scroll)) = m.drag_anchor {
        m.scroll = (start_scroll + start_y - y).clamp(0.0, m.menu.max_scroll(m.tab));
    }
}

#[cfg(test)]
#[path = "tests/app.rs"]
mod tests;
