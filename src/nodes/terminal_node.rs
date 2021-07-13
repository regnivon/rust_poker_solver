use crate::{
    cfr::traversal::Traversal,
    ranges::combination::{Board, Combination},
};

use super::node::CfrNode;

pub struct TerminalNode {
    win_utility: f64,
    player_node: u8,
}

impl CfrNode for TerminalNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &Vec<f64>,
        board: &Board,
    ) -> Vec<f64> {
        self.dispatch_utility(traversal, op_reach_prob, board)
    }

    fn best_response(
        &self,
        traversal: &Traversal,
        op_reach_prob: &Vec<f64>,
        board: &Board,
    ) -> Vec<f64> {
        self.dispatch_utility(traversal, op_reach_prob, board)
    }
}

impl TerminalNode {
    pub fn new(pot_size: f64, player_node: u8) -> Self {
        Self {
            win_utility: pot_size / 2.0,
            player_node,
        }
    }

    fn dispatch_utility(
        &self,
        traversal: &Traversal,
        op_reach_prob: &Vec<f64>,
        board: &Board,
    ) -> Vec<f64> {
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
        win_utility: f64,
        op_reach_prob: &Vec<f64>,
        traverser_hands: &Vec<Combination>,
        opp_hands: &Vec<Combination>,
    ) -> Vec<f64> {
        let num_hands = traverser_hands.len();

        let mut utility = vec![0.0; num_hands];
        let mut card_removal = [0.0; 52];

        let mut probability_sum = 0.0;

        for i in 0..opp_hands.len() {
            if op_reach_prob[i] > 0.0 {
                probability_sum += op_reach_prob[i];

                card_removal[usize::from(opp_hands[i].hand[0])] += op_reach_prob[i];
                card_removal[usize::from(opp_hands[i].hand[1])] += op_reach_prob[i];
            }
        }

        for i in 0..traverser_hands.len() {
            utility[i] = (probability_sum
                - card_removal[usize::from(traverser_hands[i].hand[0])]
                - card_removal[usize::from(traverser_hands[i].hand[1])]
                + op_reach_prob[i])
                * win_utility;
        }

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

        println!("{:?}", traverser_hands);
        println!("{:?}", traverser_hands.len());

        let result = node.traverser_utility(5.0, &op_reach_prob, &traverser_hands, &opp_hands);

        println!("{:?}", result);
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

        println!("{:?}", traverser_hands);
        println!("{:?}", traverser_hands.len());
        println!("{:?}", op_reach_prob);

        let result = node.traverser_utility(5.0, &op_reach_prob, &traverser_hands, &opp_hands);

        println!("{:?}", result);
        for i in 0..6 {
            assert_eq!(result[i], 30.0);
        }
        for i in 6..12 {
            assert_eq!(result[i], 5.0);
        }
    }
}
