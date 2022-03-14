use crate::nodes::node::{CfrNode, Node, NodeResult, NodeResultType};
use crate::{
    cfr::traversal::Traversal,
    ranges::{
        combination::Board,
        utility::{
            board_has_turn, build_initial_suit_groups, build_next_suit_groups, check_card_overlap,
            get_suit,
        },
    },
};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

pub struct ChanceNode {
    street: u8,
    next_nodes: Vec<Node>,
    pub next_cards: Vec<u8>,
    next_weights: Vec<i8>,
    parallel: bool,
}

impl CfrNode for ChanceNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let mut result = vec![0.0; traversal.get_num_hands_for_traverser(board)];
        let next_boards: Vec<Board> = self
            .next_cards
            .iter()
            .map(|c| {
                let mut b = *board;
                if self.street == 1 {
                    b[3] = *c;
                } else {
                    b[4] = *c;
                }
                b
            })
            .collect();

        let sub_results: Vec<Vec<f32>> = if self.parallel {
            self.next_nodes
                .par_iter_mut()
                .zip(next_boards.par_iter())
                .map(|(node, new_board)| {
                    let next_probs = traversal.get_next_reach_probs(new_board, op_reach_prob);
                    let utility = node.cfr_traversal(traversal, &next_probs, new_board);
                    let mut mapped_utility = vec![0.0; result.len()];
                    traversal.map_utility_backwards(new_board, &utility, &mut mapped_utility);
                    mapped_utility
                })
                .collect()
        } else {
            self.next_nodes
                .iter_mut()
                .zip(next_boards.iter())
                .map(|(node, new_board)| {
                    let next_probs = traversal.get_next_reach_probs(new_board, op_reach_prob);
                    let utility = node.cfr_traversal(traversal, &next_probs, new_board);
                    let mut mapped_utility = vec![0.0; result.len()];
                    traversal.map_utility_backwards(new_board, &utility, &mut mapped_utility);
                    mapped_utility
                })
                .collect()
        };

        for (runout, weight) in sub_results.iter().zip(self.next_weights.iter()) {
            result
                .iter_mut()
                .zip(runout.iter())
                .for_each(|(utility, runout_utility)| {
                    *utility += runout_utility * f32::from(*weight);
                });
        }

        let hands = traversal.get_range_for_active_player(board);

        if self.street == 1 {
            result.iter_mut().zip(hands.iter()).for_each(|(ev, hand)| {
                if hand.weight != 0 {
                    *ev /= 45.0 * f32::from(hand.weight);
                }
            });
        } else {
            result.iter_mut().zip(hands.iter()).for_each(|(ev, hand)| {
                if hand.weight != 0 {
                    *ev /= 44.0 * f32::from(hand.weight);
                }
            });
        }

        traversal.merge_canonical_utilities(board, &mut result);

        result
    }

    fn best_response(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let mut result = vec![0.0; traversal.get_num_hands_for_traverser(board)];

        let next_boards: Vec<Board> = self
            .next_cards
            .iter()
            .map(|c| {
                let mut b = *board;
                if self.street == 1 {
                    b[3] = *c;
                } else {
                    b[4] = *c;
                }
                b
            })
            .collect();

        let sub_results: Vec<Vec<f32>> = if self.parallel {
            self.next_nodes
                .par_iter_mut()
                .zip(next_boards.par_iter())
                .map(|(node, new_board)| {
                    let next_probs = traversal.get_next_reach_probs(new_board, op_reach_prob);
                    let utility = node.best_response(traversal, &next_probs, new_board);
                    let mut mapped_utility = vec![0.0; result.len()];
                    traversal.map_utility_backwards(new_board, &utility, &mut mapped_utility);
                    mapped_utility
                })
                .collect()
        } else {
            self.next_nodes
                .iter_mut()
                .zip(next_boards.iter())
                .map(|(node, new_board)| {
                    let next_probs = traversal.get_next_reach_probs(new_board, op_reach_prob);
                    let utility = node.best_response(traversal, &next_probs, new_board);
                    let mut mapped_utility = vec![0.0; result.len()];
                    traversal.map_utility_backwards(new_board, &utility, &mut mapped_utility);
                    mapped_utility
                })
                .collect()
        };

        for (runout, weight) in sub_results.iter().zip(self.next_weights.iter()) {
            result
                .iter_mut()
                .zip(runout.iter())
                .for_each(|(utility, runout_utility)| {
                    *utility += runout_utility * f32::from(*weight);
                });
        }

        let hands = traversal.get_range_for_active_player(board);

        if self.street == 1 {
            result.iter_mut().zip(hands.iter()).for_each(|(ev, hand)| {
                if hand.weight != 0 {
                    *ev /= 45.0 * f32::from(hand.weight);
                }
            });
        } else {
            result.iter_mut().zip(hands.iter()).for_each(|(ev, hand)| {
                if hand.weight != 0 {
                    *ev /= 44.0 * f32::from(hand.weight);
                }
            });
        }

        traversal.merge_canonical_utilities(board, &mut result);

        result
    }

    fn output_results(&self) -> Option<NodeResult> {
        let next = if self.street == 1 {
            self.next_nodes
                .iter()
                .filter_map(|node| node.output_results())
                .collect()
        } else {
            vec![]
        };

        Some(NodeResult {
            node_type: NodeResultType::Chance,
            node_strategy: None,
            node_ev: None,
            next_cards: Option::from(self.next_cards.clone()),
            next_nodes: next,
        })
    }
}

