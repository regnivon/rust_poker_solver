use super::node::{CfrNode, Node};
use crate::nodes::node::{NodeResult, NodeResultType};
use crate::{cfr::traversal::Traversal, ranges::combination::Board};
#[cfg(all(target_arch = "aarch64"))]
use std::arch::aarch64::*;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use std::arch::x86_64::*;
use std::borrow::Borrow;
use std::cmp::{max, min};

pub struct ActionNode {
    pub player_node: u8,
    num_hands: usize,
    num_actions: usize,
    pub pot_size: f32,
    pub ip_stack: f32,
    pub oop_stack: f32,
    next_nodes: Vec<Node>,
    regret_accumulator: Vec<f32>,
    strategy_accumulator: Vec<f32>,
    node_ev: Option<Vec<f32>>,
}

impl CfrNode for ActionNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        let mut node_utility = vec![0.0; traversal.get_num_hands_for_traverser(board)];
        if traversal.traverser == self.player_node {
            self.traverser_cfr(traversal, op_reach_prob, &mut node_utility, board)
        } else {
            self.opponent_cfr(traversal, op_reach_prob, &mut node_utility, board)
        }
        node_utility
    }

    fn best_response(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        if self.player_node == traversal.traverser {
            let mut best_ev = vec![0.0; self.num_hands];
            let mut node_evs = vec![];
            for action in 0..self.num_actions {
                let next_ev =
                    self.next_nodes[action].best_response(traversal, op_reach_prob, board);

                if traversal.persist_evs {
                    node_evs.extend_from_slice(&next_ev)
                }
                best_ev
                    .iter_mut()
                    .zip(next_ev.iter())
                    .for_each(|(best, next)| {
                        if action == 0 || next > best {
                            *best = *next;
                        }
                    });
            }
            if traversal.persist_evs {
                let opp_hands = traversal.get_range_for_opponent(board);
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

                for (i, ev) in node_evs.iter_mut().enumerate() {
                    *ev /= probability_sum
                        - card_removal[usize::from(opp_hands[i % opp_hands.len()].hand[0])]
                        - card_removal[usize::from(opp_hands[i % opp_hands.len()].hand[1])]
                        + op_reach_prob[i % opp_hands.len()];
                    *ev += self.pot_size;
                }

                self.node_ev = Some(node_evs)
            }
            best_ev
        } else {
            let mut node_ev = vec![0.0; traversal.get_num_hands_for_traverser(board)];
            let average_strategy = self.get_average_strategy();
            for action in 0..self.num_actions {
                let action_offset = action * self.num_hands;
                let mut next_reach = vec![0.0; op_reach_prob.len()];
                let strategy_slice = &average_strategy[action_offset..];

                next_reach
                    .iter_mut()
                    .zip(strategy_slice.iter())
                    .zip(op_reach_prob.iter())
                    .for_each(|((next, strategy), prob)| {
                        *next = strategy * prob;
                    });

                let action_ev =
                    self.next_nodes[action].best_response(traversal, &next_reach, board);

                node_ev
                    .iter_mut()
                    .zip(action_ev.iter())
                    .for_each(|(node, action)| {
                        *node += *action;
                    });
            }
            node_ev
        }
    }

    fn output_results(&self) -> Option<NodeResult> {
        Some(NodeResult {
            node_type: NodeResultType::Action,
            node_strategy: Some(self.get_average_strategy()),
            node_ev: self.node_ev.clone(),
            next_cards: None,
            next_nodes: self
                .next_nodes
                .iter()
                .filter_map(|node| node.output_results())
                .collect(),
        })
    }
}

impl ActionNode {
    pub fn new(
        player_node: u8,
        num_hands: usize,
        pot_size: f32,
        ip_stack: f32,
        oop_stack: f32,
    ) -> Self {
        Self {
            player_node,
            num_hands,
            num_actions: 0,
            pot_size,
            ip_stack,
            oop_stack,
            next_nodes: vec![],
            regret_accumulator: vec![],
            strategy_accumulator: vec![],
            node_ev: None,
        }
    }

    pub fn init_vectors(&mut self) {
        self.regret_accumulator = vec![0.0; self.num_hands * self.num_actions];
        self.strategy_accumulator = vec![0.0; self.num_hands * self.num_actions];
    }

    pub fn add_child(&mut self, child: Node) {
        self.num_actions += 1;
        self.next_nodes.push(child);
    }

