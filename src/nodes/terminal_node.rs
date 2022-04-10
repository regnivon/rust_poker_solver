use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use crate::{
    cfr::traversal::Traversal,
    ranges::combination::{Board, Combination},
};
use crate::nodes::node::{CfrNode, NodeResult};

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
        let opp_hands = traversal.get_range_for_opponent(board);

        let util = if traversal.traverser == self.player_node {
            self.win_utility
        } else {
            -self.win_utility
        };

        terminal_utility(util, op_reach_prob, opp_hands)
    }
}

fn terminal_utility(
    win_utility: f32,
    op_reach_prob: &[f32],
    hands: &[Combination],
) -> Vec<f32> {
    let num_hands = hands.len();

    let mut utility = vec![0.0; num_hands];
    let mut card_removal = [0.0; 52];

    let mut probability_sum = 0.0;

    op_reach_prob
        .iter()
        .zip(hands.iter())
        .for_each(|(prob, hand)| {
            if *prob > 0.0 {
                probability_sum += prob;

                card_removal[usize::from(hand.hand[0])] += prob;
                card_removal[usize::from(hand.hand[1])] += prob;
            }
        });

    utility
        .iter_mut()
        .zip(hands.iter())
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

#[cfg(test)]
mod tests {
    use crate::ranges::utility::construct_starting_range_from_string;
    use test::Bencher;
    use crate::nodes::terminal_node::terminal_utility;

    extern crate test;

    use super::TerminalNode;

    #[bench]
    fn bench_standard_terminal(b: &mut Bencher) {
        let board = [2, 13, 24, 35, 47];
        let mut traverser_hands = construct_starting_range_from_string("77,66,55,44,33,22,A7s,A6s,K8s,K5s,K4s,K3s,K2s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,J6s,J5s,J4s,J3s,J2s,T5s,T4s,T3s,T2s,96s,95s,85s,84s,74s,73s,63s,53s,52s,43s,42s,32s,A9o,A8o,A7o,A6o,A5o,A4o,A3o,KTo,K9o,K8o,K7o,K6o,QTo,Q9o,Q8o,JTo,J9o,J8o,T9o,T8o,98o,87o,76o,65o,A9s@75,A8s@75,A2s@75,K7s@75,K6s@75,Q8s@75,T7s@75,T6s@75,97s@75,86s@75,75s@75,64s@75,QJo@75,88@50,ATs@50,A3s@50,KTs@50,K9s@50,Q9s@50,J7s@50,94s@50,93s@50,54s@50,AJo@50,ATo@50,KQo@50,KJo@50,99@25,A4s@25,KJs@25,QJs@25,QTs@25,98s@25,87s@25,76s@25,65s@25".to_string(), &board);

        let op_reach_prob = vec![1.0; traverser_hands.len()];
        b.iter(|| {
            test::black_box(terminal_utility(1.0, &op_reach_prob, &traverser_hands));
        });
    }
}
