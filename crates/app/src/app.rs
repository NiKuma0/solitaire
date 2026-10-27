//! Ties the table, settings screen, animator and theme together.

use macroquad::math::Vec2;
use solitaire_core::Game;

use crate::anim::{Animator, Cue};
use crate::controls::step;
use crate::dialog::{self, DialogHit, DialogLayout};
use crate::feedback::{Feedback, Target};
use crate::gesture::Gesture;
use crate::layout::{Button, Insets};
use crate::main_menu::{Item, MainHit, MainMenu};
use crate::menu::{Menu, MenuHit, Row, Tab, apply, selected};
use crate::persist;
use crate::picker::{Picker, PickerHit, PickerLayout};
use crate::reveal::Reveal;
use crate::scene::placements;
use crate::settings::{AnimSpeed, Settings};
use crate::settings_screen::{SettingsHit, SettingsScreen};
use crate::stats::{GameRecord, Stats};
use crate::storage::{MemoryStorage, Storage};
use crate::table::{Event, Table};
use crate::theme::{Palette, ThemeId, palette};
use crate::transition::Transition;

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
    view: Screen,
    /// Fading from the previous screen to `view`.
    transition: Option<Transition<Screen>>,
    /// Draw mode changed on this visit to the menu screen.
    draw_changed: bool,
    settings_scroll: f32,
    /// Pointer y and scroll when a drag on the settings screen began.
    settings_drag: Option<(f32, f32)>,
    /// A first tap on "Reset" asks; a second one before this time resets.
    confirm_reset_until: Option<f64>,
    clock: f64,
    feedback: Feedback,
    picker: Option<Picker>,
    /// "Start new game?", while it is open or closing.
    dialog: Option<Reveal>,
}

/// The full-screen views; the settings panel and dialogs sit on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Game,
    Menu,
    Settings,
}

