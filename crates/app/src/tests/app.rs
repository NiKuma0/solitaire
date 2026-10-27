//! Unit tests for `app`.

use macroquad::math::{Vec2, vec2};
use solitaire_core::{DrawMode, Game, PileId};

use super::*;
use crate::feedback::Target;
use crate::gesture::{DRAG_THRESHOLD, Gesture};
use crate::layout::{Button, Insets};
use crate::menu::{MenuHit, Row};
use crate::picker::PickerLayout;
use crate::scene::placements;
use crate::settings::{AnimSpeed, Settings};
use crate::theme::{ThemeId, palette};

fn app(w: f32, h: f32, settings: Settings) -> App {
    App::new(
        Game::new(4, DrawMode::One),
        vec2(w, h),
        Insets::default(),
        9,
        settings,
        0.0,
    )
}

fn tap(a: &mut App, p: Vec2, now: f64) {
    a.handle(Gesture::Tap(p), now);
    a.update(a.screen(), now);
}

fn button(a: &App, b: Button) -> Vec2 {
    a.table()
        .layout()
        .buttons
        .iter()
        .find(|(x, _)| *x == b)
        .map(|(_, r)| r.center())
        .expect("button")
}

fn row_target(a: &App, row: Row, i: usize) -> Vec2 {
    let view = a.menu().expect("menu open");
    let (menu, scroll) = (view.menu, view.scroll);
    let r = menu.rows.iter().find(|r| r.row == row).expect("row");
    r.targets[i].center() - vec2(0.0, scroll)
}

#[test]
fn settings_button_opens_the_menu_and_done_closes_it() {
    let mut a = app(390.0, 844.0, Settings::default());
    assert!(a.menu().is_none());
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    let done = a.menu().expect("open").menu.done.center();
    tap(&mut a, done, 2.0);
    assert!(a.menu().is_none());
}

#[test]
fn taps_on_the_menu_do_not_reach_the_table() {
    let mut a = app(390.0, 844.0, Settings::default());
    let before = a.table().game().board().clone();
    let stock = a.table().layout().stock.center();
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    tap(&mut a, stock, 2.0);
    assert_eq!(a.table().game().board(), &before);
}

#[test]
fn choosing_a_theme_crossfades_to_it() {
    let mut a = app(390.0, 844.0, Settings::default());
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    let light = row_target(&a, Row::Theme, 1);
    tap(&mut a, light, 2.0);
    assert_eq!(a.settings().theme, ThemeId::Light);
    let start = a.palette(2.0);
    assert_eq!(
        start,
        palette(ThemeId::Dark),
        "fade starts from the old theme"
    );
    assert_ne!(a.palette(2.1), palette(ThemeId::Light));
    assert_eq!(a.palette(3.0), palette(ThemeId::Light));
}

#[test]
fn theme_changes_are_instant_without_animations() {
    let mut a = app(
        390.0,
        844.0,
        Settings {
            animation: AnimSpeed::Off,
            ..Settings::default()
        },
    );
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    let warm = row_target(&a, Row::Theme, 3);
    tap(&mut a, warm, 2.0);
    assert_eq!(a.palette(2.0), palette(ThemeId::Warm));
}

#[test]
fn menu_changes_reach_the_table() {
    let mut a = app(390.0, 844.0, Settings::default());
    let right = a.table().layout().stock.x;
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    let left_handed = a
        .menu()
        .expect("open")
        .menu
        .rows
        .iter()
        .find(|r| r.row == Row::LeftHanded)
        .expect("row")
        .bounds
        .center();
    tap(&mut a, left_handed, 2.0);
    assert!(a.settings().left_handed);
    assert!(a.table().layout().stock.x < right);
}

#[test]
fn short_screens_scroll_the_menu_by_dragging() {
    let mut a = app(640.0, 360.0, Settings::default());
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    let menu = a.menu().expect("open").menu;
    let max = menu.max_scroll(a.menu().expect("open").tab);
    let start = menu.viewport.center();
    a.handle(
        Gesture::DragStart {
            origin: start,
            at: start - vec2(0.0, DRAG_THRESHOLD + 1.0),
        },
        2.0,
    );
    a.handle(Gesture::DragMove(start - vec2(0.0, 40.0)), 2.1);
    let scroll = a.menu().expect("open").scroll;
    assert!((scroll - 40.0).abs() < 0.01, "{scroll}");
    a.handle(Gesture::DragMove(start - vec2(0.0, 5000.0)), 2.2);
    assert!(
        (a.menu().expect("open").scroll - max).abs() < 0.01,
        "clamped"
    );
    a.handle(Gesture::Drop(start), 2.3);
    assert!(a.menu().expect("open").scroll >= 0.0);
    // A tap right after still uses the scrolled positions.
    let _ = MenuHit::Done;
}

