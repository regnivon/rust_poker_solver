use crate::{
    cfr::traversal::Traversal,
    ranges::combination::{Board, Range},
};

use super::node::CfrNode;

pub fn showdown(hands: &Range, op_reach_prob: &[f32], win_utility: f32) -> Vec<f32> {
    let mut sum = 0.0;
    let num_hands = hands.len();

    let mut utility = vec![0.0; num_hands];
    let mut card_removal = [0.0; 52];

    for k in 0..num_hands {
        if op_reach_prob[k] > 0.0 {
            card_removal[usize::from(hands[k].hand[0])] -= op_reach_prob[k];
            card_removal[usize::from(hands[k].hand[1])] -= op_reach_prob[k];
            sum -= op_reach_prob[k];
        }
    }

    let mut i = 0;

    while i < num_hands {
        let mut j = i + 1;
        while j < num_hands && hands[j].rank == hands[i].rank {
            j += 1;
        }

        for k in i..j {
            card_removal[usize::from(hands[k].hand[0])] += op_reach_prob[k];
            card_removal[usize::from(hands[k].hand[1])] += op_reach_prob[k];
            sum += op_reach_prob[k];
        }

        for k in i..j {
            utility[k] = win_utility
                * (sum
                    - card_removal[usize::from(hands[k].hand[0])]
                    - card_removal[usize::from(hands[k].hand[1])])
        }

        for k in i..j {
            card_removal[usize::from(hands[k].hand[0])] += op_reach_prob[k];
            card_removal[usize::from(hands[k].hand[1])] += op_reach_prob[k];
            sum += op_reach_prob[k];
        }
        i = j;
    }

    utility
}

pub struct ShowdownNode {
    win_utility: f32,
}

impl CfrNode for ShowdownNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let opp_hands = traversal.get_range_for_opponent(board);
        showdown(opp_hands, op_reach_prob, self.win_utility)
    }

    fn best_response(
        &self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let opp_hands = traversal.get_range_for_opponent(board);
        showdown(opp_hands, op_reach_prob, self.win_utility)
    }
}

impl ShowdownNode {
    pub fn new(pot_size: f32) -> Self {
        Self {
            win_utility: pot_size / 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cfr::traversal::Traversal,
        nodes::node::CfrNode,
        ranges::{range_manager::RangeManager, utility::construct_starting_range_from_string},
    };

    use super::ShowdownNode;

    #[test]
    fn test_utility() {
        let mut node = ShowdownNode::new(10.0);
        let board = [51, 26, 20, 15, 11];

        let op_reach_prob = vec![1.0; 18];

        let traverser_hands = construct_starting_range_from_string("QQ,33,22".to_string(), &board);
        let opp_hands = construct_starting_range_from_string("QQ,33,22".to_string(), &board);

        let opp_rm = RangeManager::new(opp_hands, board);
        let ip_rm = RangeManager::new(traverser_hands, board);

        let trav = Traversal::new(opp_rm, ip_rm);

        let result = node.cfr_traversal(&trav, &op_reach_prob, &board);

        for i in 0..6 {
            assert_eq!(result[i], -60.0);
        }

        for i in 6..12 {
            assert_eq!(result[i], 0.0);
        }

        for i in 12..18 {
            assert_eq!(result[i], 60.0);
        }
    }
}
