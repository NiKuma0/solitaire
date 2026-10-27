//! Ties the table, the screens, the animator and the theme together.

use macroquad::math::{Rect, Vec2, vec2};
use solitaire_core::Game;

use crate::anim::{Animator, Cue};
use crate::controls::{Selector, Step, step};
use crate::dialog::{self, DialogHit, DialogLayout};
use crate::feedback::{Feedback, Target};
use crate::gesture::Gesture;
use crate::layout::{Button, Insets};
use crate::main_menu::{Item, MainHit, MainMenu};
use crate::menu::{MenuHit, Row, apply, selected};
use crate::persist;
use crate::picker::{Picker, PickerHit, PickerLayout};
use crate::reveal::Reveal;
use crate::scene::placements;
use crate::settings::{AnimSpeed, Settings};
use crate::settings_screen::{Control, SettingsHit, SettingsScreen};
use crate::stats::{GameRecord, Stats};
use crate::stats_screen::{FILTERS, StatsScreen};
use crate::storage::{MemoryStorage, Storage};
use crate::table::{Event, Table};
use crate::theme::{Palette, ThemeId, palette};
use crate::transition::Transition;

/// Theme crossfade length when animations are on.
const FADE_SECONDS: f32 = 0.3;

/// Room left around a control scrolled into view for the keyboard focus.
const FOCUS_MARGIN: f32 = 16.0;

/// Targets that move with a screen's scrolling content (not the header's).
const fn scrolls(target: Target) -> bool {
    matches!(target, Target::Settings(_) | Target::Stats(_))
}

/// A selector's chevrons that lead somewhere, scrolled up by `scroll`.
fn chevrons(
    s: &Selector,
    index: usize,
    len: usize,
    scroll: f32,
    make: impl Fn(Step) -> Target,
) -> impl Iterator<Item = (Target, Rect)> {
    [(Step::Prev, s.prev), (Step::Next, s.next)]
        .into_iter()
        .filter(move |&(st, _)| step(index, len, st).is_some())
        .map(move |(st, r)| (make(st), r.offset(vec2(0.0, -scroll))))
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
    /// How far the current screen's content is scrolled up.
    scroll: f32,
    /// Pointer y and scroll when a drag on the content began.
    drag: Option<(f32, f32)>,
    /// Which totals the statistics screen shows: an index into `FILTERS`.
    stats_filter: usize,
    /// A first tap on "Reset" asks; a second one before this time resets.
    confirm_reset_until: Option<f64>,
    clock: f64,
    feedback: Feedback,
    picker: Option<Picker>,
    /// "Start new game?", while it is open or closing.
    dialog: Option<Reveal>,
    /// The control the keyboard focus is on; shown only after Tab.
    focus: Option<Target>,
}

/// The full-screen views; the picker and dialog sit on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Game,
    Menu,
    Settings,
    Stats,
    About,
}