    fn get_strategy(&self) -> Vec<f32> {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { self.get_strategy_avx2_optimized() };
            }
        }

        #[cfg(all(target_arch = "aarch64"))]
        {
            return unsafe { self.get_strategy_neon_optimized() };
        }

        self.get_strategy_fallback()
    }

    fn get_strategy_fallback(&self) -> Vec<f32> {
        let nums = self.num_actions * self.num_hands;
        if self.num_actions == 1 {
            return vec![1.0; nums];
        }
        let mut strategy = vec![0.0; nums];

        match self.num_actions {
            2 => {
                let regret_sum0 = &self.regret_accumulator[0..self.num_hands];
                let regret_sum1 = &self.regret_accumulator[self.num_hands..];

                let (strategy0, strategy1) = strategy.split_at_mut(self.num_hands);

                strategy0
                    .iter_mut()
                    .zip(strategy1.iter_mut())
                    .zip(regret_sum0.iter())
                    .zip(regret_sum1.iter())
                    .for_each(|(((s0, s1), r0), r1)| {
                        if *r0 > 0.0 {
                            if *r1 > 0.0 {
                                let positive_regret_sum = r0 + r1;
                                *s0 = r0 / positive_regret_sum;
                                *s1 = r1 / positive_regret_sum;
                            } else {
                                *s0 = 1.0;
                            }
                        } else if *r1 > 0.0 {
                            *s1 = 1.0;
                        } else {
                            *s0 = 0.5;
                            *s1 = 0.5;
                        }
                    });
            }
            3 => {
                let regret_sum0 = &self.regret_accumulator[0..self.num_hands];
                let regret_sum1 = &self.regret_accumulator[self.num_hands..self.num_hands * 2];
                let regret_sum2 = &self.regret_accumulator[self.num_hands * 2..];

                let (strategy0, strategy12) = strategy.split_at_mut(self.num_hands);
                let (strategy1, strategy2) = strategy12.split_at_mut(self.num_hands);

                strategy0
                    .iter_mut()
                    .zip(strategy1.iter_mut())
                    .zip(strategy2.iter_mut())
                    .zip(regret_sum0.iter())
                    .zip(regret_sum1.iter())
                    .zip(regret_sum2.iter())
                    .for_each(|(((((s0, s1), s2), r0), r1), r2)| {
                        if *r0 > 0.0 {
                            if *r1 > 0.0 {
                                if *r2 > 0.0 {
                                    let positive_regret_sum = r0 + r1 + r2;
                                    *s0 = r0 / positive_regret_sum;
                                    *s1 = r1 / positive_regret_sum;
                                    *s2 = r2 / positive_regret_sum;
                                } else {
                                    let positive_regret_sum = r0 + r1;
                                    *s0 = r0 / positive_regret_sum;
                                    *s1 = r1 / positive_regret_sum;
                                }
                            } else if *r2 > 0.0 {
                                let positive_regret_sum = r0 + r2;
                                *s0 = r0 / positive_regret_sum;
                                *s2 = r2 / positive_regret_sum;
                            } else {
                                *s0 = 1.0;
                            }
                        } else if *r1 > 0.0 {
                            if *r2 > 0.0 {
                                let positive_regret_sum = r1 + r2;
                                *s1 = r1 / positive_regret_sum;
                                *s2 = r2 / positive_regret_sum;
                            } else {
                                *s1 = 1.0;
                            }
                        } else if *r2 > 0.0 {
                            *s2 = 1.0;
                        } else {
                            *s0 = 1.0 / 3.0;
                            *s1 = 1.0 / 3.0;
                            *s2 = 1.0 / 3.0;
                        }
                    });
            }
            _ => {
                let probability = 1.0 / (self.num_actions as f32);
                for hand in 0..self.num_hands {
                    let mut normalizing_value = 0.0;
                    for action in 0..self.num_actions {
                        if self.regret_accumulator[hand + action * self.num_hands] > 0.0 {
                            normalizing_value +=
                                self.regret_accumulator[hand + action * self.num_hands];
                        }
                    }

                    if normalizing_value > 0.0 {
                        for action in 0..self.num_actions {
                            if self.regret_accumulator[hand + action * self.num_hands] > 0.0 {
                                strategy[hand + action * self.num_hands] = self.regret_accumulator
                                    [hand + action * self.num_hands]
                                    / normalizing_value
                            }
                        }
                    } else {
                        for action in 0..self.num_actions {
                            strategy[hand + action * self.num_hands] = probability;
                        }
                    }
                }
            }
        }

        strategy
    }

    #[cfg(all(target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    unsafe fn get_strategy_avx2_optimized(&self) -> Vec<f32> {
        let nums = self.num_actions * self.num_hands;
        if self.num_actions == 1 {
            return vec![1.0; nums];
        }

        let mut strategy = vec![0.0; nums];
        let left_over = self.num_hands % 8;
        let simd_stop_index = self.num_hands - left_over;

        let probability = 1.0 / (self.num_actions as f32);

        for hand in simd_stop_index..self.num_hands {
            let mut normalizing_value = 0.0;
            for action in 0..self.num_actions {
                if self.regret_accumulator[hand + action * self.num_hands] > 0.0 {
                    normalizing_value += self.regret_accumulator[hand + action * self.num_hands];
                }
            }

            if normalizing_value > 0.0 {
                for action in 0..self.num_actions {
                    if self.regret_accumulator[hand + action * self.num_hands] > 0.0 {
                        strategy[hand + action * self.num_hands] = self.regret_accumulator
                            [hand + action * self.num_hands]
                            / normalizing_value
                    }
                }
            } else {
                for action in 0..self.num_actions {
                    strategy[hand + action * self.num_hands] = probability;
                }
            }
        }
        let zeros = _mm256_set1_ps(0.0);
        let probability_vec = _mm256_set1_ps(probability);

        // loop unrolling here is very worth doing ~50% increases in speed and get strategy is 20% of runtime
        // general case has some comments to make this semi understandable
        match self.num_actions {
            2 => {
                let (strategy0, strategy1) = strategy.split_at_mut(self.num_hands);
                for hand in (0..simd_stop_index).step_by(8) {
                    let regret_sum0 = self.regret_accumulator.get_unchecked(hand);
                    let regret_sum1 = self.regret_accumulator.get_unchecked(self.num_hands + hand);

                    // sum positive regrets
                    let regret_with_negatives_zeroed_0 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum0),
                        zeros,
                    );
                    let regret_with_negatives_zeroed_1 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum1),
                        zeros,
                    );

                    let norm = _mm256_add_ps(
                        regret_with_negatives_zeroed_0,
                        regret_with_negatives_zeroed_1,
                    );

                    let mask = _mm256_cmp_ps::<_CMP_EQ_OS>(zeros, norm);

                    let result0 = _mm256_div_ps(
                        regret_with_negatives_zeroed_0,
                        norm,
                    );
                    let result1 = _mm256_div_ps(
                        regret_with_negatives_zeroed_1,
                        norm,
                    );

                    _mm256_storeu_ps(
                        strategy0.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result0, probability_vec, mask),
                    );
                    _mm256_storeu_ps(
                        strategy1.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result1, probability_vec, mask),
                    );
                }
            }
            3 => {
                let (strategy0, strategy12) = strategy.split_at_mut(self.num_hands);
                let (strategy1, strategy2) = strategy12.split_at_mut(self.num_hands);
                for hand in (0..simd_stop_index).step_by(8) {
                    let regret_sum0 = self.regret_accumulator.get_unchecked(hand);
                    let regret_sum1 = self.regret_accumulator.get_unchecked(self.num_hands + hand);
                    let regret_sum2 = self.regret_accumulator.get_unchecked(self.num_hands + self.num_hands + hand);

                    // sum positive regrets
                    let regret_with_negatives_zeroed_0 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum0),
                        zeros,
                    );
                    let regret_with_negatives_zeroed_1 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum1),
                        zeros,
                    );
                    let regret_with_negatives_zeroed_2 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum2),
                        zeros,
                    );

                    let norm = _mm256_add_ps(
                        _mm256_add_ps(
                            regret_with_negatives_zeroed_0,
                            regret_with_negatives_zeroed_1,
                        ),
                        regret_with_negatives_zeroed_2
                    );

                    let mask = _mm256_cmp_ps::<_CMP_EQ_OS>(zeros, norm);

                    let result0 = _mm256_div_ps(
                        regret_with_negatives_zeroed_0,
                        norm,
                    );
                    let result1 = _mm256_div_ps(
                        regret_with_negatives_zeroed_1,
                        norm,
                    );
                    let result2 = _mm256_div_ps(
                        regret_with_negatives_zeroed_2,
                        norm,
                    );

                    _mm256_storeu_ps(
                        strategy0.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result0, probability_vec, mask),
                    );
                    _mm256_storeu_ps(
                        strategy1.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result1, probability_vec, mask),
                    );
                    _mm256_storeu_ps(
                        strategy2.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result2, probability_vec, mask),
                    );
                }
            }
            4 => {
                let (strategy0, strategy12) = strategy.split_at_mut(self.num_hands);
                let (strategy1, strategy23) = strategy12.split_at_mut(self.num_hands);
                let (strategy2, strategy3) = strategy23.split_at_mut(self.num_hands);
                for hand in (0..simd_stop_index).step_by(8) {
                    let regret_sum0 = self.regret_accumulator.get_unchecked(hand);
                    let regret_sum1 = self.regret_accumulator.get_unchecked(self.num_hands + hand);
                    let regret_sum2 = self.regret_accumulator.get_unchecked(self.num_hands + self.num_hands + hand);
                    let regret_sum3 = self.regret_accumulator.get_unchecked(self.num_hands + self.num_hands + self.num_hands + hand);

                    // sum positive regrets
                    let regret_with_negatives_zeroed_0 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum0),
                        zeros,
                    );
                    let regret_with_negatives_zeroed_1 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum1),
                        zeros,
                    );
                    let regret_with_negatives_zeroed_2 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum2),
                        zeros,
                    );
                    let regret_with_negatives_zeroed_3 = _mm256_max_ps(
                        _mm256_loadu_ps(regret_sum3),
                        zeros,
                    );

                    let norm = _mm256_add_ps(
                        _mm256_add_ps(
                            _mm256_add_ps(
                                regret_with_negatives_zeroed_0,
                                regret_with_negatives_zeroed_1,
                            ),
                            regret_with_negatives_zeroed_2
                        ),
                        regret_with_negatives_zeroed_3
                    );

                    let mask = _mm256_cmp_ps::<_CMP_EQ_OS>(zeros, norm);

                    let result0 = _mm256_div_ps(
                        regret_with_negatives_zeroed_0,
                        norm,
                    );
                    let result1 = _mm256_div_ps(
                        regret_with_negatives_zeroed_1,
                        norm,
                    );
                    let result2 = _mm256_div_ps(
                        regret_with_negatives_zeroed_2,
                        norm,
                    );
                    let result3 = _mm256_div_ps(
                        regret_with_negatives_zeroed_3,
                        norm,
                    );

                    _mm256_storeu_ps(
                        strategy0.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result0, probability_vec, mask),
                    );
                    _mm256_storeu_ps(
                        strategy1.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result1, probability_vec, mask),
                    );
                    _mm256_storeu_ps(
                        strategy2.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result2, probability_vec, mask),
                    );
                    _mm256_storeu_ps(
                        strategy3.get_unchecked_mut(hand),
                        _mm256_blendv_ps(result3, probability_vec, mask),
                    );
                }
            }
            _ => {
                for hand in (0..simd_stop_index).step_by(8) {
                    let mut normalizing_vec_ptr = [0.0; 8].as_mut_ptr();
                    for action in 0..self.num_actions {
                        let regret_with_negatives_zeroed = _mm256_max_ps(
                            _mm256_loadu_ps(
                                self.regret_accumulator
                                    .get_unchecked(hand + action * self.num_hands),
                            ),
                            zeros,
                        );

                        let regret_sum = _mm256_add_ps(
                            _mm256_loadu_ps(normalizing_vec_ptr),
                            regret_with_negatives_zeroed,
                        );

                        _mm256_storeu_ps(normalizing_vec_ptr, regret_sum);
                    }

                    for action in 0..self.num_actions {
                        let norm = _mm256_loadu_ps(normalizing_vec_ptr);
                        // div by 0 results in inf values, we find the 0 values here
                        let mask = _mm256_cmp_ps::<_CMP_EQ_OS>(zeros, norm);
                        let regret_with_negatives_zeroed = _mm256_max_ps(
                            _mm256_loadu_ps(
                                self.regret_accumulator
                                    .get_unchecked(hand + action * self.num_hands),
                            ),
                            zeros,
                        );

                        let result = _mm256_div_ps(regret_with_negatives_zeroed, norm);

                        // using the mask above, take the probability_vec if the mask was true (norm = 0)
                        // and otherwise take the real result
                        _mm256_storeu_ps(
                            strategy.get_unchecked_mut(hand + action * self.num_hands),
                            _mm256_blendv_ps(result, probability_vec, mask),
                        );
                    }
                }
            }
        }

        strategy
    }

    #[cfg(all(target_arch = "aarch64"))]
    #[target_feature(enable = "neon")]
    unsafe fn get_strategy_neon_optimized(&self) -> Vec<f32> {
        let nums = self.num_actions * self.num_hands;
        if self.num_actions == 1 {
            return vec![1.0; nums];
        }

        let mut strategy = vec![0.0; nums];
        let left_over = self.num_hands % 4;
        let simd_stop_index = self.num_hands - left_over;

        let probability = 1.0 / (self.num_actions as f32);

        for hand in simd_stop_index..self.num_hands {
            let mut normalizing_value = 0.0;
            for action in 0..self.num_actions {
                if self.regret_accumulator[hand + action * self.num_hands] > 0.0 {
                    normalizing_value += self.regret_accumulator[hand + action * self.num_hands];
                }
            }

            if normalizing_value > 0.0 {
                for action in 0..self.num_actions {
                    if self.regret_accumulator[hand + action * self.num_hands] > 0.0 {
                        strategy[hand + action * self.num_hands] = self.regret_accumulator
                            [hand + action * self.num_hands]
                            / normalizing_value
                    }
                }
            } else {
                for action in 0..self.num_actions {
                    strategy[hand + action * self.num_hands] = probability;
                }
            }
        }
        let zeros = vld1q_dup_f32(0.0.borrow());
        let probability_vec = vld1q_dup_f32(probability.borrow());

        // loop unrolling here is very worth doing ~50% increases in speed and get strategy is 20% of runtime
        // general case has some comments to make this semi understandable
        match self.num_actions {
            2 => {
                let (strategy0, strategy1) = strategy.split_at_mut(self.num_hands);
                for hand in (0..simd_stop_index).step_by(4) {
                    let regret_sum0 = self.regret_accumulator.get_unchecked(hand);
                    let regret_sum1 = self.regret_accumulator.get_unchecked(self.num_hands + hand);

                    // sum positive regrets
                    let regret_with_negatives_zeroed_0 = vmaxq_f32(vld1q_f32(regret_sum0), zeros);
                    let regret_with_negatives_zeroed_1 = vmaxq_f32(vld1q_f32(regret_sum1), zeros);

                    let norm = vaddq_f32(
                        regret_with_negatives_zeroed_0,
                        regret_with_negatives_zeroed_1,
                    );
                    let mask = vceqq_f32(zeros, norm);

                    let result0 = vdivq_f32(regret_with_negatives_zeroed_0, norm);
                    let result1 = vdivq_f32(regret_with_negatives_zeroed_1, norm);

                    vst1q_f32(
                        strategy0.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result0),
                    );
                    vst1q_f32(
                        strategy1.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result1),
                    );
                }
            }
            3 => {
                let (strategy0, strategy12) = strategy.split_at_mut(self.num_hands);
                let (strategy1, strategy2) = strategy12.split_at_mut(self.num_hands);
                for hand in (0..simd_stop_index).step_by(4) {
                    let regret_sum0 = self.regret_accumulator.get_unchecked(hand);
                    let regret_sum1 = self.regret_accumulator.get_unchecked(self.num_hands + hand);
                    let regret_sum2 = self
                        .regret_accumulator
                        .get_unchecked(self.num_hands + self.num_hands + hand);

                    // sum positive regrets
                    let regret_with_negatives_zeroed_0 = vmaxq_f32(vld1q_f32(regret_sum0), zeros);
                    let regret_with_negatives_zeroed_1 = vmaxq_f32(vld1q_f32(regret_sum1), zeros);
                    let regret_with_negatives_zeroed_2 = vmaxq_f32(vld1q_f32(regret_sum2), zeros);

                    let norm = vaddq_f32(
                        vaddq_f32(
                            regret_with_negatives_zeroed_0,
                            regret_with_negatives_zeroed_1,
                        ),
                        regret_with_negatives_zeroed_2,
                    );

                    let mask = vceqq_f32(zeros, norm);

                    let result0 = vdivq_f32(regret_with_negatives_zeroed_0, norm);
                    let result1 = vdivq_f32(regret_with_negatives_zeroed_1, norm);
                    let result2 = vdivq_f32(regret_with_negatives_zeroed_2, norm);

                    vst1q_f32(
                        strategy0.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result0),
                    );
                    vst1q_f32(
                        strategy1.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result1),
                    );
                    vst1q_f32(
                        strategy2.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result2),
                    );
                }
            }
            4 => {
                let (strategy0, strategy12) = strategy.split_at_mut(self.num_hands);
                let (strategy1, strategy23) = strategy12.split_at_mut(self.num_hands);
                let (strategy2, strategy3) = strategy23.split_at_mut(self.num_hands);
                for hand in (0..simd_stop_index).step_by(4) {
                    let regret_sum0 = self.regret_accumulator.get_unchecked(hand);
                    let regret_sum1 = self.regret_accumulator.get_unchecked(self.num_hands + hand);
                    let regret_sum2 = self
                        .regret_accumulator
                        .get_unchecked(self.num_hands + self.num_hands + hand);
                    let regret_sum3 = self
                        .regret_accumulator
                        .get_unchecked(self.num_hands + self.num_hands + self.num_hands + hand);

                    // sum positive regrets
                    let regret_with_negatives_zeroed_0 = vmaxq_f32(vld1q_f32(regret_sum0), zeros);
                    let regret_with_negatives_zeroed_1 = vmaxq_f32(vld1q_f32(regret_sum1), zeros);
                    let regret_with_negatives_zeroed_2 = vmaxq_f32(vld1q_f32(regret_sum2), zeros);
                    let regret_with_negatives_zeroed_3 = vmaxq_f32(vld1q_f32(regret_sum3), zeros);

                    let norm = vaddq_f32(
                        vaddq_f32(
                            vaddq_f32(
                                regret_with_negatives_zeroed_0,
                                regret_with_negatives_zeroed_1,
                            ),
                            regret_with_negatives_zeroed_2,
                        ),
                        regret_with_negatives_zeroed_3,
                    );

                    let mask = vceqq_f32(zeros, norm);

                    let result0 = vdivq_f32(regret_with_negatives_zeroed_0, norm);
                    let result1 = vdivq_f32(regret_with_negatives_zeroed_1, norm);
                    let result2 = vdivq_f32(regret_with_negatives_zeroed_2, norm);
                    let result3 = vdivq_f32(regret_with_negatives_zeroed_3, norm);

                    vst1q_f32(
                        strategy0.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result0),
                    );
                    vst1q_f32(
                        strategy1.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result1),
                    );
                    vst1q_f32(
                        strategy2.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result2),
                    );
                    vst1q_f32(
                        strategy3.get_unchecked_mut(hand),
                        vbslq_f32(mask, probability_vec, result3),
                    );
                }
            }
            _ => {
                for hand in (0..simd_stop_index).step_by(4) {
                    let mut normalizing_vec_ptr = [0.0; 4].as_mut_ptr();
                    for action in 0..self.num_actions {
                        let regret_with_negatives_zeroed = vmaxq_f32(
                            vld1q_f32(
                                self.regret_accumulator
                                    .get_unchecked(hand + action * self.num_hands),
                            ),
                            zeros,
                        );

                        let regret_sum =
                            vaddq_f32(vld1q_f32(normalizing_vec_ptr), regret_with_negatives_zeroed);

                        vst1q_f32(normalizing_vec_ptr, regret_sum);
                    }

                    for action in 0..self.num_actions {
                        let norm = vld1q_f32(normalizing_vec_ptr);
                        // div by 0 results in inf values, we find the 0 values here
                        let mask = vceqq_f32(zeros, norm);
                        let regret_with_negatives_zeroed = vmaxq_f32(
                            vld1q_f32(
                                self.regret_accumulator
                                    .get_unchecked(hand + action * self.num_hands),
                            ),
                            zeros,
                        );

                        let result = vdivq_f32(regret_with_negatives_zeroed, norm);

                        // using the mask above, take the probability_vec if the mask was true (norm = 0)
                        // and otherwise take the real result
                        vst1q_f32(
                            strategy.get_unchecked_mut(hand + action * self.num_hands),
                            vbslq_f32(mask, probability_vec, result),
                        );
                    }
                }
            }
        }

        strategy
    }

    fn get_average_strategy(&self) -> Vec<f32> {
        let nums = self.num_actions * self.num_hands;
        let mut average_strategy = vec![0.0; nums];

        for hand in 0..self.num_hands {
            let mut normalizing_value = 0.0;
            for action in 0..self.num_actions {
                normalizing_value += self.strategy_accumulator[hand + action * self.num_hands];
            }

            if normalizing_value > 0.0 {
                for action in 0..self.num_actions {
                    average_strategy[hand + action * self.num_hands] += self.strategy_accumulator
                        [hand + action * self.num_hands]
                        / normalizing_value;
                }
            } else {
                let probability = 1.0 / (self.num_actions as f32);
                for action in 0..self.num_actions {
                    average_strategy[hand + action * self.num_hands] = probability;
                }
            }
        }

        average_strategy
    }

    fn regret_sum_update(
        &mut self,
        traversal: &Traversal,
        action_utility: &Vec<Vec<f32>>,
        node_utility: &[f32],
    ) {
        let alpha = f64::from(traversal.iteration).powf(1.45);
        let positive_multiplier = alpha / (alpha + 1.0);
        let negative_multiplier = 0.5;

        for (action, action_util) in action_utility.iter().enumerate() {
            self.regret_accumulator[action * self.num_hands..(action + 1) * self.num_hands]
                .iter_mut()
                .zip(action_util.iter())
                .zip(node_utility.iter())
                .for_each(|((regret, action_util), node_util)| {
                    *regret += action_util - node_util;
                    if *regret > 0.0 {
                        *regret *= positive_multiplier as f32;
                    } else {
                        *regret *= negative_multiplier;
                    }
                });
        }
    }

    fn strategy_sum_update(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        strategies: &[f32],
    ) {
        let strategy_multiplier =
            (f64::from(traversal.iteration) / f64::from(traversal.iteration + 1)).powi(2) as f32;
        let round_multiplier = 0.98;

        match self.num_actions {
            2 => {
                let strategy0 = &strategies[0..self.num_hands];
                let strategy1 = &strategies[self.num_hands..];

                let (strategy_sum0, strategy_sum1) =
                    self.strategy_accumulator.split_at_mut(self.num_hands);

                strategy_sum0
                    .iter_mut()
                    .zip(strategy_sum1.iter_mut())
                    .zip(strategy0.iter())
                    .zip(strategy1.iter())
                    .zip(op_reach_prob.iter())
                    .for_each(|((((sum_0, sum_1), strat0), strat1), prob)| {
                        *sum_0 =
                            ((*sum_0 * round_multiplier) + (strat0 * prob)) * strategy_multiplier;
                        *sum_1 =
                            ((*sum_1 * round_multiplier) + (strat1 * prob)) * strategy_multiplier;
                    });
            }
            3 => {
                let strategy0 = &strategies[0..self.num_hands];
                let strategy1 = &strategies[self.num_hands..self.num_hands * 2];
                let strategy2 = &strategies[self.num_hands * 2..];

                let (strategy_sum0, strategy_sum12) =
                    self.strategy_accumulator.split_at_mut(self.num_hands);
                let (strategy_sum1, strategy_sum2) = strategy_sum12.split_at_mut(self.num_hands);

                strategy_sum0
                    .iter_mut()
                    .zip(strategy_sum1.iter_mut())
                    .zip(strategy_sum2.iter_mut())
                    .zip(strategy0.iter())
                    .zip(strategy1.iter())
                    .zip(strategy2.iter())
                    .zip(op_reach_prob.iter())
                    .for_each(
                        |((((((sum_0, sum_1), sum_2), strat0), strat1), strat2), prob)| {
                            *sum_0 = ((*sum_0 * round_multiplier) + (strat0 * prob))
                                * strategy_multiplier;
                            *sum_1 = ((*sum_1 * round_multiplier) + (strat1 * prob))
                                * strategy_multiplier;
                            *sum_2 = ((*sum_2 * round_multiplier) + (strat2 * prob))
                                * strategy_multiplier;
                        },
                    );
            }
            _ => {
                for action in 0..self.num_actions {
                    let strategy_slice = &strategies[action * self.num_hands..];
                    self.strategy_accumulator
                        [action * self.num_hands..(action + 1) * self.num_hands]
                        .iter_mut()
                        .zip(op_reach_prob.iter())
                        .zip(strategy_slice.iter())
                        .for_each(|((strategy_sum, prob), strategy)| {
                            *strategy_sum = ((*strategy_sum * round_multiplier)
                                + (strategy * prob))
                                * strategy_multiplier;
                        });
                }
            }
        }
    }

    fn traverser_cfr(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        node_utility: &mut [f32],
        board: &Board,
    ) {
        let mut action_utility = vec![vec![0.0; 0]; self.num_actions];
        let strategies = self.get_strategy();

        for i in 0..self.num_actions {
            let action_offset = i * self.num_hands;
            let result = self.next_nodes[i].cfr_traversal(traversal, op_reach_prob, board);
            let strategy_slice = &strategies[action_offset..];

            node_utility
                .iter_mut()
                .zip(strategy_slice.iter())
                .zip(result.iter())
                .for_each(|((node, strategy), result)| {
                    *node += strategy * result;
                });

            action_utility[i] = result;
        }

        self.regret_sum_update(traversal, &action_utility, node_utility)
    }

    fn opponent_cfr(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        node_utility: &mut [f32],
        board: &Board,
    ) {
        let strategies = self.get_strategy();

        for i in 0..self.num_actions {
            let action_offset = i * self.num_hands;
            let strategy_slice = &strategies[action_offset..];
            let mut next_reach_prob = vec![0.0; op_reach_prob.len()];

            next_reach_prob
                .iter_mut()
                .zip(strategy_slice.iter())
                .zip(op_reach_prob.iter())
                .for_each(|((next, strategy), prob)| {
                    *next = strategy * prob;
                });

            let result = self.next_nodes[i].cfr_traversal(traversal, &next_reach_prob, board);

            node_utility
                .iter_mut()
                .zip(result.iter())
                .for_each(|(utility, result)| {
                    *utility += result;
                });
        }

        self.strategy_sum_update(traversal, op_reach_prob, &strategies)
    }
}

