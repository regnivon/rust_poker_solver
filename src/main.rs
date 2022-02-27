mod cfr;
mod nodes;
mod ranges;

extern crate rayon;

use crate::{
    cfr::{game::Game, game_params::GameParams, traversal::Traversal},
    ranges::{
        combination::Board,
        range_manager::{IsomorphicRangeManager, RangeManagers, DefaultRangeManager},
        utility::{
            build_initial_suit_groups, card_to_number, construct_starting_range_from_string,
        },
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_trainer();
    Ok(())
}

fn run_trainer() {
    let board: Board = [
        card_to_number("kc".to_string()),
        card_to_number("7h".to_string()),
        card_to_number("2d".to_string()),
        52, //card_to_number("3d".to_string()),
        52, //card_to_number("2c".to_string()),
    ];
    let starting_combinations = construct_starting_range_from_string("random".to_string(), &board);
    let starting_combinations2 = construct_starting_range_from_string("random".to_string(), &board);
    // let starting_combinations = construct_starting_range_from_string("TT,99,88,77,66,55,44,33,22,ATs,A9s,A8s,A7s,A6s,A5s,A4s,A3s,A2s,KTs,K9s,K8s,K7s,K6s,K5s,K4s,K3s,K2s,QTs,Q9s,Q8s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,JTs,J9s,J8s,J7s,J6s,J5s,J4s,T9s,T8s,T7s,T6s,T5s,T4s,98s,97s,96s,95s,94s,87s,86s,85s,76s,75s,74s,65s,65s,64s,54s,43s,AJo,ATo,A9o,A8o,A7o,A6o,A5o,A4o,KJo,KTo,QJo,QTo,JTo".to_string(), &board);
    // let starting_combinations2 = construct_starting_range_from_string("TT,99,88,77,66,55,44,33,22,ATs,A9s,A8s,A7s,A6s,A5s,A4s,A3s,A2s,KTs,K9s,K8s,K7s,K6s,K5s,K4s,K3s,K2s,QTs,Q9s,Q8s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,JTs,J9s,J8s,J7s,J6s,J5s,J4s,T9s,T8s,T7s,T6s,T5s,T4s,98s,97s,96s,95s,94s,87s,86s,85s,76s,75s,74s,65s,65s,64s,54s,43s,AJo,ATo,A9o,A8o,A7o,A6o,A5o,A4o,KJo,KTo,QJo,QTo,JTo".to_string(), &board);

    let sg = build_initial_suit_groups(&board);
    let mut iso = false;
    for suit in 0u8..4 {
        if sg[usize::from(suit)] != suit {
            iso = true;
        }
    }
    let rm = if iso {
        RangeManagers::from(IsomorphicRangeManager::new(starting_combinations, board))
    } else {
        RangeManagers::from(DefaultRangeManager::new(starting_combinations, board))
    };

    let rm2 = if iso {
        RangeManagers::from(IsomorphicRangeManager::new(starting_combinations2, board))
    } else {
        RangeManagers::from(DefaultRangeManager::new(starting_combinations2, board))
    };

    let params = GameParams::new(
        1,
        60.0,
        1000.0,
        1.0,
        0.75,
        vec![vec![]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
    );

    let trav = Traversal::new(rm, rm2);

    let mut game = Game::new(trav, params, board);

    /*
    Iteration 0 OOP BR 139.70445 IP BR 146.03627 exploitability = 238.11726 percent of the pot
    Iteration 25 OOP BR 2.500492 IP BR 7.2116675 exploitability = 8.093467 percent of the pot
    Iteration 50 OOP BR -1.7358472 IP BR 3.756436 exploitability = 1.6838242 percent of the pot

    real    0m54.853s
    user    11m19.130s
    sys     0m5.169s
     */
    game.train(75);
}
