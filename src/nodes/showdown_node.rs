use std::arch::aarch64::{vld1q_dup_f32, vld1q_f32, vmulq_f32, vst1q_f32};
use std::borrow::Borrow;
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
        // let rank_table = traversal.get_rank_table(board);
        // jump_showdown(opp_hands, rank_table, op_reach_prob, self.win_utility)
        showdown(opp_hands, op_reach_prob, self.win_utility)
    }

    fn best_response(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let opp_hands = traversal.get_range_for_opponent(board);
        // let rank_table = traversal.get_rank_table(board);
        // jump_showdown(opp_hands, rank_table, op_reach_prob, self.win_utility)
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
                    - card_removal[usize::from(hand.hand[1])])
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

pub fn jump_showdown(hands: &Range, rank_table: &[usize], op_reach_prob: &[f32], win_utility: f32) -> Vec<f32> {
    let mut sum = 0.0;
    let num_hands = hands.len();

    let mut sum_of_prob_per_rank = vec![0.0; rank_table.len()];
    let mut net_prob = vec![0.0; num_hands];
    let mut utility = vec![0.0; num_hands];
    let mut card_removal = [0.0; 52];

    let left_over = num_hands % 4;
    let simd_stop_index = num_hands - left_over;

    let mut rank_start = 0;

    for (rank, &rank_end) in rank_table.iter().enumerate() {
        let prob_slice = &op_reach_prob[rank_start..rank_end];
        let hand_slice = &hands[rank_start..rank_end];

        hand_slice.iter().zip(prob_slice.iter()).for_each(|(hand, prob)| {
            sum_of_prob_per_rank[rank] += prob;

            card_removal[usize::from(hand.hand[0])] -= prob;
            card_removal[usize::from(hand.hand[1])] -= prob;
        });

        sum -= sum_of_prob_per_rank[rank];
        rank_start = rank_end;
    }

    rank_start = 0;

    for (rank, &rank_end) in rank_table.iter().enumerate() {
        let prob_slice = &op_reach_prob[rank_start..rank_end];
        let hand_slice = &hands[rank_start..rank_end];
        let net_prob_slice = &mut net_prob[rank_start..rank_end];

        sum += sum_of_prob_per_rank[rank];

        hand_slice.iter().zip(prob_slice.iter()).for_each(|(hand, prob)| {
            card_removal[usize::from(hand.hand[0])] += prob;
            card_removal[usize::from(hand.hand[1])] += prob;
        });

        net_prob_slice.iter_mut().zip(hand_slice.iter()).for_each(|(prob, hand)| {
            *prob = sum
                - card_removal[usize::from(hand.hand[0])]
                - card_removal[usize::from(hand.hand[1])];
        });

        sum += sum_of_prob_per_rank[rank];

        hand_slice.iter().zip(prob_slice.iter()).for_each(|(hand, prob)| {
            card_removal[usize::from(hand.hand[0])] += prob;
            card_removal[usize::from(hand.hand[1])] += prob;
        });

        rank_start = rank_end;
    }

    unsafe {
        let win_util_vec =  vld1q_dup_f32(win_utility.borrow());
        for i in 0..simd_stop_index {

            vst1q_f32(
                utility.get_unchecked_mut(i),
                vmulq_f32(
                    vld1q_f32(net_prob.get_unchecked(i)),
                    win_util_vec
                )
            )
        }
    }


    // rank_start = 0;
    // for &rank_end in rank_table.iter() {
    //     let prob_slice = &op_reach_prob[rank_start..rank_end];
    //     let hand_slice = &hands[rank_start..rank_end];
    //     let util_slice = &mut utility[rank_start..rank_end];
    //     let mut rank_sum = 0.0;
    //
    //     prob_slice
    //         .iter()
    //         .zip(hand_slice.iter())
    //         .for_each(|(prob, hand)| {
    //             card_removal[usize::from(hand.hand[0])] += prob;
    //             card_removal[usize::from(hand.hand[1])] += prob;
    //             rank_sum += prob;
    //         });
    //
    //     sum += rank_sum;
    //
    //     util_slice
    //         .iter_mut()
    //         .zip(hand_slice.iter())
    //         .for_each(|(util, hand)| {
    //             *util = win_utility
    //                 * (sum
    //                 - card_removal[usize::from(hand.hand[0])]
    //                 - card_removal[usize::from(hand.hand[1])])
    //         });
    //
    //     prob_slice
    //         .iter()
    //         .zip(hand_slice.iter())
    //         .for_each(|(prob, hand)| {
    //             card_removal[usize::from(hand.hand[0])] += prob;
    //             card_removal[usize::from(hand.hand[1])] += prob;
    //         });
    //     sum += rank_sum;
    //
    //     rank_start = rank_end;
    // }

    utility
}

