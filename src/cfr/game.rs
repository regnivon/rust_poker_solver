use super::{game_params::GameParams, traversal::Traversal};
use crate::nodes::all_in_showdown_node::AllInShowdownNode;
use crate::nodes::chance_node::ChanceNode;
use crate::nodes::node::{CfrNode, NodeResult};
use crate::ranges::combination::Combination;
use crate::ranges::range_manager::RangeManager;
use crate::ranges::utility::{number_to_card, range_relative_probabilities};
use crate::{nodes::{
    action_node::ActionNode, node::Node, showdown_node::ShowdownNode,
    terminal_node::TerminalNode,
}, ranges::{combination::Board, utility::unblocked_hands}};
use cloud_storage::Client;

use serde::{Deserialize, Serialize};
use tracing::info;
use crate::cfr::traversal::build_traversal_from_ranges;
use crate::nodes::node::Node::{
    ShowdownNode as OtherShowdownNode,
    ActionNode as OtherActionNode,
    ChanceNode as OtherChanceNode,
    AllInShowdownNode as OtherAllInShowdownNode,
    TerminalNode as OtherTerminalNode
};

pub async fn run_trainer(
    board: Board,
    oop_range: &str,
    ip_range: &str,
    params: GameParams,
    bucket_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let traversal = build_traversal_from_ranges(board, oop_range, ip_range);

    let mut game = Game::new(traversal, params, board);

    game.train(0.35);
    let file_name = format!(
        "{}{}{}.json",
        number_to_card(board[0]),
        number_to_card(board[1]),
        number_to_card(board[2])
    );
    // game.output_results(bucket_name, file_name.as_ref()).await?;
    Ok(())
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameResult {
    pub oop_range: Vec<Combination>,
    pub ip_range: Vec<Combination>,
    pub game_params: GameParams,
    pub starting_board: Board,
    pub node_results: NodeResult,
}

pub struct Game {
    traversal: Traversal,
    pub root: Node,
    game_params: GameParams,
    starting_board: Board,
}

impl Game {
    pub fn new(traversal: Traversal, game_params: GameParams, starting_board: Board) -> Self {
        Self {
            traversal,
            game_params,
            root: OtherShowdownNode(ShowdownNode::new(0.0)),
            starting_board,
        }
    }

    pub fn train(&mut self, target_nash_distance: f32) {
        self.construct_tree();

        self.traversal.traverser = 0;

        let ip_range = self.traversal.get_range_for_opponent(&self.starting_board);
        let oop_range = self
            .traversal
            .get_range_for_active_player(&self.starting_board);

        let ip: Vec<f32> = ip_range.iter().map(|combo| combo.combos).collect();
        let oop: Vec<f32> = oop_range.iter().map(|combo| combo.combos).collect();

        let ip_relative_probs = range_relative_probabilities(ip_range, oop_range);
        let oop_relative_probs = range_relative_probabilities(oop_range, ip_range);

        let mut iterations = 0;
        loop {
            if iterations % 25 == 0 {
                self.traversal.traverser = 0;
                let oop_br = self.overall_best_response(&oop_relative_probs, &ip);
                self.traversal.traverser = 1;
                let ip_br = self.overall_best_response(&ip_relative_probs, &oop);
                let exploitability = (ip_br + oop_br) / 2.0 / self.game_params.starting_pot * 100.0;
                info!(
                    "Iteration {} OOP BR {} IP BR {} exploitability = {} percent of the pot",
                    iterations, oop_br, ip_br, exploitability
                );
                if exploitability < target_nash_distance {
                    break;
                }
            }

            self.traversal.iteration = iterations;
            self.traversal.traverser = 0;
            self.root
                .cfr_traversal(&self.traversal, &ip, &self.starting_board);
            self.traversal.traverser = 1;
            self.root
                .cfr_traversal(&self.traversal, &oop, &self.starting_board);
            iterations += 1;
        }

        info!("Reached target exploitability, persisting node EVs");
        self.traversal.persist_evs = true;
        self.traversal.traverser = 0;
        self.overall_best_response(&oop_relative_probs, &ip);
        self.traversal.traverser = 1;
        self.overall_best_response(&ip_relative_probs, &oop);
        info!("Done persisting node EVs");
    }

    fn overall_best_response(
        &mut self,
        responder_relative_probs: &[f32],
        opp_reach_probs: &[f32],
    ) -> f32 {
        let responder_hands = self
            .traversal
            .get_range_for_active_player(&self.starting_board);
        let opponent_hands = self.traversal.get_range_for_opponent(&self.starting_board);

        let unblocked = unblocked_hands(responder_hands, opponent_hands);

        let evs: Vec<f32> =
            self.root
                .best_response(&self.traversal, opp_reach_probs, &self.starting_board);

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

        self.root = OtherActionNode(root);
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
            root.add_child(OtherShowdownNode(next));
        } else if call_stacks == 0.0 {
            let next = AllInShowdownNode::new(root.pot_size + last_bet_size, street);
            root.add_child(OtherAllInShowdownNode(next));
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
                next.add_next_node(OtherActionNode(next_game_node));
            }

            root.add_child(OtherChanceNode(next));
        }

        if bet_number > 0 {
            let fold = TerminalNode::new(root.pot_size - last_bet_size, root.player_node ^ 1);
            root.add_child(OtherTerminalNode(fold));
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

        root.add_child(OtherActionNode(next));
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
            let mut v = self
                .get_current_bets(street, root.player_node, bet_number)
                .to_vec();
            v.push(self.game_params.all_in_cut_off);
            v
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
                root.add_child(OtherActionNode(next));
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
                root.add_child(OtherActionNode(next));
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

    pub async fn output_results(
        &self,
        bucket_name: &str,
        file_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Uploading file {} to bucket {}", file_name, bucket_name);
        let result = GameResult {
            oop_range: self.traversal.oop_rm.get_starting_combinations(),
            ip_range: self.traversal.ip_rm.get_starting_combinations(),
            game_params: self.game_params.clone(),
            starting_board: self.starting_board,
            node_results: self.root.output_results().unwrap(),
        };

        let bytes = serde_json::to_string(&result).unwrap().as_bytes().to_vec();

        let client = Client::default();
        client
            .object()
            .create(bucket_name, bytes, file_name, "application/json")
            .await?;
        Ok(())
    }
}