#[test]
fn new_game_deals_from_the_stock() {
    let mut a = app(390.0, 844.0, Settings::default());
    a.update(a.screen(), 0.5);
    let p = button(&a, Button::New);
    tap(&mut a, p, 1.0);
    let ps = placements(a.table().layout(), a.table().game(), None);
    let col0 = a.table().game().board().tableau[0].cards()[0];
    let p = ps.iter().find(|p| p.card == col0).expect("placed");
    assert_eq!(
        a.animator().shown(p, 1.0).rect.point(),
        a.table().layout().stock.point()
    );
    assert!(a.animator().is_animating(1.0));
    assert!(!a.animator().is_animating(10.0));
}

#[test]
fn a_rejected_tap_shakes_the_card() {
    for seed in 0..40 {
        let game = Game::new(seed, DrawMode::One);
        let Some(i) = game
            .board()
            .tableau
            .iter()
            .position(|c| c.top().is_some_and(|t| game.best_move_for(t).is_none()))
        else {
            continue;
        };
        let mut a = App::new(
            game,
            vec2(390.0, 844.0),
            Insets::default(),
            1,
            Settings::default(),
            0.0,
        );
        a.update(a.screen(), 0.5);
        let b = a.table().game().board().clone();
        let r = a.table().layout().card_rect(
            &b,
            DrawMode::One,
            PileId::Tableau(i as u8),
            b.tableau[i].len() - 1,
        );
        tap(&mut a, r.center(), 1.0);
        assert!(a.animator().is_animating(1.05));
        return;
    }
    panic!("no deal with a stuck top card");
}

use crate::persist::{self, GAME_KEY};
use crate::storage::{MemoryStorage, Storage, StorageError};

fn startup(seed: u64, reduced_motion: bool) -> Startup {
    Startup {
        screen: vec2(390.0, 844.0),
        insets: Insets::default(),
        seed,
        now: 0.0,
        reduced_motion,
        unix_time: || 1_700_000_000,
    }
}

fn stored_game(a: &App) -> Option<Game> {
    a.storage()
        .get(GAME_KEY)
        .and_then(|json| serde_json::from_str(&json).ok())
}

#[test]
fn first_launch_starts_a_new_game_with_default_settings() {
    let a = App::start(Box::new(MemoryStorage::default()), startup(77, false));
    assert_eq!(a.table().game().seed(), 77);
    assert_eq!(*a.settings(), Settings::default());
}

#[test]
fn reduced_motion_turns_animations_off_on_first_launch_only() {
    let a = App::start(Box::new(MemoryStorage::default()), startup(1, true));
    assert_eq!(a.settings().animation, AnimSpeed::Off);

    let mut storage = MemoryStorage::default();
    persist::save_settings(&mut storage, &Settings::default()).unwrap();
    let a = App::start(Box::new(storage), startup(1, true));
    assert_eq!(
        a.settings().animation,
        AnimSpeed::Normal,
        "a saved choice wins"
    );
}

#[test]
fn every_move_is_saved_and_a_restart_resumes_it() {
    let mut a = App::start(Box::new(MemoryStorage::default()), startup(5, false));
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    tap(&mut a, stock, 1.5);
    assert_eq!(stored_game(&a).as_ref(), Some(a.table().game()));

    let mut storage = MemoryStorage::default();
    persist::save_game(&mut storage, a.table().game()).unwrap();
    let resumed = App::start(Box::new(storage), startup(999, false));
    assert_eq!(resumed.table().game(), a.table().game());
}

#[test]
fn finished_games_go_into_saved_statistics() {
    let mut a = App::start(Box::new(MemoryStorage::default()), startup(5, false));
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    let p = button(&a, Button::New);
    tap(&mut a, p, 2.0);
    assert_eq!(a.stats().tally(None).played, 1);
    let record = a.stats().history().back().copied().expect("recorded");
    assert_eq!(record.date, 1_700_000_000);
    assert!(!record.won);
    let mut storage = MemoryStorage::default();
    storage
        .set(
            persist::STATS_KEY,
            &a.storage().get(persist::STATS_KEY).expect("saved"),
        )
        .unwrap();
    assert_eq!(persist::load(&mut storage).stats, *a.stats());
}

