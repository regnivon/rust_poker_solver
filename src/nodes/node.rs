use super::action_node::ActionNode;
use super::all_in_showdown_node::AllInShowdownNode;
use super::chance_node::ChanceNode;
use super::showdown_node::ShowdownNode;
use super::terminal_node::TerminalNode;

use enum_dispatch::enum_dispatch;

use crate::cfr::traversal::Traversal;
use crate::ranges::combination::Board;

#[enum_dispatch]
pub trait CfrNode {
    fn cfr_traversal(
        &mut self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32>;
    fn best_response(
        &self,
        traversal: &Traversal,
        op_reach_prob: &[f32],
        board: &Board,
    ) -> Vec<f32>;
}

#[enum_dispatch(CfrNode)]
pub enum Node {
    ActionNode,
    ChanceNode,
    ShowdownNode,
    TerminalNode,
    AllInShowdownNode,
}
