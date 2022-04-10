use tracing::info;
use crate::ranges::{
    combination::{Board, Combination},
    range_manager::{RangeManager, RangeManagers, DefaultRangeManager, IsomorphicRangeManager},
    utility::{build_initial_suit_groups, build_player_specific_merged_range, construct_starting_range_from_string},
};

pub fn build_traversal_from_ranges(board: Board, oop_range: &str, ip_range: &str) -> Traversal {
    let merged = if oop_range.eq_ignore_ascii_case("random") || ip_range.eq_ignore_ascii_case("random") {
        construct_starting_range_from_string("random".to_string(), &board)
    } else {
        construct_starting_range_from_string(format!("{},{}", oop_range, ip_range), &board)
    };
    let oop_combinations = build_player_specific_merged_range(oop_range.to_string(), &merged);
    let ip_combinations = build_player_specific_merged_range(ip_range.to_string(), &merged);

    let sg = build_initial_suit_groups(&board);
    let mut iso = false;
    for suit in 0u8..4 {
        if sg[usize::from(suit)] != suit {
            iso = true;
        }
    }
    let oop_rm = if iso {
        RangeManagers::from(IsomorphicRangeManager::new(oop_combinations, board))
    } else {
        RangeManagers::from(DefaultRangeManager::new(oop_combinations, board))
    };

    let ip_rm = if iso {
        RangeManagers::from(IsomorphicRangeManager::new(ip_combinations, board))
    } else {
        RangeManagers::from(DefaultRangeManager::new(ip_combinations, board))
    };

    Traversal::new(oop_rm, ip_rm)
}

pub struct Traversal {
    pub oop_rm: RangeManagers,
    pub ip_rm: RangeManagers,
    pub traverser: u8,
    pub iteration: u32,
    pub persist_evs: bool,
}

impl Traversal {
    pub fn new(oop_rm: RangeManagers, ip_rm: RangeManagers) -> Self {
        Self {
            oop_rm,
            ip_rm,
            traverser: 0,
            iteration: 0,
            persist_evs: false,
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