impl ChanceNode {
    pub fn new(board: &Board, street: u8, parallel: bool) -> Self {
        let mut next_cards = vec![];
        let mut next_weights = vec![];
        build_next(board, &mut next_cards, &mut next_weights);

        Self {
            street,
            next_nodes: vec![],
            next_cards,
            next_weights,
            parallel,
        }
    }

    pub fn add_next_node(&mut self, child: Node) {
        self.next_nodes.push(child);
    }
}

fn build_next(board: &Board, next_cards: &mut Vec<u8>, next_weights: &mut Vec<i8>) {
    let suit_groups = if board_has_turn(board) {
        let flop_groups = build_initial_suit_groups(&[board[0], board[1], board[2], 52, 52]);
        build_next_suit_groups(board, &flop_groups)
    } else {
        build_initial_suit_groups(board)
    };

    let mut suit_weights = [0i8; 4];

    for suit in 0u8..4 {
        if suit_groups[usize::from(suit)] == suit {
            suit_weights[usize::from(suit)] =
                suit_groups.iter().filter(|&n| *n == suit).count() as i8;
        } else {
            suit_weights[usize::from(suit)] = 0;
        }
    }

    for i in 0..52 {
        if !check_card_overlap(i, board) {
            let suit = get_suit(i);
            if suit_weights[usize::from(suit)] != 0 {
                next_cards.push(i);
                next_weights.push(suit_weights[usize::from(suit)]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ranges::{combination::Board, utility::card_to_number};

    use super::*;

    #[test]
    fn test_correct_turn_cards_amount() {
        let board: Board = [
            card_to_number("kc".to_string()),
            card_to_number("7h".to_string()),
            card_to_number("2h".to_string()),
            52,
            52,
        ];
        let chance = ChanceNode::new(&board, 1, true);
        assert_eq!(chance.next_cards.len(), 36);
        assert_eq!(chance.next_weights.len(), 36);
        for card in chance.next_cards.iter() {
            assert!(!board.contains(card));
        }
    }

    #[test]
    fn test_correct_turn_cards_amount_2() {
        let board: Board = [
            card_to_number("7c".to_string()),
            card_to_number("7h".to_string()),
            card_to_number("7d".to_string()),
            52,
            52,
        ];
        let chance = ChanceNode::new(&board, 1, true);
        assert_eq!(chance.next_cards.len(), 25);
        assert_eq!(chance.next_weights.len(), 25);
        println!("{:?}", chance.next_weights);
        for card in chance.next_cards.iter() {
            assert!(!board.contains(card));
        }
    }

    #[test]
    fn test_correct_turn_cards_amount_3() {
        let board: Board = [
            card_to_number("kc".to_string()),
            card_to_number("7c".to_string()),
            card_to_number("2c".to_string()),
            52,
            52,
        ];
        let chance = ChanceNode::new(&board, 1, true);
        assert_eq!(chance.next_cards.len(), 23);
        assert_eq!(chance.next_weights.len(), 23);
        println!("{:?}", chance.next_weights);
        for card in chance.next_cards.iter() {
            assert!(!board.contains(card));
        }
    }

    #[test]
    fn test_correct_river_cards_amount() {
        let board: Board = [2, 6, 20, 15, 52];
        let chance = ChanceNode::new(&board, 1, true);
        assert_eq!(chance.next_cards.len(), 48);
        for card in chance.next_cards.iter() {
            assert!(!board.contains(card));
        }
    }
}
