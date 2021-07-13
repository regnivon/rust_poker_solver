use crate::ranges::{
    combination::{Board, Combination},
    range_manager::RangeManager,
};

pub struct Traversal {
    oop_rm: RangeManager,
    ip_rm: RangeManager,
    pub traverser: u8,
    pub iteration: u32,
}

impl Traversal {
    pub fn new(oop_rm: RangeManager, ip_rm: RangeManager) -> Self {
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

    pub fn get_next_reach_probs(&self, new_board: &Board, opp_reach_probs: &Vec<f64>) -> Vec<f64> {
        if self.traverser == 1 {
            return self.oop_rm.get_next_reach_probs(new_board, opp_reach_probs);
        }
        self.ip_rm.get_next_reach_probs(new_board, opp_reach_probs)
    }

    pub fn map_utility_backwards(&self, new_board: &Board, utility: &Vec<f64>, mapped_utility: &Vec<f64>) {
        
    }
}
