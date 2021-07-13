use crate::{cfr::traversal::Traversal, ranges::combination::Board};

use super::node::CfrNode;

pub struct AllInShowdownNode {}

impl CfrNode for AllInShowdownNode {
    fn cfr_traversal(
        &mut self,
        _traversal: &Traversal,
        _op_reach_prob: &Vec<f64>,
        _board: &Board,
    ) -> Vec<f64> {
        todo!("Not Implemented")
    }

    fn best_response(
        &self,
        _traversal: &Traversal,
        _op_reach_prob: &Vec<f64>,
        _board: &Board,
    ) -> Vec<f64> {
        todo!("Not Implemented")
    }
}
