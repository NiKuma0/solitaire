//! Property tests: random legal play never breaks the game's invariants (SPEC §10).

use proptest::prelude::*;
use proptest::sample::Index;
use solitaire_core::{Column, DrawMode, Game, Move, PileId};

fn draw_mode() -> impl Strategy<Value = DrawMode> {
    prop_oneof![Just(DrawMode::One), Just(DrawMode::Three)]
}

/// Plays `choices` as indices into the legal moves, checking every step.
fn play(seed: u64, draw: DrawMode, choices: &[Index]) -> Result<Game, TestCaseError> {
    let mut game = Game::new(seed, draw);
    for choice in choices {
        let moves = game.legal_moves();
        if moves.is_empty() {
            break;
        }
        let mv = moves[choice.index(moves.len())];
        let before = game.board().clone();

        let outcome = game
            .apply(mv)
            .map_err(|e| TestCaseError::fail(format!("{mv:?}: {e}")))?;
        let after = game.board().clone();

        // 52 unique cards, ordered foundations, well-formed columns.
        prop_assert_eq!(after.validate(), Ok(()));
        prop_assert_eq!(
            i64::from(outcome.points),
            i64::from(after.score) - i64::from(before.score)
        );
        prop_assert_eq!(after.moves, before.moves + 1);

        // undo ∘ apply = id, and redo brings the move back.
        game.undo()
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(game.board(), &before);
        game.redo()
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(game.board(), &after);
    }
    Ok(game)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn random_play_keeps_invariants(
        seed in any::<u64>(),
        draw in draw_mode(),
        choices in prop::collection::vec(any::<Index>(), 0..300),
    ) {
        play(seed, draw, &choices)?;
    }

    #[test]
    fn undoing_everything_returns_to_the_deal(
        seed in any::<u64>(),
        draw in draw_mode(),
        choices in prop::collection::vec(any::<Index>(), 0..100),
    ) {
        let mut game = play(seed, draw, &choices)?;
        while game.undo().is_ok() {}
        let fresh = Game::new(seed, draw);
        prop_assert_eq!(game.board(), fresh.board());
    }

    #[test]
    fn saves_round_trip(
        seed in any::<u64>(),
        draw in draw_mode(),
        choices in prop::collection::vec(any::<Index>(), 0..100),
    ) {
        let game = play(seed, draw, &choices)?;
        let json = serde_json::to_string(&game).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let loaded: Game = serde_json::from_str(&json).map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(loaded, game);
    }

    #[test]
    fn tap_suggestions_are_legal(
        seed in any::<u64>(),
        draw in draw_mode(),
        choices in prop::collection::vec(any::<Index>(), 0..100),
    ) {
        let game = play(seed, draw, &choices)?;
        let legal = game.legal_moves();
        let b = game.board();
        let tappable = b.waste.last().into_iter().chain(b.tableau.iter().flat_map(Column::face_up));
        for &card in tappable {
            if let Some(mv) = game.best_move_for(card) {
                prop_assert!(legal.contains(&mv), "{card}: {mv:?}");
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Every state reachable from a dead end is a dead end too.
    #[test]
    fn stuck_is_closed_under_moves(
        seed in any::<u64>(),
        draw in draw_mode(),
        choices in prop::collection::vec(any::<Index>(), 0..200),
    ) {
        let game = play(seed, draw, &choices)?;
        let moves = game.legal_moves();
        if game.is_stuck() {
            for mv in moves {
                let mut next = game.clone();
                next.apply(mv).map_err(|e| TestCaseError::fail(e.to_string()))?;
                prop_assert!(next.is_stuck(), "{mv:?} escaped a dead end");
            }
        }
    }
}

/// A greedy player reaches auto-complete in some deals; every plan offered must win.
#[test]
fn auto_complete_plans_always_win() {
    let mut offered = 0;
    for seed in 0..100 {
        for draw in [DrawMode::One, DrawMode::Three] {
            let mut game = Game::new(seed, draw);
            for _ in 0..300 {
                if let Some(plan) = game.auto_complete_plan() {
                    offered += 1;
                    for mv in plan {
                        game.apply(mv).expect("planned move is legal");
                    }
                    assert!(game.is_won(), "seed {seed} {draw:?}");
                    break;
                }
                let Some(mv) = greedy(&game) else { break };
                game.apply(mv).expect("legal move");
            }
        }
    }
    // These fixed seeds reach auto-complete 10 times; a drop means the test lost its teeth.
    assert!(offered >= 5, "only {offered} auto-complete plans checked");
}

fn greedy(game: &Game) -> Option<Move> {
    let moves = game.legal_moves();
    let pick = |pred: &dyn Fn(&Move) -> bool| moves.iter().copied().find(|m| pred(m));
    pick(&|m| {
        matches!(
            m,
            Move::Transfer {
                to: PileId::Foundation(_),
                ..
            }
        )
    })
    .or_else(|| pick(&|m| reveals(game, *m)))
    .or_else(|| {
        pick(&|m| {
            matches!(
                m,
                Move::Transfer {
                    from: PileId::Waste,
                    ..
                }
            )
        })
    })
    .or_else(|| pick(&|m| matches!(m, Move::Draw | Move::Recycle)))
}

fn reveals(game: &Game, mv: Move) -> bool {
    let mut next = game.clone();
    next.apply(mv).is_ok_and(|o| o.revealed)
}
