use crate::{
    cfr::traversal::Traversal,
    ranges::combination::{Board, Combination},
};

use crate::nodes::node::{CfrNode, NodeResult};
use crate::ranges::utility::hand_to_string;

#[derive(Debug)]
pub struct TerminalNode {
    win_utility: f32,
    player_node: u8,
}

impl CfrNode for TerminalNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        self.dispatch_utility(traversal, op_reach_prob, board)
    }

    fn best_response(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        self.dispatch_utility(traversal, op_reach_prob, board)
    }

    fn output_results(&self) -> Option<NodeResult> {
        None
    }
}

impl TerminalNode {
    pub fn new(pot_size: f32, player_node: u8) -> Self {
        Self {
            win_utility: pot_size / 2.0,
            player_node,
        }
    }

    fn dispatch_utility(
        &self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let traverser_hands = traversal.get_range_for_active_player(board);
        let opp_hands = traversal.get_range_for_opponent(board);

        let util = if traversal.traverser == self.player_node {
            self.win_utility
        } else {
            -self.win_utility
        };

        self.traverser_utility(util, op_reach_prob, traverser_hands, opp_hands)
    }

    fn traverser_utility(
        &self,
        win_utility: f32,
        op_reach_prob: &[f32],
        traverser_hands: &[Combination],
        opp_hands: &[Combination],
    ) -> Vec<f32> {
        let num_hands = traverser_hands.len();

        let mut utility = vec![0.0; num_hands];
        let mut card_removal = [0.0; 52];

        let mut probability_sum = 0.0;

        op_reach_prob
            .iter()
            .zip(opp_hands.iter())
            .for_each(|(prob, hand)| {
                if *prob > 0.0 {
                    probability_sum += prob;

                    card_removal[usize::from(hand.hand[0])] += prob;
                    card_removal[usize::from(hand.hand[1])] += prob;
                }
            });

        utility
            .iter_mut()
            .zip(traverser_hands.iter())
            .zip(op_reach_prob.iter())
            .for_each(|((util, combo), opp_prob)| {
                *util = (probability_sum
                    - card_removal[usize::from(combo.hand[0])]
                    - card_removal[usize::from(combo.hand[1])]
                    + opp_prob)
                    * win_utility;
            });

        utility
    }
}

#[cfg(test)]
mod tests {
    use crate::ranges::utility::construct_starting_range_from_string;

    use super::TerminalNode;

    #[test]
    fn test_traverser_utility() {
        let node = TerminalNode::new(10.0, 1);

        let op_reach_prob = vec![1.0; 30];

        let traverser_hands = construct_starting_range_from_string(
            "QQ,JJ,55,44,22".to_string(),
            &[51, 50, 49, 48, 47],
        );
        let opp_hands = construct_starting_range_from_string(
            "QQ,JJ,55,44,22".to_string(),
            &[51, 50, 49, 48, 47],
        );

        let result = node.traverser_utility(5.0, &op_reach_prob, &traverser_hands, &opp_hands);

        for i in 0..30 {
            assert_eq!(result[i], 125.0);
        }
    }

    #[test]
    fn test_traverser_utility2() {
        let node = TerminalNode::new(10.0, 1);

        let mut op_reach_prob = vec![1.0; 12];

        for i in 0..6 {
            op_reach_prob[i] = 0.0
        }

        let traverser_hands =
            construct_starting_range_from_string("QQ,JJ".to_string(), &[51, 50, 49, 48, 47]);
        let opp_hands =
            construct_starting_range_from_string("QQ,JJ".to_string(), &[51, 50, 49, 48, 47]);

        let result = node.traverser_utility(5.0, &op_reach_prob, &traverser_hands, &opp_hands);

        for i in 0..6 {
            assert_eq!(result[i], 30.0);
        }
        for i in 6..12 {
            assert_eq!(result[i], 5.0);
        }
    }
}
