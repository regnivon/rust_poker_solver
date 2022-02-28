use crate::ranges::{
    combination::{Board, Combination},
    range_manager::{RangeManager, RangeManagers},
};

pub struct Traversal {
    oop_rm: RangeManagers,
    ip_rm: RangeManagers,
    pub traverser: u8,
    pub iteration: u32,
}

impl Traversal {
    pub fn new(oop_rm: RangeManagers, ip_rm: RangeManagers) -> Self {
        Self {
            oop_rm,
            ip_rm,
            traverser: 0,
            iteration: 0,
        }
    }

    pub fn get_range_for_active_player(&self, board: &Board) -> &Vec<Combination> {
        if self.traverser == 1 {
            return self.ip_rm.get_range_for_board(board);
        }
        self.oop_rm.get_range_for_board(board)
    }

    pub fn get_range_for_opponent(&self, board: &Board) -> &Vec<Combination> {
        if self.traverser == 1 {
            return self.oop_rm.get_range_for_board(board);
        }
        self.ip_rm.get_range_for_board(board)
    }

    pub fn get_num_hands_for_traverser(&self, board: &Board) -> usize {
        if self.traverser == 1 {
            return self.ip_rm.get_range_for_board(board).len();
        }
        self.oop_rm.get_range_for_board(board).len()
    }

    pub fn get_num_hands_for_player(&self, player: u8, board: &Board) -> usize {
        if player == 1 {
            return self.ip_rm.get_range_for_board(board).len();
        }
        self.oop_rm.get_range_for_board(board).len()
    }

    pub fn get_next_reach_probs(&self, new_board: &Board, opp_reach_probs: &[f32]) -> Vec<f32> {
        if self.traverser == 1 {
            return self.oop_rm.get_next_reach_probs(new_board, opp_reach_probs);
        }
        self.ip_rm.get_next_reach_probs(new_board, opp_reach_probs)
    }

    pub fn map_utility_backwards(
        &self,
        new_board: &Board,
        utility: &[f32],
        mapped_utility: &mut Vec<f32>,
    ) {
        if self.traverser == 1 {
            return self
                .oop_rm
                .map_utility_backwards(new_board, utility, mapped_utility);
        }
        self.ip_rm
            .map_utility_backwards(new_board, utility, mapped_utility)
    }

    pub fn merge_canonical_utilities(&self, board: &Board, utility: &mut Vec<f32>) {
        if self.traverser == 1 {
            return self.oop_rm.merge_canonical_utilities(board, utility);
        }
        self.ip_rm.merge_canonical_utilities(board, utility)
    }
}
