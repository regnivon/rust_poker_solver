use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::{
    cfr::traversal::Traversal,
    ranges::{combination::Board, utility::check_card_overlap},
};

use super::node::{CfrNode, Node};

pub struct ChanceNode {
    street: u8,
    next_nodes: Vec<Node>,
    pub next_cards: Vec<u8>,
    parallel: bool,
}

impl CfrNode for ChanceNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &Vec<f64>,
        board: &Board,
    ) -> Vec<f64> {
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

        let sub_results: Vec<Vec<f64>> = if self.parallel {
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

        for i in 0..sub_results.len() {
            for hand in 0..result.len() {
                result[hand] += sub_results[i][hand];
            }
        }

        if self.street == 1 {
            for hand in result.iter_mut() {
                *hand /= 45.0
            }
        } else {
            for hand in result.iter_mut() {
                *hand /= 44.0
            }
        }

        result
    }

    fn best_response(
        &self,
        traversal: &Traversal,
        op_reach_prob: &Vec<f64>,
        board: &Board,
    ) -> Vec<f64> {
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

        let sub_results: Vec<Vec<f64>> = if self.parallel {
            self.next_nodes
                .par_iter()
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
                .iter()
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

        for i in 0..sub_results.len() {
            for hand in 0..result.len() {
                result[hand] += sub_results[i][hand];
            }
        }

        if self.street == 1 {
            for hand in result.iter_mut() {
                *hand /= 45.0
            }
        } else {
            for hand in result.iter_mut() {
                *hand /= 44.0
            }
        }

        result
    }
}

impl ChanceNode {
    pub fn new(board: &Board, street: u8, parallel: bool) -> Self {
        let mut next_cards = vec![];
        for i in 0..52 {
            if !check_card_overlap(i, board) {
                next_cards.push(i);
            }
        }

        Self {
            street,
            next_nodes: vec![],
            next_cards,
            parallel,
        }
    }

    pub fn add_next_node(&mut self, child: Node) {
        self.next_nodes.push(child);
    }
}

#[cfg(test)]
mod tests {
    use crate::ranges::combination::Board;

    use super::ChanceNode;

    #[test]
    fn test_correct_turn_cards_amount() {
        let board: Board = [2, 6, 20, 52, 52];
        let chance = ChanceNode::new(&board, 1, true);
        assert_eq!(chance.next_cards.len(), 49);
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