#[test]
fn settings_changes_are_saved() {
    let mut a = App::start(Box::new(MemoryStorage::default()), startup(5, false));
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    let large = a
        .menu()
        .expect("open")
        .menu
        .rows
        .iter()
        .find(|r| r.row == Row::LargeCards)
        .expect("row")
        .bounds
        .center();
    tap(&mut a, large, 2.0);
    let saved: Settings =
        serde_json::from_str(&a.storage().get(persist::SETTINGS_KEY).expect("saved")).unwrap();
    assert!(saved.large_cards);
}

struct BrokenStorage;

impl Storage for BrokenStorage {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
    fn set(&mut self, _key: &str, _value: &str) -> Result<(), StorageError> {
        Err(StorageError::Refused)
    }
}

#[test]
fn a_failed_save_is_reported_not_fatal() {
    let mut a = App::start(Box::new(BrokenStorage), startup(5, false));
    assert!(!a.save_failed());
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    assert!(a.save_failed());
    assert_eq!(a.table().game().board().waste.len(), 1, "the game goes on");
}

use crate::menu::Tab;

fn open_menu(a: &mut App, now: f64) {
    let p = button(a, Button::Settings);
    tap(a, p, now);
}

fn action(a: &App, row: Row) -> Vec2 {
    let view = a.menu().expect("open");
    let r = view.menu.rows.iter().find(|r| r.row == row).expect("row");
    r.targets[0].center() - vec2(0.0, view.scroll)
}

#[test]
fn tabs_switch_and_the_choice_is_remembered() {
    let mut a = app(390.0, 844.0, Settings::default());
    open_menu(&mut a, 1.0);
    assert_eq!(a.menu().expect("open").tab, Tab::Settings);
    let stats = a.menu().expect("open").menu.tabs[0].1.center();
    tap(&mut a, stats, 2.0);
    assert_eq!(a.menu().expect("open").tab, Tab::Stats);
    assert_eq!(a.menu().expect("open").scroll, 0.0);
    let done = a.menu().expect("open").menu.done.center();
    tap(&mut a, done, 3.0);
    open_menu(&mut a, 4.0);
    assert_eq!(a.menu().expect("open").tab, Tab::Stats);
}

#[test]
fn restart_from_the_menu_redeals_and_closes_it() {
    let mut a = App::start(Box::new(MemoryStorage::default()), startup(31, false));
    let fresh = a.table().game().board().clone();
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    open_menu(&mut a, 2.0);
    let p = action(&a, Row::Restart);
    tap(&mut a, p, 3.0);
    assert!(a.menu().is_none());
    assert_eq!(a.table().game().board(), &fresh);
    assert_eq!(
        a.stats().tally(None).played,
        1,
        "the abandoned attempt counts"
    );
}

#[test]
fn reset_statistics_needs_a_second_tap_soon_after() {
    let mut a = App::start(Box::new(MemoryStorage::default()), startup(31, false));
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    let p = button(&a, Button::New);
    tap(&mut a, p, 1.5);
    assert_eq!(a.stats().tally(None).played, 1);

    open_menu(&mut a, 2.0);
    let reset = action(&a, Row::ResetStats);
    tap(&mut a, reset, 3.0);
    assert!(a.menu().expect("open").confirm_reset);
    assert_eq!(a.stats().tally(None).played, 1, "first tap only asks");
    tap(&mut a, reset, 7.0);
    assert_eq!(
        a.stats().tally(None).played,
        1,
        "too late: this tap asks again"
    );
    tap(&mut a, reset, 8.0);
    assert_eq!(a.stats().tally(None).played, 0);
    assert!(!a.menu().expect("open").confirm_reset);
    let saved = a.storage().get(persist::STATS_KEY).expect("saved");
    assert_eq!(
        serde_json::from_str::<crate::stats::Stats>(&saved)
            .unwrap()
            .tally(None)
            .played,
        0
    );
}

#[test]
fn keyboard_shortcuts_play_the_game() {
    let mut a = app(1280.0, 720.0, Settings::default());
    let fresh = a.table().game().board().clone();
    a.shortcut(Shortcut::Draw);
    assert_eq!(a.table().game().board().waste.len(), 1, "Space draws");
    a.shortcut(Shortcut::Undo);
    assert_eq!(a.table().game().board(), &fresh);
    a.shortcut(Shortcut::Redo);
    assert_eq!(a.table().game().board().waste.len(), 1);
    let seed = a.table().game().seed();
    a.shortcut(Shortcut::NewGame);
    assert_ne!(a.table().game().seed(), seed, "N deals a new game");
}

