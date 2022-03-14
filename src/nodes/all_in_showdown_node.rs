use super::node::CfrNode;
use crate::nodes::node::NodeResult;
use crate::{
    cfr::traversal::Traversal,
    nodes::showdown_node::showdown,
    ranges::{combination::Board, utility::check_card_overlap},
};

#[derive(Debug)]
pub struct AllInShowdownNode {
    win_utility: f32,
    street: u8,
}

impl CfrNode for AllInShowdownNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        self.all_in_showdown_node_utility(traversal, op_reach_prob, board)
    }

    fn best_response(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        self.all_in_showdown_node_utility(traversal, op_reach_prob, board)
    }

    fn output_results(&self) -> Option<NodeResult> {
        None
    }
}

impl AllInShowdownNode {
    pub fn new(pot_size: f32, street: u8) -> Self {
        Self {
            win_utility: pot_size / 2.0,
            street,
        }
    }

    fn all_in_showdown_node_utility(
        &self,
        traversal: &Traversal,
        op_reach_probs: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let mut utility = vec![0.0; traversal.get_num_hands_for_traverser(board)];
        let hands = traversal.get_range_for_active_player(board);

        if self.street == 1 {
            for turn in 0..52 {
                if !check_card_overlap(turn, board) {
                    let mut next_board = *board;
                    next_board[3] = turn;

                    let turn_probs = traversal.get_next_reach_probs(&next_board, op_reach_probs);
                    let mut turn_utility = vec![0.0; turn_probs.len()];
                    for river in (turn + 1)..52 {
                        if !check_card_overlap(river, &next_board) {
                            next_board[4] = river;
                            let river_probs =
                                traversal.get_next_reach_probs(&next_board, &turn_probs);
                            let river_hands = traversal.get_range_for_active_player(&next_board);
                            let river_utility =
                                showdown(river_hands, &river_probs, self.win_utility);
                            traversal.map_utility_backwards(
                                &next_board,
                                &river_utility,
                                &mut turn_utility,
                            );
                        }
                    }
                    next_board[4] = 52;
                    let turn_hands = traversal.get_range_for_active_player(&next_board);

                    turn_utility
                        .iter_mut()
                        .zip(turn_hands.iter())
                        .for_each(|(util, hand)| {
                            if hand.weight != 0 {
                                *util /= f32::from(hand.weight);
                            }
                        });

                    traversal.merge_canonical_utilities(&next_board, &mut turn_utility);

                    next_board[4] = 52;
                    traversal.map_utility_backwards(&next_board, &turn_utility, &mut utility)
                }
            }
            utility
                .iter_mut()
                .zip(hands.iter())
                .for_each(|(val, hand)| {
                    *val /= 990.0 * f32::from(hand.weight);
                });
        } else {
            for river in 0..52 {
                if !check_card_overlap(river, board) {
                    let mut next_board = *board;
                    next_board[4] = river;
                    let river_probs = traversal.get_next_reach_probs(&next_board, op_reach_probs);
                    let hands = traversal.get_range_for_opponent(&next_board);
                    let river_utility = showdown(hands, &river_probs, self.win_utility);

                    traversal.map_utility_backwards(&next_board, &river_utility, &mut utility);
                }
            }
            utility
                .iter_mut()
                .zip(hands.iter())
                .for_each(|(val, hand)| {
                    *val /= 44.0 * f32::from(hand.weight);
                });
        }

        traversal.merge_canonical_utilities(&board, &mut utility);

        utility
    }
}
