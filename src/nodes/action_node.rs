use crate::{cfr::traversal::Traversal, ranges::combination::Board};

use super::node::{CfrNode, Node};

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
        &self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32> {
        if self.player_node == traversal.traverser {
            let mut best_ev = vec![0.0; self.num_hands];
            for action in 0..self.num_actions {
                let next_ev =
                    self.next_nodes[action].best_response(traversal, op_reach_prob, board);
                for hand_index in 0..best_ev.len() {
                    if action == 0 || next_ev[hand_index] > best_ev[hand_index] {
                        best_ev[hand_index] = next_ev[hand_index];
                    }
                }
            }
            best_ev
        } else {
            let mut node_ev = vec![0.0; traversal.get_num_hands_for_traverser(board)];
            let average_strategy = self.get_average_strategy();
            for action in 0..self.num_actions {
                let action_offset = action * self.num_hands;
                let mut next_reach = vec![0.0; op_reach_prob.len()];
                for j in 0..self.num_hands {
                    next_reach[j] = average_strategy[j + action_offset] * op_reach_prob[j];
                }

                let action_ev =
                    self.next_nodes[action].best_response(traversal, &next_reach, board);
                for j in 0..self.num_hands {
                    node_ev[j] += action_ev[j];
                }
            }
            node_ev
        }
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
        }
    }

    pub fn init_vectors(&mut self) {
        self.regret_accumulator = vec![0.0; self.num_actions * self.num_hands];
        self.strategy_accumulator = vec![0.0; self.num_actions * self.num_hands];
    }

    pub fn add_child(&mut self, child: Node) {
        self.num_actions += 1;
        self.next_nodes.push(child);
    }

    fn get_strategy(&self) -> Vec<f32> {
        let nums = self.num_actions * self.num_hands;
        let mut strategy = vec![0.0; nums];

        let probability = 1.0 / (self.num_actions as f32);

        for hand in 0..self.num_hands {
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
        action_utility: &[f32],
        node_utility: &[f32],
    ) {
        let alpha = f64::from(traversal.iteration).powf(1.5);
        let positive_multiplier = alpha / (alpha + 1.0);
        let negative_multiplier = 0.5;

        for action in 0..self.num_actions {
            let offset = action * self.num_hands;
            for hand in 0..self.num_hands {
                self.regret_accumulator[hand + offset] +=
                    action_utility[hand + offset] - node_utility[hand];
                if self.regret_accumulator[hand + offset] > 0.0 {
                    self.regret_accumulator[hand + offset] *= positive_multiplier as f32;
                } else {
                    self.regret_accumulator[hand + offset] *= negative_multiplier;
                }
            }
        }
    }

    fn strategy_sum_update(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        strategies: &[f32],
    ) {
        let strategy_multiplier =
            (f64::from(traversal.iteration) / f64::from(traversal.iteration + 1)).powi(2);

        for action in 0..self.num_actions {
            for hand in 0..self.num_hands {
                self.strategy_accumulator[hand + action * self.num_hands] *= 0.9;
                self.strategy_accumulator[hand + action * self.num_hands] +=
                    op_reach_prob[hand] * strategies[hand + action * self.num_hands];
                self.strategy_accumulator[hand + action * self.num_hands] *=
                    strategy_multiplier as f32;
            }
        }
    }

    fn traverser_cfr(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        node_utility: &mut Vec<f32>,
        board: &Board,
    ) {
        let mut action_utility = vec![0.0; self.num_actions * self.num_hands];
        let strategies = self.get_strategy();

        for i in 0..self.num_actions {
            let action_offset = i * self.num_hands;

            let result = self.next_nodes[i].cfr_traversal(traversal, op_reach_prob, board);

            action_utility[action_offset..(node_utility.len() + action_offset)].clone_from_slice(&result[..node_utility.len()]);

            for hand in 0..node_utility.len() {
                node_utility[hand] += strategies[hand + action_offset] * result[hand];
            }
        }

        self.regret_sum_update(traversal, &action_utility, node_utility)
    }

    fn opponent_cfr(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        node_utility: &mut Vec<f32>,
        board: &Board,
    ) {
        let strategies = self.get_strategy();

        for i in 0..self.num_actions {
            let action_offset = i * self.num_hands;

            let mut next_reach_prob = vec![0.0; op_reach_prob.len()];
            for hand in 0..next_reach_prob.len() {
                next_reach_prob[hand] = strategies[hand + action_offset] * op_reach_prob[hand];
            }

            let result = self.next_nodes[i].cfr_traversal(traversal, &next_reach_prob, board);
            for hand in 0..node_utility.len() {
                node_utility[hand] += result[hand];
            }
        }

        self.strategy_sum_update(traversal, op_reach_prob, &strategies)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cfr::traversal::Traversal,
        nodes::{
            node::{CfrNode, Node},
            showdown_node::ShowdownNode,
            terminal_node::TerminalNode,
        },
        ranges::{range_manager::RangeManager, utility::construct_starting_range_from_string},
    };

    use super::ActionNode;

    #[test]
    fn test_cfr() {
        let mut action = ActionNode::new(1, 18, 10.0, 15.0, 10.0);
        let terminal_node = Node::from(TerminalNode::new(10.0, 0));
        let showdown_node = Node::from(ShowdownNode::new(20.0));

        action.add_child(terminal_node);
        action.add_child(showdown_node);
        action.init_vectors();

        let board = [51, 26, 20, 15, 11];

        let op_reach_prob = vec![1.0; 18];

        let traverser_hands = construct_starting_range_from_string("QQ,33,22".to_string(), &board);
        let opp_hands = construct_starting_range_from_string("QQ,33,22".to_string(), &board);

        let opp_rm = RangeManager::new(opp_hands, board);
        let ip_rm = RangeManager::new(traverser_hands, board);

        let mut trav = Traversal::new(opp_rm, ip_rm);
        trav.traverser = 1;

        let result = action.cfr_traversal(&trav, &op_reach_prob, &board);

        for i in 0..6 {
            assert_eq!(result[i], -92.5);
        }

        for i in 6..12 {
            assert_eq!(result[i], -32.5);
        }

        for i in 12..18 {
            assert_eq!(result[i], 27.5);
        }
    }
}
