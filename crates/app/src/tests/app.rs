//! Unit tests for `app`.

use macroquad::math::{Vec2, vec2};
use solitaire_core::{DrawMode, Game, PileId};

use super::*;
use crate::controls::Step;
use crate::dialog::DialogLayout;
use crate::feedback::Target;
use crate::gesture::{DRAG_THRESHOLD, Gesture};
use crate::layout::{Button, Insets};
use crate::main_menu::{Item, MainHit, MainMenu};
use crate::menu::Row;
use crate::picker::PickerLayout;
use crate::reveal::Reveal;
use crate::scene::placements;
use crate::settings::{AnimSpeed, Settings};
use crate::settings_screen::{Control, SettingsHit, SettingsScreen};
use crate::stats_screen::StatsScreen;
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

/// Taps New and, if asked, confirms.
fn start_new_game(a: &mut App, now: f64) {
    let p = button(a, Button::New);
    tap(a, p, now);
    if a.dialog().is_some_and(Reveal::is_open) {
        let confirm = DialogLayout::new(a.table().layout()).confirm.center();
        tap(a, confirm, now + 0.5);
    }
}

/// Opens the menu screen from the game.
fn open_menu_screen(a: &mut App, now: f64) -> MainMenu {
    let p = button(a, Button::Settings);
    tap(a, p, now);
    MainMenu::new(a.table().layout())
}

fn item(m: &MainMenu, item: Item) -> Vec2 {
    m.items
        .iter()
        .find(|(i, _)| *i == item)
        .map(|(_, r)| r.center())
        .expect("item")
}

/// Opens the settings panel through the menu screen.
fn open_settings(a: &mut App, now: f64) {
    let m = open_menu_screen(a, now);
    tap(a, item(&m, Item::Settings), now);
}

/// Where `row`'s control is now, after scrolling the settings screen to show it.
fn row_control(a: &mut App, row: Row, now: f64) -> Vec2 {
    let screen = SettingsScreen::new(a.table().layout());
    let target = screen.control_center(row);
    let view = screen.viewport;
    let want = (target.y - view.center().y).clamp(0.0, screen.max_scroll());
    let from = view.center();
    let by = want - a.settings_view().scroll;
    if by.abs() > 0.5 {
        a.handle(
            Gesture::DragStart {
                origin: from,
                at: from,
            },
            now,
        );
        a.handle(Gesture::Drop(from - vec2(0.0, by)), now);
    }
    target - vec2(0.0, a.settings_view().scroll)
}

fn tap_row(a: &mut App, row: Row, now: f64) {
    let p = row_control(a, row, now);
    tap(a, p, now);
}

#[test]
fn settings_open_from_the_menu_screen_and_back_returns_to_it() {
    let mut a = app(390.0, 844.0, Settings::default());
    open_settings(&mut a, 1.0);
    assert_eq!(a.view(), Screen::Settings);
    assert_eq!(a.settings_view().scroll, 0.0);
    let back = button(&a, Button::Settings);
    tap(&mut a, back, 2.0);
    assert_eq!(a.view(), Screen::Menu);
}

#[test]
fn taps_on_the_menu_do_not_reach_the_table() {
    let mut a = app(390.0, 844.0, Settings::default());
    let before = a.table().game().board().clone();
    let stock = a.table().layout().stock.center();
    open_settings(&mut a, 1.0);
    tap(&mut a, stock, 2.0);
    assert_eq!(a.table().game().board(), &before);
}

#[test]
fn choosing_a_theme_crossfades_to_it() {
    let mut a = app(390.0, 844.0, Settings::default());
    let picker = open_picker(&mut a, 1.0);
    tap(&mut a, picker.swatches[1].center(), 2.0);
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
    let picker = open_picker(&mut a, 1.0);
    tap(&mut a, picker.swatches[3].center(), 2.0);
    assert_eq!(a.palette(2.0), palette(ThemeId::Warm));
}

#[test]
fn menu_changes_reach_the_table() {
    let mut a = app(390.0, 844.0, Settings::default());
    let right = a.table().layout().stock.x;
    let m = open_menu_screen(&mut a, 1.0);
    tap(&mut a, m.hand.prev.center(), 2.0);
    assert!(a.settings().left_handed);
    assert!(a.table().layout().stock.x < right);
}