/// The settings screen's state, for drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettingsView {
    pub scroll: f32,
    /// "Reset" was tapped once and waits for a second tap.
    pub confirm_reset: bool,
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
            view: Screen::Game,
            transition: None,
            draw_changed: false,
            settings_scroll: 0.0,
            settings_drag: None,
            confirm_reset_until: None,
            clock: now,
            feedback: Feedback::default(),
            picker: None,
            dialog: None,
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

    /// How lit `target` is by a recent press, 0 to 1.
    #[must_use]
    pub fn feedback(&self, target: Target, now: f64) -> f32 {
        self.feedback.level(target, now)
    }

    /// The theme picker, while it is open or closing.
    #[must_use]
    pub const fn picker(&self) -> Option<&Picker> {
        self.picker.as_ref()
    }

    /// The new game dialog, while it is open or closing.
    #[must_use]
    pub const fn dialog(&self) -> Option<&Reveal> {
        self.dialog.as_ref()
    }

    pub fn handle(&mut self, gesture: Gesture, now: f64) {
        if self.menu.is_some() {
            self.handle_menu(gesture, now);
        } else if self.picker.is_some_and(|p| p.reveal.is_open()) {
            self.handle_picker(gesture, now);
        } else if self.dialog.is_some_and(|d| d.is_open()) {
            self.handle_dialog(gesture, now);
        } else if self.view == Screen::Menu {
            self.handle_main_menu(gesture, now);
        } else if self.view == Screen::Settings {
            self.handle_settings(gesture, now);
        } else if let Gesture::Tap(p) = gesture
            && self.button_at(p) == Some(Button::New)
        {
            self.feedback.press(Target::Button(Button::New), now);
            self.new_game(now);
        } else {
            if let Gesture::Tap(p) = gesture
                && let Some(&(button, _)) = self
                    .table
                    .layout()
                    .buttons
                    .iter()
                    .find(|(_, r)| r.contains(p))
                && self.table.is_enabled(button)
            {
                self.feedback.press(Target::Button(button), now);
            }
            self.table.set_clock(now);
            self.table.handle(gesture);
        }
    }

    /// Keyboard shortcuts and other direct button presses.
    /// A keyboard shortcut (SPEC §8). Ignored while the menu is open.
    pub fn shortcut(&mut self, shortcut: Shortcut, now: f64) {
        if self.view != Screen::Game || self.dialog.is_some_and(|d| d.is_open()) {
            return;
        }
        match shortcut {
            Shortcut::Undo => self.table.press(Button::Undo),
            Shortcut::Redo => self.table.press(Button::Redo),
            Shortcut::NewGame => self.new_game(now),
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
                    self.draw_changed = false;
                    self.go(Screen::Menu, now);
                }
                Event::OpenThemes => {
                    self.picker = Some(Picker::open(now, self.settings.animation));
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

        if self.picker.is_some_and(|p| p.reveal.is_gone(now)) {
            self.picker = None;
        }
        if self.dialog.is_some_and(|d| d.is_gone(now)) {
            self.dialog = None;
        }
        if self.transition.is_some_and(|t| t.is_done(now)) {
            self.transition = None;
        }
        if self.view == Screen::Settings {
            let max = SettingsScreen::new(self.table.layout()).max_scroll();
            self.settings_scroll = self.settings_scroll.min(max);
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
            Gesture::Tap(p) => match m.menu.hit(p, m.scroll, m.tab).inspect(|&hit| {
                self.feedback.press(Target::Menu(hit), now);
            }) {
                Some(MenuHit::Done) => {
                    self.menu = None;
                }
                Some(MenuHit::Action(Row::Restart)) => {
                    self.menu = None;
                    self.table.restart();
                    self.go(Screen::Game, now);
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
                // Tabs included: settings ignore them, and the panel only shows statistics now.
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

    /// The screen the player is on (or heading to).
    #[must_use]
    pub const fn view(&self) -> Screen {
        self.view
    }

    /// The screen to draw at `now` and its opacity, mid-transition.
    #[must_use]
    pub fn shown_screen(&self, now: f64) -> (Screen, f32) {
        self.transition
            .map_or((self.view, 1.0), |t| t.frame(self.view, now))
    }

    fn go(&mut self, to: Screen, now: f64) {
        if to != self.view {
            self.transition = Some(Transition::new(self.view, now, self.settings.animation));
            self.view = to;
        }
    }

    /// Back arrow, palette, selectors and the three items; everything else is ignored.
    fn handle_main_menu(&mut self, gesture: Gesture, now: f64) {
        let Gesture::Tap(p) = gesture else {
            return;
        };
        if self.header_tap(p, now) {
            return;
        }
        let Some(hit) = MainMenu::new(self.table.layout()).hit(p) else {
            return;
        };
        self.feedback.press(Target::Main(hit), now);
        match hit {
            MainHit::Draw(s) => {
                let current = usize::from(self.settings.draw == solitaire_core::DrawMode::Three);
                if let Some(i) = step(current, 2, s) {
                    self.change_setting(MenuHit::Choose(Row::Draw, i), now);
                    self.draw_changed = true;
                }
            }
            MainHit::Hand(s) => {
                let current = usize::from(!self.settings.left_handed);
                if step(current, 2, s).is_some() {
                    self.change_setting(MenuHit::Toggle(Row::LeftHanded), now);
                }
            }
            MainHit::Item(item) => self.open_item(item, now),
        }
    }

    /// The back arrow and the palette, on screens other than the game.
    /// True if `p` was one of them.
    fn header_tap(&mut self, p: Vec2, now: f64) -> bool {
        match self.button_at(p) {
            Some(Button::Settings) => {
                self.feedback.press(Target::Button(Button::Settings), now);
                match self.view {
                    Screen::Settings => self.go(Screen::Menu, now),
                    Screen::Menu | Screen::Game => self.back_to_game(now),
                }
                true
            }
            Some(Button::Theme) => {
                self.picker = Some(Picker::open(now, self.settings.animation));
                true
            }
            _ => false,
        }
    }

    /// Where a menu item leads: the settings screen (About is at its end) or the statistics.
    fn open_item(&mut self, item: Item, now: f64) {
        match item {
            Item::Settings | Item::About => {
                self.settings_scroll = if item == Item::About {
                    SettingsScreen::new(self.table.layout()).max_scroll()
                } else {
                    0.0
                };
                self.confirm_reset_until = None;
                self.go(Screen::Settings, now);
            }
            Item::Stats => {
                self.menu = Some(MenuState {
                    menu: Menu::new(self.screen, self.insets),
                    screen: self.screen,
                    tab: Tab::Stats,
                    scroll: 0.0,
                    drag_anchor: None,
                    confirm_reset_until: None,
                });
            }
        }
    }

    /// The settings screen's state, for drawing.
    #[must_use]
    pub fn settings_view(&self) -> SettingsView {
        SettingsView {
            scroll: self.settings_scroll,
            confirm_reset: self.confirm_reset_until.is_some_and(|t| self.clock < t),
        }
    }

    /// Controls change settings; drags scroll; Restart and Reset act.
    fn handle_settings(&mut self, gesture: Gesture, now: f64) {
        self.clock = now;
        let screen = SettingsScreen::new(self.table.layout());
        let scroll_to = |app: &mut Self, y: f32| {
            if let Some((start_y, start_scroll)) = app.settings_drag {
                app.settings_scroll = (start_scroll + start_y - y).clamp(0.0, screen.max_scroll());
            }
        };
        let p = match gesture {
            Gesture::Tap(p) => p,
            Gesture::DragStart { origin, at } => {
                self.settings_drag = Some((origin.y, self.settings_scroll));
                scroll_to(self, at.y);
                return;
            }
            Gesture::DragMove(p) => {
                scroll_to(self, p.y);
                return;
            }
            Gesture::Drop(p) => {
                scroll_to(self, p.y);
                self.settings_drag = None;
                return;
            }
        };
        if self.header_tap(p, now) {
            return;
        }
        let Some(hit) = screen.hit(p, self.settings_scroll) else {
            return;
        };
        self.feedback.press(Target::Settings(hit), now);
        match hit {
            SettingsHit::Toggle(row) => self.change_setting(MenuHit::Toggle(row), now),
            SettingsHit::Step(row, s) => {
                let current = selected(&self.settings, row).unwrap_or(0);
                if let Some(i) = step(current, row.options().len(), s) {
                    self.change_setting(MenuHit::Choose(row, i), now);
                }
            }
            SettingsHit::Action(Row::Restart) => {
                self.table.restart();
                self.go(Screen::Game, now);
            }
            SettingsHit::Action(Row::ResetStats) => {
                if self.confirm_reset_until.is_some_and(|t| now < t) {
                    self.confirm_reset_until = None;
                    self.stats.reset();
                    let result = persist::save_stats(self.storage.as_mut(), &self.stats);
                    self.save_failed = result.is_err();
                } else {
                    self.confirm_reset_until = Some(now + CONFIRM_SECONDS);
                }
            }
            SettingsHit::Action(_) => {}
        }
    }

    /// Back to the game; a new draw mode deals at once, or asks if the game is under way.
    fn back_to_game(&mut self, now: f64) {
        self.go(Screen::Game, now);
        if std::mem::take(&mut self.draw_changed)
            && self.settings.draw != self.table.game().draw_mode()
        {
            self.new_game(now);
        }
    }

    fn button_at(&self, p: Vec2) -> Option<Button> {
        let buttons = &self.table.layout().buttons;
        buttons.iter().find(|(_, r)| r.contains(p)).map(|&(b, _)| b)
    }

    /// Deals a new game, first asking if the current one is under way.
    fn new_game(&mut self, now: f64) {
        let game = self.table.game();
        if game.board().moves > 0 && !game.is_won() {
            self.dialog = Some(Reveal::open(
                now,
                self.settings.animation,
                dialog::OPEN_SECONDS,
            ));
        } else {
            self.table.press(Button::New);
        }
    }

    /// Cancel, a tap off the panel or any drag closes the dialog; confirm deals.
    fn handle_dialog(&mut self, gesture: Gesture, now: f64) {
        let Some(reveal) = &mut self.dialog else {
            return;
        };
        let Gesture::Tap(p) = gesture else {
            if matches!(gesture, Gesture::DragStart { .. }) {
                reveal.close(now);
            }
            return;
        };
        let hit = DialogLayout::new(self.table.layout()).hit(p);
        if let Some(h @ (DialogHit::Cancel | DialogHit::Confirm)) = hit {
            self.feedback.press(Target::Dialog(h), now);
        }
        match hit {
            Some(DialogHit::Panel) => {}
            Some(DialogHit::Confirm) => {
                reveal.close(now);
                self.table.press(Button::New);
            }
            Some(DialogHit::Cancel) | None => reveal.close(now),
        }
    }

    /// Swatches pick a theme, drags scroll them, anything outside closes the picker.
    fn handle_picker(&mut self, gesture: Gesture, now: f64) {
        let layout = PickerLayout::new(self.table.layout());
        let Some(picker) = &mut self.picker else {
            return;
        };
        let scroll_to = |picker: &mut Picker, x: f32| {
            if let Some((start_x, start_scroll)) = picker.drag_anchor {
                picker.scroll = (start_scroll + start_x - x).clamp(0.0, layout.max_scroll());
            }
        };
        match gesture {
            Gesture::Tap(p) => match layout.hit(p, picker.scroll) {
                Some(PickerHit::Swatch(i)) => {
                    self.feedback
                        .press(Target::Menu(MenuHit::Choose(Row::Theme, i)), now);
                    self.change_setting(MenuHit::Choose(Row::Theme, i), now);
                }
                Some(PickerHit::Capsule) => {}
                Some(PickerHit::Button) | None => picker.reveal.close(now),
            },
            Gesture::DragStart { origin, at } => {
                if layout.capsule.contains(origin) {
                    picker.drag_anchor = Some((origin.x, picker.scroll));
                    scroll_to(picker, at.x);
                } else {
                    picker.reveal.close(now);
                }
            }
            Gesture::DragMove(p) => scroll_to(picker, p.x),
            Gesture::Drop(p) => {
                scroll_to(picker, p.x);
                picker.drag_anchor = None;
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