// pub fn jump_showdown(hands: &Range, tables: &ShowdownTables, op_reach_prob: &[f32], win_utility: f32) -> Vec<f32> {
//     const NUM_CARDS: usize = 52;
//     let jump_table = &tables.rank_table;
//     let num_hands = hands.len();
//     let num_ranks = jump_table.len();
//
//     let mut utility = vec![0.0; num_hands];
//     let mut card_removal_per_rank = vec![0.0; NUM_CARDS * num_ranks];
//     let mut card_removal_sum = [0.0; NUM_CARDS];
//     let mut final_card_removal = vec![0.0; NUM_CARDS * num_ranks];
//
//     let mut probability_sum = 0.0;
//     let mut sum_of_prob_per_rank = vec![0.0; num_ranks];
//     let mut rank_probability_sum = vec![0.0; num_ranks];
//
//     let mut rank_start = 0;
//     for rank in 0..num_ranks {
//         let rank_end = jump_table[rank];
//         let probability_slice = &op_reach_prob[rank_start..rank_end];
//         let hands_in_rank = &hands[rank_start..rank_end];
//
//         sum_of_prob_per_rank[rank] = probability_slice.iter().sum();
//
//         hands_in_rank
//             .iter()
//             .zip(probability_slice.iter())
//             .for_each(|(hand, probability)| {
//                 hand.hand.iter().for_each(|&card| {
//                     card_removal_per_rank[rank * NUM_CARDS + usize::from(card)] += probability;
//                     card_removal_sum[usize::from(card)] += probability;
//                 });
//             });
//
//         probability_sum += sum_of_prob_per_rank[rank];
//         rank_start = rank_end;
//
//         if rank == 0 {
//             rank_probability_sum[rank] = sum_of_prob_per_rank[rank];
//             continue;
//         }
//         rank_probability_sum[rank] = rank_probability_sum[rank - 1] + sum_of_prob_per_rank[rank] + sum_of_prob_per_rank[rank - 1];
//     }
//
//     for rank_sum in rank_probability_sum.iter_mut() {
//         *rank_sum -= probability_sum
//     }
//
//     let last_seen_cards = &tables.last_seen_card_per_rank;
//     for i in 0..num_ranks {
//         let last_seen_cards_rank = &last_seen_cards[i];
//         for &c in &tables.cards_per_rank[i] {
//             let card = usize::from(c);
//             // first time we have seen
//             if last_seen_cards_rank[card] == NUM_CARDS {
//                 final_card_removal[i * NUM_CARDS + card] =
//                     card_removal_per_rank[i * NUM_CARDS + card]
//                         - card_removal_sum[card];
//                 continue;
//             }
//             final_card_removal[i * NUM_CARDS + card] =
//                 final_card_removal[last_seen_cards_rank[card] * NUM_CARDS + card]
//                     + card_removal_per_rank[last_seen_cards_rank[card] * NUM_CARDS + card]
//                     + card_removal_per_rank[i * NUM_CARDS + card];
//         }
//
//
//         // for card in 0..NUM_CARDS {
//         //     if i == 0 {
//         //         final_card_removal[i * NUM_CARDS + card] =
//         //             card_removal_per_rank[i * NUM_CARDS + card]
//         //                 - card_removal_sum[card];
//         //         continue;
//         //     }
//         //     final_card_removal[i * NUM_CARDS + card] =
//         //         final_card_removal[(i - 1) * NUM_CARDS + card]
//         //             + card_removal_per_rank[(i - 1) * NUM_CARDS + card]
//         //             + card_removal_per_rank[i * NUM_CARDS + card];
//         // }
//     }
//
//     rank_start = 0;
//     for (rank, &rank_end) in jump_table.iter().enumerate() {
//         let hand_slice = &hands[rank_start..rank_end];
//         let util_slice = &mut utility[rank_start..rank_end];
//         let probability = rank_probability_sum[rank];
//
//         util_slice
//             .iter_mut()
//             .zip(hand_slice.iter())
//             .for_each(|(util, hand)| {
//                 *util = win_utility
//                     * (probability
//                     - final_card_removal[rank * NUM_CARDS + usize::from(hand.hand[0])]
//                     - final_card_removal[rank * NUM_CARDS + usize::from(hand.hand[1])]);
//             });
//
//         rank_start = rank_end;
//     }
//
//     utility
// }