#[test]
fn the_settings_screen_scrolls_by_dragging() {
    let mut a = app(640.0, 360.0, Settings::default());
    open_settings(&mut a, 1.0);
    let screen = SettingsScreen::new(a.table().layout());
    let max = screen.max_scroll();
    assert!(max > 0.0);
    let start = screen.viewport.center();
    a.handle(
        Gesture::DragStart {
            origin: start,
            at: start - vec2(0.0, DRAG_THRESHOLD + 1.0),
        },
        2.0,
    );
    a.handle(Gesture::DragMove(start - vec2(0.0, 40.0)), 2.1);
    let scroll = a.settings_view().scroll;
    assert!((scroll - 40.0).abs() < 0.01, "{scroll}");
    a.handle(Gesture::DragMove(start - vec2(0.0, 5000.0)), 2.2);
    assert!((a.settings_view().scroll - max).abs() < 0.01, "clamped");
    a.handle(Gesture::DragMove(start + vec2(0.0, 5000.0)), 2.3);
    a.handle(Gesture::Drop(start + vec2(0.0, 5000.0)), 2.4);
    assert_eq!(a.settings_view().scroll, 0.0, "clamped at the top");
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
    start_new_game(&mut a, 2.0);
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
    open_settings(&mut a, 1.0);
    tap_row(&mut a, Row::LargeCards, 2.0);
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

#[test]
fn statistics_have_their_own_screen_with_a_filter() {
    // Short enough to scroll even without games.
    let mut a = app(640.0, 360.0, Settings::default());
    let m = open_menu_screen(&mut a, 1.0);
    tap(&mut a, item(&m, Item::Stats), 2.0);
    assert_eq!(a.view(), Screen::Stats);
    assert_eq!(a.stats_view().filter, 0);
    let s = StatsScreen::new(a.table().layout(), a.stats().history().len());
    tap(&mut a, s.filter.next.center(), 3.0);
    assert_eq!(a.stats_view().filter, 1);
    tap(&mut a, s.filter.next.center(), 4.0);
    tap(&mut a, s.filter.next.center(), 5.0);
    assert_eq!(a.stats_view().filter, 2, "stops at the end");

    let from = s.viewport.center();
    a.handle(
        Gesture::DragStart {
            origin: from,
            at: from,
        },
        6.0,
    );
    a.handle(Gesture::Drop(from - vec2(0.0, 100.0)), 6.1);
    assert!((a.stats_view().scroll - 100.0).abs() < 0.01);

    let back = button(&a, Button::Settings);
    tap(&mut a, back, 7.0);
    assert_eq!(a.view(), Screen::Menu);
}

#[test]
fn restart_from_settings_redeals_and_returns_to_the_game() {
    let mut a = App::start(Box::new(MemoryStorage::default()), startup(31, false));
    let fresh = a.table().game().board().clone();
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    open_settings(&mut a, 2.0);
    tap_row(&mut a, Row::Restart, 3.0);
    assert_eq!(a.view(), Screen::Game);
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
    start_new_game(&mut a, 1.2);
    assert_eq!(a.stats().tally(None).played, 1);

    open_settings(&mut a, 2.0);
    let reset = row_control(&mut a, Row::ResetStats, 2.5);
    tap(&mut a, reset, 3.0);
    assert!(a.settings_view().confirm_reset);
    assert_eq!(a.stats().tally(None).played, 1, "first tap only asks");
    tap(&mut a, reset, 7.0);
    assert_eq!(
        a.stats().tally(None).played,
        1,
        "too late: this tap asks again"
    );
    tap(&mut a, reset, 8.0);
    assert_eq!(a.stats().tally(None).played, 0);
    assert!(!a.settings_view().confirm_reset);
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
    a.shortcut(Shortcut::Draw, 0.0);
    assert_eq!(a.table().game().board().waste.len(), 1, "Space draws");
    a.shortcut(Shortcut::Undo, 0.0);
    assert_eq!(a.table().game().board(), &fresh);
    a.shortcut(Shortcut::Redo, 0.0);
    assert_eq!(a.table().game().board().waste.len(), 1);
    let seed = a.table().game().seed();
    a.shortcut(Shortcut::NewGame, 1.0);
    assert!(a.dialog().is_some_and(Reveal::is_open), "N asks first");
    let confirm = DialogLayout::new(a.table().layout()).confirm.center();
    tap(&mut a, confirm, 2.0);
    assert_ne!(a.table().game().seed(), seed, "then deals a new game");
}

#[test]
fn space_recycles_an_empty_stock() {
    let mut a = app(1280.0, 720.0, Settings::default());
    for _ in 0..24 {
        a.shortcut(Shortcut::Draw, 0.0);
    }
    assert_eq!(a.table().game().board().stock.len(), 0);
    a.shortcut(Shortcut::Draw, 0.0);
    assert_eq!(a.table().game().board().stock.len(), 24);
}

#[test]
fn shortcuts_wait_while_the_menu_is_open() {
    let mut a = app(1280.0, 720.0, Settings::default());
    open_settings(&mut a, 1.0);
    a.shortcut(Shortcut::Draw, 0.0);
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
fn tapping_a_settings_control_lights_it() {
    let mut a = app(390.0, 844.0, Settings::default());
    open_settings(&mut a, 1.0);
    tap_row(&mut a, Row::FourColor, 2.0);
    assert!(a.settings().four_color);
    assert_eq!(
        a.feedback(Target::Settings(SettingsHit::Toggle(Row::FourColor)), 2.0),
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
    assert!(a.picker().is_some_and(|p| p.reveal.is_open()));
    tap(&mut a, layout.button.center(), 2.0);
    assert!(a.picker().is_some_and(|p| !p.reveal.is_open()), "closing");
    a.update(a.screen(), 3.0);
    assert!(a.picker().is_none(), "gone after the animation");
}

#[test]
fn tapping_a_swatch_applies_its_theme_and_keeps_the_picker_open() {
    let mut a = app(390.0, 844.0, Settings::default());
    let layout = open_picker(&mut a, 1.0);
    tap(&mut a, layout.swatches[1].center(), 2.0);
    assert_eq!(a.settings().theme, ThemeId::ALL[1]);
    assert!(a.picker().is_some_and(|p| p.reveal.is_open()));
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
    assert!(a.picker().is_some_and(|p| !p.reveal.is_open()));
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
    assert!(a.picker().is_some_and(|p| p.reveal.is_open()));
}

#[test]
fn new_game_asks_first_once_cards_have_moved() {
    let mut a = app(390.0, 844.0, Settings::default());
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    let before = a.table().game().board().clone();
    let p = button(&a, Button::New);
    tap(&mut a, p, 2.0);
    assert!(a.dialog().is_some_and(Reveal::is_open));
    assert_eq!(a.table().game().board(), &before, "nothing dealt yet");

    let d = DialogLayout::new(a.table().layout());
    tap(&mut a, d.cancel.center(), 3.0);
    assert!(a.dialog().is_some_and(|d| !d.is_open()));
    assert_eq!(a.table().game().board(), &before);
    a.update(a.screen(), 4.0);
    assert!(a.dialog().is_none(), "gone after the animation");

    tap(&mut a, p, 5.0);
    tap(&mut a, d.confirm.center(), 6.0);
    assert_eq!(a.table().game().board().moves, 0, "a fresh deal");
    assert_ne!(a.table().game().board(), &before);
}

#[test]
fn an_untouched_game_is_replaced_without_asking() {
    let mut a = app(390.0, 844.0, Settings::default());
    let seed = a.table().game().seed();
    let p = button(&a, Button::New);
    tap(&mut a, p, 1.0);
    assert!(a.dialog().is_none());
    assert_ne!(a.table().game().seed(), seed);
}

#[test]
fn tapping_outside_the_dialog_cancels_and_reaches_nothing_else() {
    let mut a = app(390.0, 844.0, Settings::default());
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 1.0);
    let p = button(&a, Button::New);
    tap(&mut a, p, 2.0);
    let before = a.table().game().board().clone();
    tap(&mut a, stock, 3.0);
    assert!(a.dialog().is_some_and(|d| !d.is_open()));
    assert_eq!(
        a.table().game().board(),
        &before,
        "the stock tap was swallowed"
    );
    let panel = DialogLayout::new(a.table().layout()).panel;
    tap(&mut a, p, 4.0);
    tap(&mut a, vec2(panel.center().x, panel.y + 4.0), 5.0);
    assert!(
        a.dialog().is_some_and(Reveal::is_open),
        "taps on the panel keep it"
    );
}

#[test]
fn the_menu_button_opens_the_menu_screen_and_back_returns() {
    let mut a = app(390.0, 844.0, Settings::default());
    assert_eq!(a.view(), Screen::Game);
    open_menu_screen(&mut a, 1.0);
    assert_eq!(a.view(), Screen::Menu);
    let back = button(&a, Button::Settings);
    tap(&mut a, back, 2.0);
    assert_eq!(a.view(), Screen::Game);
}

#[test]
fn taps_on_the_menu_screen_do_not_reach_the_cards() {
    let mut a = app(390.0, 844.0, Settings::default());
    let before = a.table().game().board().clone();
    open_menu_screen(&mut a, 1.0);
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 2.0);
    assert_eq!(a.table().game().board(), &before);
}

#[test]
fn selectors_change_draw_and_hand_and_are_saved() {
    let mut a = App::start(Box::new(MemoryStorage::default()), startup(5, false));
    let m = open_menu_screen(&mut a, 1.0);
    tap(&mut a, m.draw.next.center(), 2.0);
    assert_eq!(a.settings().draw, DrawMode::Three);
    tap(&mut a, m.hand.prev.center(), 3.0);
    assert!(a.settings().left_handed);
    let saved: Settings =
        serde_json::from_str(&a.storage().get(persist::SETTINGS_KEY).expect("saved")).unwrap();
    assert_eq!(saved.draw, DrawMode::Three);
    assert!(saved.left_handed);
    assert_eq!(m.hit(m.draw.next.center()), Some(MainHit::Draw(Step::Next)));
}

#[test]
fn about_has_its_own_screen() {
    let mut a = app(390.0, 844.0, Settings::default());
    let m = open_menu_screen(&mut a, 1.0);
    tap(&mut a, item(&m, Item::About), 2.0);
    assert_eq!(a.view(), Screen::About);
    let back = button(&a, Button::Settings);
    tap(&mut a, back, 3.0);
    assert_eq!(a.view(), Screen::Menu);
}

#[test]
fn each_screen_opens_at_the_top() {
    let mut a = app(390.0, 844.0, Settings::default());
    open_settings(&mut a, 1.0);
    let _ = row_control(&mut a, Row::ResetStats, 1.5);
    assert!(a.settings_view().scroll > 0.0);
    let back = button(&a, Button::Settings);
    tap(&mut a, back, 2.0);
    let m = MainMenu::new(a.table().layout());
    tap(&mut a, item(&m, Item::Stats), 3.0);
    assert_eq!(a.stats_view().scroll, 0.0);
}

#[test]
fn animation_speed_steps_with_the_chevrons() {
    let mut a = app(390.0, 844.0, Settings::default());
    open_settings(&mut a, 1.0);
    let screen = SettingsScreen::new(a.table().layout());
    let Some(Control::Selector(sel)) = screen.control(Row::Animation) else {
        panic!("animation is a selector");
    };
    let prev = row_control(&mut a, Row::Animation, 1.5) - sel.value.center() + sel.prev.center();
    tap(&mut a, prev, 2.0);
    assert_eq!(a.settings().animation, AnimSpeed::Fast);
    tap(&mut a, prev, 3.0);
    assert_eq!(a.settings().animation, AnimSpeed::Off);
    tap(&mut a, prev, 4.0);
    assert_eq!(a.settings().animation, AnimSpeed::Off, "stops at the end");
}

#[test]
fn a_new_draw_mode_redeals_an_untouched_game_on_the_way_back() {
    let mut a = app(390.0, 844.0, Settings::default());
    let seed = a.table().game().seed();
    let m = open_menu_screen(&mut a, 1.0);
    tap(&mut a, m.draw.next.center(), 2.0);
    let back = button(&a, Button::Settings);
    tap(&mut a, back, 3.0);
    assert_eq!(a.table().game().draw_mode(), DrawMode::Three);
    assert_ne!(a.table().game().seed(), seed);
    assert!(a.dialog().is_none());
}

#[test]
fn a_new_draw_mode_asks_before_replacing_a_game_in_progress() {
    let mut a = app(390.0, 844.0, Settings::default());
    let stock = a.table().layout().stock.center();
    tap(&mut a, stock, 0.5);
    let before = a.table().game().board().clone();
    let m = open_menu_screen(&mut a, 1.0);
    tap(&mut a, m.draw.next.center(), 2.0);
    let back = button(&a, Button::Settings);
    tap(&mut a, back, 3.0);
    assert_eq!(a.view(), Screen::Game);
    assert!(a.dialog().is_some_and(Reveal::is_open));
    let d = DialogLayout::new(a.table().layout());
    tap(&mut a, d.cancel.center(), 4.0);
    assert_eq!(a.table().game().board(), &before, "kept");
    assert_eq!(a.table().game().draw_mode(), DrawMode::One);

    open_menu_screen(&mut a, 5.0);
    tap(&mut a, back, 6.0);
    assert!(a.dialog().is_none(), "asks only right after the change");
}

#[test]
fn screens_fade_through_the_background() {
    let mut a = app(390.0, 844.0, Settings::default());
    open_menu_screen(&mut a, 1.0);
    assert_eq!(a.shown_screen(1.0), (Screen::Game, 1.0));
    let (screen, opacity) = a.shown_screen(1.1);
    assert_eq!(screen, Screen::Game);
    assert!(opacity < 1.0 && opacity > 0.0);
    assert_eq!(a.shown_screen(5.0), (Screen::Menu, 1.0));
}

#[test]
fn resizing_keeps_the_settings_scroll_in_range() {
    let mut a = app(390.0, 844.0, Settings::default());
    open_settings(&mut a, 1.0);
    let _ = row_control(&mut a, Row::ResetStats, 1.5);
    assert!(a.settings_view().scroll > 0.0);
    a.update(vec2(1280.0, 2000.0), 3.0);
    let max = SettingsScreen::new(a.table().layout()).max_scroll();
    assert!(a.settings_view().scroll <= max, "{max}");
}