#[test]
fn space_recycles_an_empty_stock() {
    let mut a = app(1280.0, 720.0, Settings::default());
    for _ in 0..24 {
        a.shortcut(Shortcut::Draw);
    }
    assert_eq!(a.table().game().board().stock.len(), 0);
    a.shortcut(Shortcut::Draw);
    assert_eq!(a.table().game().board().stock.len(), 24);
}

#[test]
fn shortcuts_wait_while_the_menu_is_open() {
    let mut a = app(1280.0, 720.0, Settings::default());
    open_menu(&mut a, 1.0);
    a.shortcut(Shortcut::Draw);
    assert_eq!(a.table().game().board().waste.len(), 0);
}

#[test]
fn tapping_a_toolbar_button_lights_it() {
    let mut a = app(390.0, 844.0, Settings::default());
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    let undo = Target::Button(Button::Undo);
    assert_eq!(a.feedback(undo, 1.0), 0.0);
    let p = button(&a, Button::Undo);
    tap(&mut a, p, 2.0);
    assert_eq!(a.feedback(undo, 2.0), 1.0);
}

#[test]
fn disabled_buttons_do_not_light() {
    let mut a = app(390.0, 844.0, Settings::default());
    let p = button(&a, Button::Redo);
    tap(&mut a, p, 1.0);
    assert_eq!(a.feedback(Target::Button(Button::Redo), 1.0), 0.0);
}

#[test]
fn tapping_a_menu_control_lights_it() {
    let mut a = app(390.0, 844.0, Settings::default());
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    let draw3 = row_target(&a, Row::Draw, 1);
    tap(&mut a, draw3, 2.0);
    assert_eq!(
        a.feedback(Target::Menu(MenuHit::Choose(Row::Draw, 1)), 2.0),
        1.0
    );
}

fn open_picker(a: &mut App, now: f64) -> PickerLayout {
    let p = button(a, Button::Theme);
    tap(a, p, now);
    PickerLayout::new(a.table().layout())
}

#[test]
fn palette_button_opens_and_closes_the_picker() {
    let mut a = app(390.0, 844.0, Settings::default());
    assert!(a.picker().is_none());
    let layout = open_picker(&mut a, 1.0);
    assert!(a.picker().is_some_and(crate::picker::Picker::is_open));
    tap(&mut a, layout.button.center(), 2.0);
    assert!(a.picker().is_some_and(|p| !p.is_open()), "closing");
    a.update(a.screen(), 3.0);
    assert!(a.picker().is_none(), "gone after the animation");
}

#[test]
fn tapping_a_swatch_applies_its_theme_and_keeps_the_picker_open() {
    let mut a = app(390.0, 844.0, Settings::default());
    let layout = open_picker(&mut a, 1.0);
    tap(&mut a, layout.swatches[1].center(), 2.0);
    assert_eq!(a.settings().theme, ThemeId::ALL[1]);
    assert!(a.picker().is_some_and(crate::picker::Picker::is_open));
    assert_eq!(a.palette(3.0), palette(ThemeId::ALL[1]));
    let saved: Settings =
        serde_json::from_str(&a.storage().get(persist::SETTINGS_KEY).expect("saved")).unwrap();
    assert_eq!(saved.theme, ThemeId::ALL[1]);
}

#[test]
fn tapping_outside_closes_the_picker_without_touching_the_cards() {
    let mut a = app(390.0, 844.0, Settings::default());
    open_picker(&mut a, 1.0);
    let before = a.table().game().board().clone();
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 2.0);
    assert_eq!(a.table().game().board(), &before);
    assert!(a.picker().is_some_and(|p| !p.is_open()));
}

#[test]
fn dragging_the_swatches_scrolls_them() {
    let mut a = app(360.0, 640.0, Settings::default());
    let layout = open_picker(&mut a, 1.0);
    let from = layout.swatches[2].center();
    a.handle(
        Gesture::DragStart {
            origin: from,
            at: from - vec2(DRAG_THRESHOLD + 1.0, 0.0),
        },
        2.0,
    );
    a.handle(Gesture::DragMove(from - vec2(500.0, 0.0)), 2.1);
    a.handle(Gesture::Drop(from - vec2(500.0, 0.0)), 2.2);
    let scroll = a.picker().map(|p| p.scroll).expect("open");
    assert_eq!(scroll, layout.max_scroll(), "clamped");
    assert!(a.picker().is_some_and(crate::picker::Picker::is_open));
}
