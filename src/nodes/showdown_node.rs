use crate::nodes::node::{CfrNode, NodeResult};
use crate::{
    cfr::traversal::Traversal,
    ranges::combination::{Board, Range},
};

#[derive(Debug)]
pub struct ShowdownNode {
    win_utility: f32,
}

const NUM_CARDS: usize = 52;

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
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let opp_hands = traversal.get_range_for_opponent(board);
        showdown(opp_hands, op_reach_prob, self.win_utility)
    }

    fn output_results(&self) -> Option<NodeResult> {
        None
    }
}

impl ShowdownNode {
    pub fn new(pot_size: f32) -> Self {
        Self {
            win_utility: pot_size / 2.0,
        }
    }
}

pub fn showdown(hands: &Range, op_reach_prob: &[f32], win_utility: f32) -> Vec<f32> {
    let mut sum = 0.0;
    let num_hands = hands.len();

    let mut utility = vec![0.0; num_hands];
    let mut card_removal = [0.0; 52];

    op_reach_prob.iter().zip(hands).for_each(|(prob, hand)| {
        if *prob > 0.0 {
            card_removal[usize::from(hand.hand[0])] -= prob;
            card_removal[usize::from(hand.hand[1])] -= prob;
            sum -= prob;
        }
    });

    let mut i = 0;

    while i < num_hands {
        let mut j = i + 1;
        while j < num_hands && hands[j].rank == hands[i].rank {
            j += 1;
        }

        let prob_slice = &op_reach_prob[i..j];
        let hand_slice = &hands[i..j];
        let util_slice = &mut utility[i..j];

        prob_slice
            .iter()
            .zip(hand_slice.iter())
            .for_each(|(prob, hand)| {
                card_removal[usize::from(hand.hand[0])] += prob;
                card_removal[usize::from(hand.hand[1])] += prob;
                sum += prob;
            });

        util_slice
            .iter_mut()
            .zip(hand_slice.iter())
            .for_each(|(util, hand)| {
                *util = win_utility
                    * (sum
                    - card_removal[usize::from(hand.hand[0])]
                    - card_removal[usize::from(hand.hand[1])]);
            });

        prob_slice
            .iter()
            .zip(hand_slice.iter())
            .for_each(|(prob, hand)| {
                card_removal[usize::from(hand.hand[0])] += prob;
                card_removal[usize::from(hand.hand[1])] += prob;
                sum += prob;
            });

        i = j;
    }

    utility
}

#[cfg(test)]
mod tests {
    use super::ShowdownNode;
    use crate::cfr::traversal::build_traversal_from_ranges;
    use crate::nodes::showdown_node::{showdown, unsafe_showdown};
    use crate::{
        cfr::traversal::Traversal,
        nodes::node::CfrNode,
        ranges::{range_manager::RangeManager, utility::construct_starting_range_from_string},
    };
    use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};
    use test::Bencher;
    use crate::ranges::combination::Combination;

    extern crate test;

    #[bench]
    fn bench_standard_utility(b: &mut Bencher) {
        let board = [2, 13, 24, 35, 47];
        let mut traverser_hands = construct_starting_range_from_string("22+,A7s,A6s,K8s,K5s,K4s,K3s,K2s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,J6s,J5s,J4s,J3s,J2s,T5s,T4s,T3s,T2s,96s,95s,85s,84s,74s,73s,63s,53s,52s,43s,42s,32s,A9o,A8o,A7o,A6o,A5o,A4o,A3o,KTo,K9o,K8o,K7o,K6o,QTo,Q9o,Q8o,JTo,J9o,J8o,T9o,T8o,98o,87o,76o,65o,A9s@75,A8s@75,A2s@75,K7s@75,K6s@75,Q8s@75,T7s@75,T6s@75,97s@75,86s@75,75s@75,64s@75,QJo@75,88@50,ATs@50,A3s@50,KTs@50,K9s@50,Q9s@50,J7s@50,94s@50,93s@50,54s@50,AJo@50,ATo@50,KQo@50,KJo@50,99@25,A4s@25,KJs@25,QJs@25,QTs@25,98s@25,87s@25,76s@25,65s@25".to_string(), &board);

        let mut board_hand = Hand::default();
        for board_card in board.iter() {
            board_hand += CARDS[usize::from(*board_card)];
        }

        traverser_hands.iter_mut().for_each(|h| {
            let eval_hand =
                board_hand + CARDS[usize::from(h.hand[0])] + CARDS[usize::from(h.hand[1])];
            h.rank = evaluate(&eval_hand);
        });

        traverser_hands.sort_by(|a, b| a.rank.cmp(&b.rank));

        let op_reach_prob = vec![1.0; traverser_hands.len()];
        b.iter(|| {
            test::black_box(showdown(&traverser_hands, &op_reach_prob, 1.0));
        });
    }
}