use std::collections::BTreeSet;
use std::iter::FromIterator;
use crate::ranges::combination::Combination;

pub struct ShowdownTables {
    rank_table: Vec<usize>,
    cards_per_rank: Vec<Vec<u8>>,
    last_seen_card_per_rank: Vec<Vec<usize>>,
}

impl ShowdownTables {
    pub fn new(hands: &Range) -> Self {
        let rank_table = build_rank_table_for_hands(hands);
        let (cards_per_rank, last_seen_card_per_rank) = build_card_table(hands, &rank_table);

        Self {
            rank_table,
            cards_per_rank,
            last_seen_card_per_rank,
        }
    }
}

fn build_rank_table_for_hands(hands: &Range) -> Vec<usize> {
    let mut table = vec![0; 0];

    let mut i = 0;
    while i < hands.len() {
        let mut j = i + 1;
        while j < hands.len() && hands[j].rank == hands[i].rank {
            j += 1;
        }
        table.push(j);
        i = j;
    }
    table
}

fn build_card_table(hands: &Range, rank_table: &[usize]) -> (Vec<Vec<u8>>, Vec<Vec<usize>>) {
    let mut cards_in_rank = vec![vec![0; 0]; rank_table.len()];
    let mut rank_start = 0;
    // temp for seeing what rank a card was last in
    let mut last_rank_for_card_temp = [NUM_CARDS; NUM_CARDS];
    let mut last_rank_for_card = vec![vec![NUM_CARDS; NUM_CARDS]; rank_table.len()];

    for (rank, &rank_end) in rank_table.iter().enumerate() {
        let mut card_set = BTreeSet::new();
        let hands_in_rank = &hands[rank_start..rank_end];

        hands_in_rank.iter().for_each(|hand| {
            card_set.insert(hand.hand[0]);
            card_set.insert(hand.hand[1]);
        });

        cards_in_rank[rank] = Vec::from_iter(card_set);

        for &c in cards_in_rank[rank].iter() {
            let card = usize::from(c);

            if last_rank_for_card_temp[card] == NUM_CARDS {
                last_rank_for_card_temp[card] = rank;
                continue;
            }
            if last_rank_for_card_temp[card] != rank {
                last_rank_for_card[rank][card] = last_rank_for_card_temp[card];
            }
        }

        rank_start = rank_end;
    }

    (cards_in_rank, last_rank_for_card)
}


#[cfg(test)]
mod tests {
    use rust_poker::hand_evaluator::{CARDS, evaluate, Hand};
    use crate::{
        cfr::traversal::Traversal,
        nodes::node::CfrNode,
        ranges::{range_manager::RangeManager, utility::construct_starting_range_from_string},
    };
    use crate::nodes::showdown_node::{jump_showdown, showdown, ShowdownTables};
    use crate::cfr::traversal::build_traversal_from_ranges;
    use test::Bencher;
    use super::ShowdownNode;

    extern crate test;


    #[bench]
    fn bench_standard_utility(b: &mut Bencher) {
        let board = [2, 13, 24, 35, 47];
        let mut traverser_hands = construct_starting_range_from_string("77,66,55,44,33,22,A7s,A6s,K8s,K5s,K4s,K3s,K2s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,J6s,J5s,J4s,J3s,J2s,T5s,T4s,T3s,T2s,96s,95s,85s,84s,74s,73s,63s,53s,52s,43s,42s,32s,A9o,A8o,A7o,A6o,A5o,A4o,A3o,KTo,K9o,K8o,K7o,K6o,QTo,Q9o,Q8o,JTo,J9o,J8o,T9o,T8o,98o,87o,76o,65o,A9s@75,A8s@75,A2s@75,K7s@75,K6s@75,Q8s@75,T7s@75,T6s@75,97s@75,86s@75,75s@75,64s@75,QJo@75,88@50,ATs@50,A3s@50,KTs@50,K9s@50,Q9s@50,J7s@50,94s@50,93s@50,54s@50,AJo@50,ATo@50,KQo@50,KJo@50,99@25,A4s@25,KJs@25,QJs@25,QTs@25,98s@25,87s@25,76s@25,65s@25".to_string(), &board);

        let mut board_hand = Hand::default();
        for board_card in board.iter() {
            board_hand += CARDS[usize::from(*board_card)];
        }

        traverser_hands.iter_mut().for_each(|h| {
            let eval_hand = board_hand + CARDS[usize::from(h.hand[0])] + CARDS[usize::from(h.hand[1])];
            h.rank = evaluate(&eval_hand);
        });

        traverser_hands.sort_by(|a, b| a.rank.cmp(&b.rank));

        let op_reach_prob = vec![1.0; traverser_hands.len()];
        b.iter(|| {
            test::black_box(showdown(&traverser_hands, &op_reach_prob, 1.0));
        });
    }

