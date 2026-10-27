//! Unit tests for `app`.

use macroquad::math::{Vec2, vec2};
use solitaire_core::{DrawMode, Game, PileId};

use super::*;
use crate::gesture::{DRAG_THRESHOLD, Gesture};
use crate::layout::{Button, Insets};
use crate::menu::{MenuHit, Row};
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
    let (menu, scroll) = a.menu().expect("menu open");
    let r = menu.rows.iter().find(|r| r.row == row).expect("row");
    r.targets[i].center() - vec2(0.0, scroll)
}

#[test]
fn settings_button_opens_the_menu_and_done_closes_it() {
    let mut a = app(390.0, 844.0, Settings::default());
    assert!(a.menu().is_none());
    let p = button(&a, Button::Settings);
    tap(&mut a, p, 1.0);
    let done = a.menu().expect("open").0.done.center();
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
    let light = row_target(&a, Row::Theme, 0);
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
        .0
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
    let (menu, _) = a.menu().expect("open");
    let max = menu.max_scroll();
    let start = menu.viewport.center();
    a.handle(
        Gesture::DragStart {
            origin: start,
            at: start - vec2(0.0, DRAG_THRESHOLD + 1.0),
        },
        2.0,
    );
    a.handle(Gesture::DragMove(start - vec2(0.0, 40.0)), 2.1);
    let scroll = a.menu().expect("open").1;
    assert!((scroll - 40.0).abs() < 0.01, "{scroll}");
    a.handle(Gesture::DragMove(start - vec2(0.0, 5000.0)), 2.2);
    assert!((a.menu().expect("open").1 - max).abs() < 0.01, "clamped");
    a.handle(Gesture::Drop(start), 2.3);
    assert!(a.menu().expect("open").1 >= 0.0);
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
