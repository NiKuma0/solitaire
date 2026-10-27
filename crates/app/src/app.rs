//! Ties the table, settings screen, animator and theme together.

use macroquad::math::Vec2;
use solitaire_core::Game;

use crate::anim::{Animator, Cue};
use crate::gesture::Gesture;
use crate::layout::{Button, Insets};
use crate::menu::{Menu, MenuHit, apply};
use crate::scene::placements;
use crate::settings::{AnimSpeed, Settings};
use crate::table::{Event, Table};
use crate::theme::{Palette, ThemeId, palette};

/// Theme crossfade length when animations are on.
const FADE_SECONDS: f32 = 0.3;

struct MenuState {
    menu: Menu,
    screen: Vec2,
    scroll: f32,
    /// Pointer y and scroll when the current drag started.
    drag_anchor: Option<(f32, f32)>,
}

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
    pub const fn screen(&self) -> Vec2 {
        self.screen
    }

    /// The open settings screen and how far it is scrolled.
    #[must_use]
    pub fn menu(&self) -> Option<(&Menu, f32)> {
        self.menu.as_ref().map(|m| (&m.menu, m.scroll))
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
    pub fn press(&mut self, button: Button) {
        if self.menu.is_none() {
            self.table.press(button);
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
            m.scroll = m.scroll.min(m.menu.max_scroll());
        }
        for event in self.table.drain_events() {
            match event {
                Event::OpenSettings => {
                    let menu = Menu::new(screen, self.insets);
                    self.menu = Some(MenuState {
                        menu,
                        screen,
                        scroll: 0.0,
                        drag_anchor: None,
                    });
                }
                Event::Rejected(card) => self.anim.shake(card, now, self.settings.animation),
                Event::NewGame => {
                    self.cue = Cue::Deal;
                    self.anim.stop_win();
                }
                Event::AutoComplete => self.cue = Cue::Cascade,
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
        let Some(m) = &mut self.menu else { return };
        match gesture {
            Gesture::Tap(p) => match m.menu.hit(p, m.scroll) {
                Some(MenuHit::Done) => self.menu = None,
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
        m.scroll = (start_scroll + start_y - y).clamp(0.0, m.menu.max_scroll());
    }
}

#[cfg(test)]
#[path = "tests/app.rs"]
mod tests;
