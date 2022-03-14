use super::action_node::ActionNode;
use super::all_in_showdown_node::AllInShowdownNode;
use super::chance_node::ChanceNode;
use super::showdown_node::ShowdownNode;
use super::terminal_node::TerminalNode;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::cfr::traversal::Traversal;
use crate::ranges::combination::Board;

#[derive(Serialize, Deserialize, Debug)]
pub enum NodeResultType {
    Action,
    Chance,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeResult {
    pub node_type: NodeResultType,
    pub node_strategy: Option<Vec<f32>>,
    pub node_ev: Option<Vec<f32>>,
    pub next_cards: Option<Vec<u8>>,
    pub next_nodes: Vec<NodeResult>,
}

#[enum_dispatch]
pub trait CfrNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32>;
    fn best_response(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32>;
    fn output_results(&self) -> Option<NodeResult>;
}

#[enum_dispatch(CfrNode)]
pub enum Node {
    ActionNode(ActionNode),
    ChanceNode(ChanceNode),
    ShowdownNode(ShowdownNode),
    TerminalNode(TerminalNode),
    AllInShowdownNode(AllInShowdownNode),
}