/// The statistics screen's state, for drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatsView {
    pub scroll: f32,
    /// Index into `stats_screen::FILTERS`.
    pub filter: usize,
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
    /// Esc: close the picker or dialog, or go back a screen.
    Back,
    /// Tab: move the keyboard focus forward.
    Next,
    /// Shift+Tab: move the keyboard focus back.
    Previous,
    /// Enter: press the focused control.
    Activate,
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
            scroll: 0.0,
            drag: None,
            stats_filter: 0,
            confirm_reset_until: None,
            clock: now,
            feedback: Feedback::default(),
            picker: None,
            dialog: None,
            focus: None,
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
        // Pointer input hides the keyboard focus.
        self.focus = None;
        self.route(gesture, now);
    }

    fn route(&mut self, gesture: Gesture, now: f64) {
        if self.picker.is_some_and(|p| p.reveal.is_open()) {
            self.handle_picker(gesture, now);
        } else if self.dialog.is_some_and(|d| d.is_open()) {
            self.handle_dialog(gesture, now);
        } else if self.view != Screen::Game {
            self.handle_content(gesture, now);
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
        match shortcut {
            Shortcut::Back => return self.back(now),
            Shortcut::Next => return self.move_focus(1),
            Shortcut::Previous => return self.move_focus(-1),
            Shortcut::Activate => return self.activate(now),
            Shortcut::Undo | Shortcut::Redo | Shortcut::NewGame | Shortcut::Draw => {}
        }
        let covered = self.picker.is_some_and(|p| p.reveal.is_open())
            || self.dialog.is_some_and(|d| d.is_open());
        if self.view != Screen::Game || covered {
            return;
        }
        match shortcut {
            Shortcut::Undo => self.table.press(Button::Undo),
            Shortcut::Redo => self.table.press(Button::Redo),
            Shortcut::NewGame => self.new_game(now),
            Shortcut::Draw => self.table.draw_from_stock(),
            Shortcut::Back | Shortcut::Next | Shortcut::Previous | Shortcut::Activate => {}
        }
    }

    /// Esc: the picker or dialog closes first, then screens step back towards the game.
    fn back(&mut self, now: f64) {
        if let Some(picker) = self.picker.as_mut().filter(|p| p.reveal.is_open()) {
            picker.reveal.close(now);
        } else if let Some(dialog) = self.dialog.as_mut().filter(|d| d.is_open()) {
            dialog.close(now);
        } else {
            match self.view {
                Screen::Settings | Screen::Stats | Screen::About => self.go(Screen::Menu, now),
                Screen::Menu => self.back_to_game(now),
                Screen::Game => {}
            }
        }
    }

    /// The mouse is at `p` (`None` when it is not, or on touch screens).
    pub fn hover(&mut self, p: Option<Vec2>) {
        let target = p.and_then(|p| self.target_at(p));
        self.feedback.hover(target);
    }

    /// Something pressable is under the mouse.
    #[must_use]
    pub const fn is_hovering(&self) -> bool {
        self.feedback.hovered().is_some()
    }

    /// Mouse wheel: scrolls the current screen by `dy` (down is positive).
    pub fn wheel(&mut self, dy: f32) {
        if self.view != Screen::Game {
            self.scroll = (self.scroll + dy).clamp(0.0, self.max_scroll());
        }
    }

    /// The focused control and where its ring goes.
    #[must_use]
    pub fn focus(&self) -> Option<(Target, Rect)> {
        let focus = self.focus?;
        self.targets().into_iter().find(|(t, _)| *t == focus)
    }

    fn move_focus(&mut self, by: isize) {
        let targets = self.targets();
        let Some(len) = isize::try_from(targets.len()).ok().filter(|&n| n > 0) else {
            return;
        };
        let at = self
            .focus
            .and_then(|f| targets.iter().position(|(t, _)| *t == f));
        let next = match at {
            Some(i) => (isize::try_from(i).unwrap_or(0) + by).rem_euclid(len),
            None if by > 0 => 0,
            None => len - 1,
        };
        let Some(&(target, rect)) = usize::try_from(next).ok().and_then(|i| targets.get(i)) else {
            return;
        };
        self.focus = Some(target);
        self.scroll_into_view(rect);
    }

    /// Enter: a tap in the middle of the focused control, keeping the focus.
    fn activate(&mut self, now: f64) {
        if let Some((target, rect)) = self.focus() {
            self.route(Gesture::Tap(rect.center()), now);
            self.focus = Some(target);
        }
    }

    /// Scrolls the current screen so `rect` sits inside its viewport.
    fn scroll_into_view(&mut self, rect: Rect) {
        let top = self.table.layout().header.bottom();
        let bottom = self.screen.y;
        let shift = if rect.y < top {
            rect.y - top - FOCUS_MARGIN
        } else if rect.bottom() > bottom {
            rect.bottom() - bottom + FOCUS_MARGIN
        } else {
            0.0
        };
        self.scroll = (self.scroll + shift).clamp(0.0, self.max_scroll());
    }

    /// The pressable thing under `p`, as hover sees it.
    fn target_at(&self, p: Vec2) -> Option<Target> {
        let header = self.table.layout().header.bottom();
        self.targets()
            .into_iter()
            .find(|(t, r)| r.contains(p) && (p.y >= header || !scrolls(*t)))
            .map(|(t, _)| t)
    }

    /// Everything the keyboard can focus on, in order, where it is now.
    fn targets(&self) -> Vec<(Target, Rect)> {
        if let Some(overlay) = self.overlay_targets() {
            return overlay;
        }
        let layout = self.table.layout();
        let button = |b: Button| {
            layout
                .buttons
                .iter()
                .find(|(x, _)| *x == b)
                .map(|&(_, r)| (Target::Button(b), r))
        };
        let mut out: Vec<_> = button(Button::Settings).into_iter().collect();
        match self.view {
            Screen::Game => {
                let bar = [
                    Button::New,
                    Button::Undo,
                    Button::Redo,
                    Button::AutoComplete,
                ];
                out.extend(
                    bar.into_iter()
                        .filter(|&b| self.table.is_enabled(b))
                        .filter_map(button),
                );
            }
            Screen::Menu => {
                let m = MainMenu::new(layout);
                let draw = usize::from(self.settings.draw == solitaire_core::DrawMode::Three);
                let hand = usize::from(!self.settings.left_handed);
                out.extend(chevrons(&m.draw, draw, 2, 0.0, |s| {
                    Target::Main(MainHit::Draw(s))
                }));
                out.extend(chevrons(&m.hand, hand, 2, 0.0, |s| {
                    Target::Main(MainHit::Hand(s))
                }));
                out.extend(
                    m.items
                        .map(|(item, r)| (Target::Main(MainHit::Item(item)), r)),
                );
            }
            Screen::Settings => out.extend(self.settings_targets()),
            Screen::Stats => {
                let s = StatsScreen::new(layout, self.stats.history().len());
                let filter = chevrons(
                    &s.filter,
                    self.stats_filter,
                    FILTERS.len(),
                    self.scroll,
                    Target::Stats,
                );
                out.extend(filter);
            }
            Screen::About => {}
        }
        out.extend(button(Button::Theme));
        out
    }

    /// The open picker's swatches or the dialog's buttons, which take the keyboard over.
    fn overlay_targets(&self) -> Option<Vec<(Target, Rect)>> {
        let layout = self.table.layout();
        if let Some(picker) = self.picker.filter(|p| p.reveal.is_open()) {
            let pl = PickerLayout::new(layout);
            let mut out: Vec<_> = pl
                .swatches
                .iter()
                .enumerate()
                .map(|(i, r)| (i, r.offset(vec2(-picker.scroll, 0.0))))
                .filter(|(_, r)| r.overlaps(&pl.area))
                .map(|(i, r)| (Target::Menu(MenuHit::Choose(Row::Theme, i)), r))
                .collect();
            out.push((Target::Button(Button::Theme), pl.button));
            return Some(out);
        }
        self.dialog.filter(super::reveal::Reveal::is_open).map(|_| {
            let d = DialogLayout::new(layout);
            vec![
                (Target::Dialog(DialogHit::Cancel), d.cancel),
                (Target::Dialog(DialogHit::Confirm), d.confirm),
            ]
        })
    }

    /// Each setting's control, scrolled.
    fn settings_targets(&self) -> Vec<(Target, Rect)> {
        let up = vec2(0.0, -self.scroll);
        let mut out = Vec::new();
        for b in SettingsScreen::new(self.table.layout()).blocks {
            let row = b.row;
            match b.control {
                Control::Toggle(r) => {
                    out.push((Target::Settings(SettingsHit::Toggle(row)), r.offset(up)));
                }
                Control::Button(r) => {
                    out.push((Target::Settings(SettingsHit::Action(row)), r.offset(up)));
                }
                Control::Selector(s) => {
                    let index = selected(&self.settings, row).unwrap_or(0);
                    let len = row.options().len();
                    out.extend(chevrons(&s, index, len, self.scroll, |st| {
                        Target::Settings(SettingsHit::Step(row, st))
                    }));
                }
            }
        }
        out
    }

    /// Once per frame, after input: reacts to what changed and syncs animations.
    pub fn update(&mut self, screen: Vec2, now: f64) {
        self.screen = screen;
        self.table.resize(screen, self.insets);
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
        self.scroll = self.scroll.min(self.max_scroll());

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

    /// Every screen opens at its top.
    fn go(&mut self, to: Screen, now: f64) {
        if to != self.view {
            self.transition = Some(Transition::new(self.view, now, self.settings.animation));
            self.view = to;
            self.scroll = 0.0;
            self.drag = None;
            self.confirm_reset_until = None;
        }
    }

    /// How far the current screen's content can scroll.
    fn max_scroll(&self) -> f32 {
        let layout = self.table.layout();
        match self.view {
            Screen::Settings => SettingsScreen::new(layout).max_scroll(),
            Screen::Stats => StatsScreen::new(layout, self.stats.history().len()).max_scroll(),
            Screen::Game | Screen::Menu | Screen::About => 0.0,
        }
    }

    /// The screens other than the game: drags scroll, taps go to the screen.
    fn handle_content(&mut self, gesture: Gesture, now: f64) {
        self.clock = now;
        let scroll_to = |app: &mut Self, y: f32| {
            if let Some((start_y, start_scroll)) = app.drag {
                app.scroll = (start_scroll + start_y - y).clamp(0.0, app.max_scroll());
            }
        };
        let p = match gesture {
            Gesture::Tap(p) => p,
            Gesture::DragStart { origin, at } => {
                self.drag = Some((origin.y, self.scroll));
                scroll_to(self, at.y);
                return;
            }
            Gesture::DragMove(p) => {
                scroll_to(self, p.y);
                return;
            }
            Gesture::Drop(p) => {
                scroll_to(self, p.y);
                self.drag = None;
                return;
            }
        };
        if self.header_tap(p, now) {
            return;
        }
        match self.view {
            Screen::Menu => self.tap_main_menu(p, now),
            Screen::Settings => self.tap_settings(p, now),
            Screen::Stats => {
                let games = self.stats.history().len();
                let hit = StatsScreen::new(self.table.layout(), games).hit(p, self.scroll);
                if let Some(s) = hit
                    && let Some(i) = step(self.stats_filter, FILTERS.len(), s)
                {
                    self.feedback.press(Target::Stats(s), now);
                    self.stats_filter = i;
                }
            }
            Screen::Game | Screen::About => {}
        }
    }

    /// Selectors and the three items.
    fn tap_main_menu(&mut self, p: Vec2, now: f64) {
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
                    Screen::Settings | Screen::Stats | Screen::About => self.go(Screen::Menu, now),
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

    /// Each menu item opens its own screen.
    fn open_item(&mut self, item: Item, now: f64) {
        let to = match item {
            Item::Settings => Screen::Settings,
            Item::Stats => Screen::Stats,
            Item::About => Screen::About,
        };
        self.go(to, now);
    }

    /// The settings screen's state, for drawing.
    #[must_use]
    pub fn settings_view(&self) -> SettingsView {
        SettingsView {
            scroll: self.scroll,
            confirm_reset: self.confirm_reset_until.is_some_and(|t| self.clock < t),
        }
    }

    /// The statistics screen's state, for drawing.
    #[must_use]
    pub fn stats_view(&self) -> StatsView {
        StatsView {
            scroll: self.scroll,
            filter: self.stats_filter,
        }
    }

    /// Controls change settings; Restart and Reset act.
    fn tap_settings(&mut self, p: Vec2, now: f64) {
        let screen = SettingsScreen::new(self.table.layout());
        let Some(hit) = screen.hit(p, self.scroll) else {
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

#[cfg(test)]
#[path = "tests/app.rs"]
mod tests;
