mod cfr;
mod nodes;
mod ranges;

extern crate rayon;

use crate::{
    cfr::{game::Game, game_params::GameParams, traversal::Traversal},
    ranges::{
        combination::Board,
        range_manager::RangeManager,
        utility::{card_to_number, construct_starting_range_from_string},
    },
};

fn main() {
    let board: Board = [
        card_to_number("kc".to_string()),
        card_to_number("7h".to_string()),
        card_to_number("2h".to_string()),
        card_to_number("3d".to_string()),
        52, //card_to_number("2c".to_string()),
    ];
    let starting_combinations = construct_starting_range_from_string("random".to_string(), &board);
    let starting_combinations2 = construct_starting_range_from_string("random".to_string(), &board);
    let rm = RangeManager::new(starting_combinations, board);
    let rm2 = RangeManager::new(starting_combinations2, board);
    let params = GameParams::new(
        2,
        60.0,
        1000.0,
        1.0,
        0.75,
        vec![vec![0.33, 0.75, 1.5]],
        vec![vec![0.33, 0.75, 1.5]],
        vec![vec![0.33, 0.75, 1.5]],
        vec![vec![0.33, 0.75, 1.5]],
        vec![vec![0.33, 0.75, 1.5]],
        vec![vec![0.33, 0.75, 1.5]],
    );

    let trav = Traversal::new(rm, rm2);

    let mut game = Game::new(trav, params, board);

    game.train(100);
}
