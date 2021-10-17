mod cfr;
mod nodes;
mod ranges;

extern crate rayon;

//use jemallocator::Jemalloc;
//#[global_allocator]
//static GLOBAL: Jemalloc = Jemalloc;

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
        card_to_number("2c".to_string()),
        52, //card_to_number("3d".to_string()),
        52, //card_to_number("2c".to_string()),
    ];
    let starting_combinations = construct_starting_range_from_string("TT,99,88,77,66,55,44,33,22,ATs,A9s,A8s,A7s,A6s,A5s,A4s,A3s,A2s,KTs,K9s,K8s,K7s,K6s,K5s,K4s,K3s,K2s,QTs,Q9s,Q8s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,JTs,J9s,J8s,J7s,J6s,J5s,J4s,T9s,T8s,T7s,T6s,T5s,T4s,98s,97s,96s,95s,94s,87s,86s,85s,76s,75s,74s,65s,65s,64s,54s,43s,AJo,ATo,A9o,A8o,A7o,A6o,A5o,A4o,KJo,KTo,QJo,QTo,JTo".to_string(), &board);
    let starting_combinations2 = construct_starting_range_from_string("TT,99,88,77,66,55,44,33,22,ATs,A9s,A8s,A7s,A6s,A5s,A4s,A3s,A2s,KTs,K9s,K8s,K7s,K6s,K5s,K4s,K3s,K2s,QTs,Q9s,Q8s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,JTs,J9s,J8s,J7s,J6s,J5s,J4s,T9s,T8s,T7s,T6s,T5s,T4s,98s,97s,96s,95s,94s,87s,86s,85s,76s,75s,74s,65s,65s,64s,54s,43s,AJo,ATo,A9o,A8o,A7o,A6o,A5o,A4o,KJo,KTo,QJo,QTo,JTo".to_string(), &board);
    let rm = RangeManager::new(starting_combinations, board);
    let rm2 = RangeManager::new(starting_combinations2, board);
    let params = GameParams::new(
        1,
        60.0,
        1000.0,
        1.0,
        0.75,
        vec![vec![]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.33, 0.75]],
        vec![vec![0.75, 1.5]],
        vec![vec![0.75, 1.5, 1000.0]],
    );

    let trav = Traversal::new(rm, rm2);

    let mut game = Game::new(trav, params, board);

    //2.82
    game.train(300);
}