    #[bench]
    fn bench_jump_utility(b: &mut Bencher) {
        let board = [2, 13, 24, 35, 47];
        let mut traverser_hands = construct_starting_range_from_string("77,66,55,44,33,22,A7s,A6s,K8s,K5s,K4s,K3s,K2s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,J6s,J5s,J4s,J3s,J2s,T5s,T4s,T3s,T2s,96s,95s,85s,84s,74s,73s,63s,53s,52s,43s,42s,32s,A9o,A8o,A7o,A6o,A5o,A4o,A3o,KTo,K9o,K8o,K7o,K6o,QTo,Q9o,Q8o,JTo,J9o,J8o,T9o,T8o,98o,87o,76o,65o,A9s@75,A8s@75,A2s@75,K7s@75,K6s@75,Q8s@75,T7s@75,T6s@75,97s@75,86s@75,75s@75,64s@75,QJo@75,88@50,ATs@50,A3s@50,KTs@50,K9s@50,Q9s@50,J7s@50,94s@50,93s@50,54s@50,AJo@50,ATo@50,KQo@50,KJo@50,99@25,A4s@25,KJs@25,QJs@25,QTs@25,98s@25,87s@25,76s@25,65s@25".to_string(), &board);

        let mut board_hand = Hand::default();
        for board_card in board.iter() {
            board_hand += CARDS[usize::from(*board_card)];
        }

        traverser_hands.iter_mut().for_each(|h| {
            let eval_hand = board_hand + CARDS[usize::from(h.hand[0])] + CARDS[usize::from(h.hand[1])];
            h.rank = evaluate(&eval_hand);
        });

        traverser_hands.sort_by(|a, b| a.rank.cmp(&b.rank));

        let op_reach_prob = vec![1.0; traverser_hands.len()];

        let table = ShowdownTables::new(&traverser_hands);

        b.iter(|| {
            test::black_box(jump_showdown(&traverser_hands, &table.rank_table, &op_reach_prob, 1.0));
        });
    }

    #[test]
    fn test_utility() {
        let board = [2, 13, 24, 35, 47];
        let mut traverser_hands = construct_starting_range_from_string("77,66,55,44,33,22,A7s,A6s,K8s,K5s,K4s,K3s,K2s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,J6s,J5s,J4s,J3s,J2s,T5s,T4s,T3s,T2s,96s,95s,85s,84s,74s,73s,63s,53s,52s,43s,42s,32s,A9o,A8o,A7o,A6o,A5o,A4o,A3o,KTo,K9o,K8o,K7o,K6o,QTo,Q9o,Q8o,JTo,J9o,J8o,T9o,T8o,98o,87o,76o,65o,A9s@75,A8s@75,A2s@75,K7s@75,K6s@75,Q8s@75,T7s@75,T6s@75,97s@75,86s@75,75s@75,64s@75,QJo@75,88@50,ATs@50,A3s@50,KTs@50,K9s@50,Q9s@50,J7s@50,94s@50,93s@50,54s@50,AJo@50,ATo@50,KQo@50,KJo@50,99@25,A4s@25,KJs@25,QJs@25,QTs@25,98s@25,87s@25,76s@25,65s@25".to_string(), &board);

        let mut board_hand = Hand::default();
        for board_card in board.iter() {
            board_hand += CARDS[usize::from(*board_card)];
        }

        traverser_hands.iter_mut().for_each(|h| {
            let eval_hand = board_hand + CARDS[usize::from(h.hand[0])] + CARDS[usize::from(h.hand[1])];
            h.rank = evaluate(&eval_hand);
        });

        traverser_hands.sort_by(|a, b| a.rank.cmp(&b.rank));

        let op_reach_prob = vec![1.0; traverser_hands.len()];

        let table = ShowdownTables::new(&traverser_hands);

        assert_eq!(showdown(&traverser_hands, &op_reach_prob, 1.0), jump_showdown(&traverser_hands, &table.rank_table, &op_reach_prob, 1.0));
    }
}
