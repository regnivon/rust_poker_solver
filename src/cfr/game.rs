use crate::nodes::all_in_showdown_node::AllInShowdownNode;
use crate::nodes::chance_node::ChanceNode;
use crate::nodes::node::CfrNode;

use crate::ranges::utility::range_relative_probabilities;
use crate::{
    nodes::{
        action_node::ActionNode, node::Node, showdown_node::ShowdownNode,
        terminal_node::TerminalNode,
    },
    ranges::{combination::Board, utility::unblocked_hands},
};

use super::{game_params::GameParams, traversal::Traversal};

pub struct Game {
    traversal: Traversal,
    root: Node,
    game_params: GameParams,
    starting_board: Board,
}

impl Game {
    pub fn new(traversal: Traversal, game_params: GameParams, starting_board: Board) -> Self {
        Self {
            traversal,
            game_params,
            root: Node::from(ShowdownNode::new(0.0)),
            starting_board,
        }
    }

    pub fn train(&mut self, iterations: u32) {
        self.construct_tree();

        self.traversal.traverser = 0;

        let ip_range = self.traversal.get_range_for_opponent(&self.starting_board);
        let oop_range = self
            .traversal
            .get_range_for_active_player(&self.starting_board);

        let ip = vec![1.0; ip_range.len()];
        let oop = vec![1.0; oop_range.len()];

        let ip_relative_probs = range_relative_probabilities(ip_range, oop_range);
        let oop_relative_probs = range_relative_probabilities(oop_range, ip_range);

        let oop_br = self.overall_best_response(&oop_relative_probs, &ip, &self.starting_board);
        self.traversal.traverser = 1;
        let ip_br = self.overall_best_response(&ip_relative_probs, &oop, &self.starting_board);
        let exploitability = (ip_br + oop_br) / 2.0 / self.game_params.starting_pot * 100.0;
        println!(
            "Iteration 0 OOP BR {} IP BR {} exploitability = {} percent of the pot",
            oop_br, ip_br, exploitability
        );

        for i in 0..=iterations {
            self.traversal.iteration = i;
            self.traversal.traverser = 0;
            self.root
                .cfr_traversal(&self.traversal, &ip, &self.starting_board);
            self.traversal.traverser = 1;
            self.root
                .cfr_traversal(&self.traversal, &ip, &self.starting_board);
            if i > 0 && i % 25 == 0 {
                self.traversal.traverser = 0;
                let oop_br =
                    self.overall_best_response(&oop_relative_probs, &ip, &self.starting_board);
                self.traversal.traverser = 1;
                let ip_br =
                    self.overall_best_response(&ip_relative_probs, &oop, &self.starting_board);
                let exploitability = (ip_br + oop_br) / 2.0 / self.game_params.starting_pot * 100.0;
                println!(
                    "Iteration {} OOP BR {} IP BR {} exploitability = {} percent of the pot",
                    i, oop_br, ip_br, exploitability
                );
            }
        }
    }

    fn overall_best_response(
        &self,
        responder_relative_probs: &[f32],
        opp_reach_probs: &[f32],
        board: &Board,
    ) -> f32 {
        let responder_hands = self.traversal.get_range_for_active_player(board);
        let opponent_hands = self.traversal.get_range_for_opponent(board);

        let unblocked = unblocked_hands(responder_hands, opponent_hands);

        let evs = self
            .root
            .best_response(&self.traversal, opp_reach_probs, board);

        let mut sum = 0.0;

        for i in 0..evs.len() {
            sum += evs[i] * responder_relative_probs[i] / unblocked[i];
        }
        sum
    }

    fn construct_tree(&mut self) {
        let mut root = ActionNode::new(
            0,
            self.traversal
                .get_num_hands_for_traverser(&self.starting_board),
            self.game_params.starting_pot,
            self.game_params.starting_stack,
            self.game_params.starting_stack,
        );

        let board = self.starting_board;

        self.add_successor_nodes(&mut root, 0, &board);

        self.root = Node::from(root);
    }

    fn add_successor_nodes(&mut self, root: &mut ActionNode, bet_number: u8, board: &Board) {
        let mut street = 3;
        if board[3] == 52 {
            street = 1;
        } else if board[4] == 52 {
            street = 2;
        }

        if root.player_node == 1 || bet_number > 0 {
            self.create_next_call_check_and_fold_nodes(root, bet_number, street, board);
        } else {
            self.create_check_to_ip_node(root, bet_number, street, board);
        }

        if root.oop_stack > 0.0 && root.ip_stack > 0.0 {
            self.create_next_bet_nodes(root, bet_number, street, board)
        }

        root.init_vectors();
    }