extern crate test;

use crate::cfr::traversal::build_traversal_from_ranges;
use crate::ranges::utility::construct_starting_range_from_string;
use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};
use test::Bencher;
use tracing::info;

#[cfg(test)]
mod tests {
    use crate::{
        cfr::traversal::Traversal,
        nodes::node::CfrNode,
        ranges::{range_manager::RangeManager, utility::construct_starting_range_from_string},
    };
    use rand::random;
    use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};

    extern crate test;

    use crate::cfr::traversal::build_traversal_from_ranges;
    use crate::nodes::action_node::ActionNode;
    use test::Bencher;

    const NUM_HANDS: usize = 600;
    const NUM_ACTIONS: usize = 4;

    fn build_node() -> ActionNode {
        ActionNode {
            player_node: 0,
            num_hands: NUM_HANDS,
            num_actions: NUM_ACTIONS,
            pot_size: 0.0,
            ip_stack: 0.0,
            oop_stack: 0.0,
            next_nodes: vec![],
            regret_accumulator: (0..NUM_ACTIONS * NUM_HANDS)
                .map(|_| {
                    let r: f32 = random();
                    if r < 0.5 {
                        -100.0 * r
                    } else {
                        r * 100.0
                    }
                })
                .collect(),
            strategy_accumulator: vec![0.0; NUM_ACTIONS * NUM_HANDS],
            node_ev: None,
        }
    }

    #[bench]
    fn standard_strategy(b: &mut Bencher) {
        let mut node = build_node();

        b.iter(|| {
            test::black_box(node.get_strategy_fallback());
        });
    }

    #[bench]
    fn standard_update_strategy(b: &mut Bencher) {
        let mut node = build_node();

        let strategy = node.get_strategy();
        let traversal = build_traversal_from_ranges([2, 4, 5, 52, 52], "random", "random");
        let prob = vec![0.5; NUM_HANDS];

        b.iter(|| {
            test::black_box(node.strategy_sum_update(&traversal, &prob, &strategy));
        });
    }

    #[bench]
    fn standard_update_regret(b: &mut Bencher) {
        let mut node = build_node();

        let strategy = node.get_strategy();
        let traversal = build_traversal_from_ranges([2, 4, 5, 52, 52], "random", "random");
        let util = vec![0.5; NUM_HANDS];
        let action = vec![vec![0.5; NUM_HANDS]; NUM_ACTIONS];

        b.iter(|| {
            test::black_box(node.regret_sum_update(&traversal, &action, &util));
        });
    }

    #[cfg(all(target_arch = "x86_64"))]
    #[bench]
    fn avx2_strategy(b: &mut Bencher) {
        let mut node = build_node();

        b.iter(|| {
            test::black_box(unsafe { node.get_strategy_avx2_optimized() });
        });
    }

    #[cfg(all(target_arch = "x86_64"))]
    #[test]
    fn test_strategy_avx2() {
        let mut node = build_node();

        let r1 = node.get_strategy_fallback();
        let r2 = unsafe { node.get_strategy_avx2_optimized() };

        assert_eq!(r1, r2);
    }

    #[cfg(all(target_arch = "aarch64"))]
    #[bench]
    fn neon_strategy(b: &mut Bencher) {
        let mut node = build_node();

        b.iter(|| {
            test::black_box(unsafe { node.get_strategy_neon_optimized() });
        });
    }

    #[cfg(all(target_arch = "aarch64"))]
    #[test]
    fn test_utility() {
        let mut node = build_node();

        let r1 = node.get_strategy_fallback();
        let r2 = unsafe { node.get_strategy_neon_optimized() };

        assert_eq!(r1, r2);
    }
}