    fn create_next_call_check_and_fold_nodes(
        &mut self,
        root: &mut ActionNode,
        bet_number: u8,
        street: u8,
        board: &Board,
    ) {
        let last_bet_size = (root.ip_stack - root.oop_stack).abs();
        let call_stacks = root.ip_stack.min(root.oop_stack);

        if street == 3 {
            let next = ShowdownNode::new(root.pot_size + last_bet_size);
            root.add_child(Node::from(next));
        } else if call_stacks == 0.0 {
            let next = AllInShowdownNode::new(root.pot_size + last_bet_size, street);
            root.add_child(Node::from(next));
        } else {
            let mut next = if self.game_params.parallel_street == street {
                ChanceNode::new(board, street, true)
            } else {
                ChanceNode::new(board, street, false)
            };

            let next_cards = next.next_cards.clone();

            for card in next_cards {
                let mut new_board = *board;
                if street == 1 {
                    new_board[3] = card;
                } else {
                    new_board[4] = card;
                }

                let mut next_game_node = ActionNode::new(
                    0,
                    self.traversal.get_num_hands_for_player(0, &new_board),
                    root.pot_size + last_bet_size,
                    call_stacks,
                    call_stacks,
                );

                self.add_successor_nodes(&mut next_game_node, 0, &new_board);
                next.add_next_node(Node::from(next_game_node));
            }

            root.add_child(Node::from(next));
        }

        if bet_number > 0 {
            let fold = TerminalNode::new(root.pot_size - last_bet_size, root.player_node ^ 1);
            root.add_child(Node::from(fold));
        }
    }

    fn create_check_to_ip_node(
        &mut self,
        root: &mut ActionNode,
        _bet_number: u8,
        _street: u8,
        board: &Board,
    ) {
        let mut next = ActionNode::new(
            1,
            self.traversal.get_num_hands_for_player(1, board),
            root.pot_size,
            root.ip_stack,
            root.oop_stack,
        );

        self.add_successor_nodes(&mut next, 0, board);

        root.add_child(Node::from(next));
    }

    fn create_next_bet_nodes(
        &mut self,
        root: &mut ActionNode,
        bet_number: u8,
        street: u8,
        board: &Board,
    ) {
        let current_bets = if root.pot_size * self.game_params.all_in_cut_off
            >= root.ip_stack.max(root.oop_stack)
        {
            vec![self.game_params.all_in_cut_off; 1]
        } else {
            self.get_current_bets(street, root.player_node, bet_number)
                .to_vec()
        };

        for bet_size in current_bets.iter() {
            let last_bet = (root.oop_stack - root.ip_stack).abs();
            let sizing = bet_size * (root.pot_size + last_bet) + last_bet;

            if root.player_node == 1 {
                let final_bet_size = (root.ip_stack.min(sizing)).min(root.oop_stack + last_bet);
                let mut next = ActionNode::new(
                    0,
                    self.traversal.get_num_hands_for_player(0, board),
                    root.pot_size + final_bet_size,
                    root.ip_stack - final_bet_size,
                    root.oop_stack,
                );

                self.add_successor_nodes(&mut next, bet_number + 1, board);
                root.add_child(Node::from(next));
                if final_bet_size < sizing {
                    break;
                }
            } else {
                let final_bet_size = (root.oop_stack.min(sizing)).min(root.ip_stack + last_bet);
                let mut next = ActionNode::new(
                    1,
                    self.traversal.get_num_hands_for_player(1, board),
                    root.pot_size + final_bet_size,
                    root.ip_stack,
                    root.oop_stack - final_bet_size,
                );

                self.add_successor_nodes(&mut next, bet_number + 1, board);
                root.add_child(Node::from(next));
                if final_bet_size < sizing {
                    break;
                }
            }
        }
    }

    fn get_current_bets(&self, street: u8, player: u8, bet_number: u8) -> &Vec<f32> {
        let bet = usize::from(bet_number);
        if street == 1 {
            if player == 0 && bet < self.game_params.oop_flop_bets.len() {
                return &self.game_params.oop_flop_bets[bet];
            } else if bet < self.game_params.ip_flop_bets.len() {
                return &self.game_params.ip_flop_bets[bet];
            }
        } else if street == 2 {
            if player == 0 && bet < self.game_params.oop_turn_bets.len() {
                return &self.game_params.oop_turn_bets[bet];
            } else if bet < self.game_params.ip_turn_bets.len() {
                return &self.game_params.ip_turn_bets[bet];
            }
        } else if street == 3 {
            if player == 0 && bet < self.game_params.oop_river_bets.len() {
                return &self.game_params.oop_river_bets[bet];
            } else if bet < self.game_params.ip_river_bets.len() {
                return &self.game_params.ip_river_bets[bet];
            }
        }
        &self.game_params.default_bets[0]
    }
}
