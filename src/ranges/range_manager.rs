use std::collections::HashMap;

use enum_dispatch::enum_dispatch;
use rust_poker::hand_evaluator::{evaluate, Hand, CARDS};
use rust_poker::HandIndexer;

use super::{
    combination::{Board, Combination},
    utility::{board_has_river, board_has_turn, check_card_overlap, check_hand_overlap},
};

#[inline(always)]
fn get_key(board: &Board) -> u64 {
    if board[3] == 52 {
        100000000 * u64::from(board[0] + 1)
            + 1000000 * u64::from(board[1] + 1)
            + 10000 * u64::from(board[2] + 1)
    } else if board[4] == 52 {
        100000000 * u64::from(board[0] + 1)
            + 1000000 * u64::from(board[1] + 1)
            + 10000 * u64::from(board[2] + 1)
            + 100 * u64::from(board[3] + 1)
    } else {
        100000000 * u64::from(board[0] + 1)
            + 1000000 * u64::from(board[1] + 1)
            + 10000 * u64::from(board[2] + 1)
            + 100 * u64::from(board[3] + 1)
            + u64::from(board[4] + 1)
    }
}

#[enum_dispatch]
pub trait RangeManager {
    fn merge_canonical_utilities(&self, board: &Board, utility: &mut Vec<f32>);
    fn map_utility_backwards(
        &self,
        new_board: &Board,
        utility: &[f32],
        mapped_utility: &mut Vec<f32>,
    );
    fn get_next_reach_probs(&self, new_board: &Board, opp_reach_probs: &[f32]) -> Vec<f32>;
    fn get_range_for_board(&self, board: &Board) -> &Vec<Combination>;
    fn get_reach_probs_mapping(&self, board: &Board) -> &Vec<usize>;
    fn get_starting_combinations(&self) -> Vec<Combination>;
}

#[enum_dispatch(RangeManager)]
pub enum RangeManagers {
    IsomorphicRangeManager,
    DefaultRangeManager,
}

pub struct IsomorphicRangeManager {
    starting_combinations: Vec<Combination>,
    ranges: HashMap<u64, Vec<Combination>>,
    reach_probs_mapping: HashMap<u64, Vec<usize>>,
    flop_indexer: HandIndexer,
    turn_indexer: HandIndexer,
    river_indexer: HandIndexer,
}

// TODO: Track where my hand is in opponents range for terminal eval
impl IsomorphicRangeManager {
    pub fn new(starting_combinations: Vec<Combination>, initial_board: Board) -> Self {
        let mut rm = IsomorphicRangeManager {
            starting_combinations,
            ranges: HashMap::new(),
            reach_probs_mapping: HashMap::new(),
            flop_indexer: HandIndexer::init(2, [2, 3].to_vec()),
            turn_indexer: HandIndexer::init(3, [2, 3, 1].to_vec()),
            river_indexer: HandIndexer::init(4, [2, 3, 1, 1].to_vec()),
        };

        rm.init(&initial_board);
        rm
    }

    fn init(&mut self, initial_board: &Board) {
        self.ranges
            .insert(get_key(initial_board), self.starting_combinations.to_vec());

        self.initialize_ranges(initial_board);
    }

    fn initialize_ranges(&mut self, initial_board: &Board) {
        // flop
        if !board_has_turn(initial_board) {
            self.init_ranges_from_flop(initial_board)
        }
        //turn
        else if !board_has_river(initial_board) {
            self.init_ranges_from_turn(initial_board)
        }
        // river
        else {
            self.init_ranges_from_river(initial_board)
        }
    }

    fn init_ranges_from_flop(&mut self, initial_board: &Board) {
        let mut canon_index_to_range_index = HashMap::new();
        let mut flop_board_hand = [0, 0, initial_board[0], initial_board[1], initial_board[2]];

        let flop_board_key = get_key(initial_board);

        let mut flop_hands: Vec<Combination> = vec![];
        let mut index_count: usize = 0;
        for hand in self.starting_combinations.iter() {
            flop_board_hand[0] = hand.hand[0];
            flop_board_hand[1] = hand.hand[1];

            let hand_index = self.flop_indexer.get_index(&flop_board_hand);
            if !canon_index_to_range_index.contains_key(&hand_index) {
                canon_index_to_range_index.insert(hand_index, index_count);
                flop_hands.push(*hand);
            } else {
                let canon_location = canon_index_to_range_index[&hand_index];
                flop_hands[canon_location].weight += 1;
                let mut combo = *hand;
                combo.canon_index = flop_hands[canon_location].raw_index;
                combo.weight = 0;
                flop_hands.push(combo);
            }
            index_count += 1;
        }

        let mut flop_hand_mapping = vec![0; 51 * 52 + 51];

        for (i, hand) in flop_hands.iter().enumerate() {
            if hand.weight != 0 {
                flop_hand_mapping[hand.raw_index] = i;
            }
        }

        for hand in flop_hands.iter() {
            if hand.weight == 0 {
                flop_hand_mapping[hand.raw_index] = flop_hand_mapping[hand.canon_index];
            }
        }

        self.add_range_for_board(flop_hands, flop_board_key);
        self.reach_probs_mapping
            .insert(flop_board_key, flop_hand_mapping);

        for turn in 0u8..52 {
            if check_card_overlap(turn, initial_board) {
                continue;
            }

            let mut turn_board = *initial_board;

            turn_board[3] = turn;

            let mut turn_board_hand = [
                0,
                0,
                initial_board[0],
                initial_board[1],
                initial_board[2],
                turn,
            ];

            let turn_board_key = get_key(&turn_board);

            let mut turn_hands: Vec<Combination> = vec![];

            index_count = 0;
            canon_index_to_range_index.clear();
            for hand in self.starting_combinations.iter() {
                if check_hand_overlap(hand.hand, &turn_board) {
                    continue;
                }

                turn_board_hand[0] = hand.hand[0];
                turn_board_hand[1] = hand.hand[1];

                let hand_index = self.turn_indexer.get_index(&turn_board_hand);
                if !canon_index_to_range_index.contains_key(&hand_index) {
                    canon_index_to_range_index.insert(hand_index, index_count);
                    turn_hands.push(hand.clone());
                } else {
                    let canon_location = canon_index_to_range_index[&hand_index];
                    turn_hands[canon_location].weight += 1;
                    let mut combo = hand.clone();
                    combo.canon_index = turn_hands[canon_location].raw_index;
                    combo.weight = 0;
                    turn_hands.push(combo);
                }
                index_count += 1;
            }

            let mut turn_reach_probs_mapping = vec![0; 51 * 52 + 51];

            for (i, hand) in turn_hands.iter().enumerate() {
                if hand.weight != 0 {
                    turn_reach_probs_mapping[hand.raw_index] = i;
                }
            }

            for hand in turn_hands.iter() {
                if hand.weight == 0 {
                    turn_reach_probs_mapping[hand.raw_index] =
                        turn_reach_probs_mapping[hand.canon_index];
                }
            }

            self.reach_probs_mapping
                .insert(turn_board_key, turn_reach_probs_mapping);

            for river in 0..52 {
                if check_card_overlap(river, &turn_board) {
                    continue;
                }

                let mut river_board = turn_board;
                river_board[4] = river;

                let river_board_key = get_key(&river_board);

                let mut river_hands: Vec<Combination> = vec![];

                let mut river_board_hand = [
                    0,
                    0,
                    initial_board[0],
                    initial_board[1],
                    initial_board[2],
                    turn,
                    river,
                ];
                let mut board_hand = Hand::default();
                for board_card in river_board.iter() {
                    board_hand += CARDS[usize::from(*board_card)];
                }

                index_count = 0;
                canon_index_to_range_index.clear();
                for hand in turn_hands.iter() {
                    if check_hand_overlap(hand.hand, &river_board) {
                        continue;
                    }

                    river_board_hand[0] = hand.hand[0];
                    river_board_hand[1] = hand.hand[1];

                    self.river_indexer.get_index(&river_board_hand);

                    let river_hand = board_hand
                        + CARDS[usize::from(hand.hand[0])]
                        + CARDS[usize::from(hand.hand[1])];

                    let mut combo = Combination::new(hand.hand, evaluate(&river_hand), hand.combos);

                    let hand_index = self.river_indexer.get_index(&river_board_hand);
                    if !canon_index_to_range_index.contains_key(&hand_index) {
                        canon_index_to_range_index.insert(hand_index, index_count);
                        river_hands.push(combo);
                    } else {
                        let canon_location = canon_index_to_range_index[&hand_index];
                        river_hands[canon_location].weight += 1;
                        combo.canon_index = river_hands[canon_location].raw_index;
                        combo.weight = 0;
                        river_hands.push(combo);
                    }
                    index_count += 1;
                }

                // do forward reach probs mapping to this river card, then quick sort the mapping and hands together
                // so that we can map forward correctly, allowing for easy O(N) showdown eval

                let mut river_reach_probs_mapping = vec![0; river_hands.len()];

                let mut j = 0;
                for i in 0..river_hands.len() {
                    while river_hands[i] != turn_hands[j] {
                        j += 1;
                    }

                    river_reach_probs_mapping[i] = j;
                }

                let permute = permutation::sort_by(&river_hands[..], |a, b| a.rank.cmp(&b.rank));
                let river_reach_probs_mapping = permute.apply_slice(&river_reach_probs_mapping[..]);
                let river_hands = permute.apply_slice(&river_hands[..]);

                self.reach_probs_mapping
                    .insert(river_board_key, river_reach_probs_mapping);

                self.add_range_for_board(river_hands, river_board_key);
            }

            self.add_range_for_board(turn_hands, turn_board_key);
        }
    }

    fn init_ranges_from_turn(&mut self, initial_board: &Board) {
        for river in 0..52 {
            if check_card_overlap(river, initial_board) {
                continue;
            }

            let mut river_board = *initial_board;
            river_board[4] = river;

            let river_board_key = get_key(&river_board);

            let mut river_hands: Vec<Combination> = vec![];

            let mut board_hand = Hand::default();
            for board_card in river_board.iter() {
                board_hand += CARDS[usize::from(*board_card)];
            }

            for hand in self.starting_combinations.iter() {
                if check_hand_overlap(hand.hand, &river_board) {
                    continue;
                }

                let river_hand = board_hand
                    + CARDS[usize::from(hand.hand[0])]
                    + CARDS[usize::from(hand.hand[1])];

                river_hands.push(Combination::new(
                    hand.hand,
                    evaluate(&river_hand),
                    hand.combos,
                ));
            }

            let mut river_reach_probs_mapping = vec![0; river_hands.len()];

            let mut j = 0;
            for i in 0..river_hands.len() {
                while river_hands[i] != self.starting_combinations[j] {
                    j += 1;
                }

                river_reach_probs_mapping[i] = j;
            }

            let permute = permutation::sort_by(&river_hands[..], |a, b| a.rank.cmp(&b.rank));
            let river_reach_probs_mapping = permute.apply_slice(&river_reach_probs_mapping[..]);
            let river_hands = permute.apply_slice(&river_hands[..]);

            self.reach_probs_mapping
                .insert(river_board_key, river_reach_probs_mapping);

            self.add_range_for_board(river_hands, river_board_key);
        }
    }

    fn init_ranges_from_river(&mut self, initial_board: &Board) {
        let river_board_key = get_key(initial_board);

        let mut river_hands: Vec<Combination> = vec![];

        let mut board_hand = Hand::default();
        for board_card in initial_board.iter() {
            board_hand += CARDS[usize::from(*board_card)];
        }

        for hand in self.starting_combinations.iter() {
            if check_hand_overlap(hand.hand, initial_board) {
                continue;
            }

            let river_hand =
                board_hand + CARDS[usize::from(hand.hand[0])] + CARDS[usize::from(hand.hand[1])];

            river_hands.push(Combination::new(
                hand.hand,
                evaluate(&river_hand),
                hand.combos,
            ));
        }

        river_hands.sort_by_key(|k| k.rank);

        self.add_range_for_board(river_hands, river_board_key);
    }

    fn add_range_for_board(&mut self, range: Vec<Combination>, board_key: u64) {
        self.ranges.insert(board_key, range);
    }
}

impl RangeManager for IsomorphicRangeManager {
    fn merge_canonical_utilities(&self, board: &Board, utility: &mut Vec<f32>) {
        let board_key = get_key(board);
        let mapping = &self.reach_probs_mapping[&board_key];
        let hands = &self.ranges[&board_key];

        for i in 0..hands.len() {
            if hands[i].weight == 0 {
                utility[i] = utility[mapping[hands[i].canon_index]];
            }
        }
    }

    fn map_utility_backwards(
        &self,
        new_board: &Board,
        utility: &[f32],
        mapped_utility: &mut Vec<f32>,
    ) {
        let board_key = get_key(new_board);
        let next_hands = &self.ranges[&board_key];

        let mut last_board = *new_board;
        if last_board[4] == 52 {
            last_board[3] = 52;
        } else {
            last_board[4] = 52;
        }
        let last_board_key = get_key(&last_board);
        let map = &self.reach_probs_mapping[&last_board_key];

        utility
            .iter()
            .zip(next_hands.iter())
            .for_each(|(util, next_hand)| {
                mapped_utility[map[next_hand.raw_index]] += util;
            });
    }

    fn get_next_reach_probs(&self, new_board: &Board, opp_reach_probs: &[f32]) -> Vec<f32> {
        let board_key = get_key(new_board);
        let next_hands = &self.ranges[&board_key];

        let mut last_board = *new_board;
        if last_board[4] == 52 {
            last_board[3] = 52;
        } else {
            last_board[4] = 52;
        }
        let last_board_key = get_key(&last_board);
        let map = &self.reach_probs_mapping[&last_board_key];

        let mut new_reach_probs = vec![0.0; next_hands.len()];

        new_reach_probs
            .iter_mut()
            .zip(next_hands.iter())
            .for_each(|(new_reach, next_hand)| {
                *new_reach = opp_reach_probs[map[next_hand.raw_index]];
            });

        new_reach_probs
    }

    fn get_range_for_board(&self, board: &Board) -> &Vec<Combination> {
        let board_key = get_key(board);
        self.ranges.get(&board_key).unwrap()
    }

    fn get_reach_probs_mapping(&self, board: &Board) -> &Vec<usize> {
        &self.reach_probs_mapping[&get_key(board)]
    }

    fn get_starting_combinations(&self) -> Vec<Combination> {
        self.starting_combinations.clone()
    }
}

#[derive(Debug, Default)]
pub struct DefaultRangeManager {
    starting_combinations: Vec<Combination>,
    ranges: HashMap<u64, Vec<Combination>>,
    reach_probs_mapping: HashMap<u64, Vec<usize>>,
}

// TODO: Track where my hand is in opponents range for terminal eval
impl DefaultRangeManager {
    pub fn new(starting_combinations: Vec<Combination>, initial_board: Board) -> Self {
        let mut rm = DefaultRangeManager {
            starting_combinations,
            ranges: HashMap::new(),
            reach_probs_mapping: HashMap::new(),
        };

        rm.init(&initial_board);
        rm
    }

    fn init(&mut self, initial_board: &Board) {
        self.ranges
            .insert(get_key(initial_board), self.starting_combinations.to_vec());

        self.initialize_ranges(initial_board);
    }

    fn initialize_ranges(&mut self, initial_board: &Board) {
        // flop
        if !board_has_turn(initial_board) {
            self.init_ranges_from_flop(initial_board)
        }
        //turn
        else if !board_has_river(initial_board) {
            self.init_ranges_from_turn(initial_board)
        }
        // river
        else {
            self.init_ranges_from_river(initial_board)
        }
    }

    fn init_ranges_from_flop(&mut self, initial_board: &Board) {
        for turn in 0u8..52 {
            if check_card_overlap(turn, initial_board) {
                continue;
            }

            let mut turn_board = *initial_board;

            turn_board[3] = turn;

            let turn_board_key = get_key(&turn_board);

            let mut turn_hands: Vec<Combination> = vec![];

            for hand in self.starting_combinations.iter() {
                if check_hand_overlap(hand.hand, &turn_board) {
                    continue;
                }

                turn_hands.push(Combination::new(hand.hand, 0, hand.combos));
            }

            let mut turn_reach_probs_mapping = vec![0; turn_hands.len()];

            // do forward reach probs mapping to this turn card
            let mut j = 0;
            for i in 0..turn_hands.len() {
                while turn_hands[i] != self.starting_combinations[j] {
                    j += 1;
                }

                turn_reach_probs_mapping[i] = j;
            }

            self.reach_probs_mapping
                .insert(turn_board_key, turn_reach_probs_mapping);

            for river in 0..52 {
                if check_card_overlap(river, &turn_board) {
                    continue;
                }

                let mut river_board = turn_board;
                river_board[4] = river;

                let river_board_key = get_key(&river_board);

                let mut river_hands: Vec<Combination> = vec![];

                let mut board_hand = Hand::default();
                for board_card in river_board.iter() {
                    board_hand += CARDS[usize::from(*board_card)];
                }

                for hand in turn_hands.iter() {
                    if check_hand_overlap(hand.hand, &river_board) {
                        continue;
                    }

                    let river_hand = board_hand
                        + CARDS[usize::from(hand.hand[0])]
                        + CARDS[usize::from(hand.hand[1])];

                    river_hands.push(Combination::new(
                        hand.hand,
                        evaluate(&river_hand),
                        hand.combos,
                    ));
                }

                // do forward reach probs mapping to this river card, then quick sort the mapping and hands together
                // so that we can map forward correctly, allowing for easy O(N) showdown eval

                let mut river_reach_probs_mapping = vec![0; river_hands.len()];

                let mut j = 0;
                for i in 0..river_hands.len() {
                    while river_hands[i] != turn_hands[j] {
                        j += 1;
                    }

                    river_reach_probs_mapping[i] = j;
                }

                let permute = permutation::sort_by(&river_hands[..], |a, b| a.rank.cmp(&b.rank));
                let river_reach_probs_mapping = permute.apply_slice(&river_reach_probs_mapping[..]);
                let river_hands = permute.apply_slice(&river_hands[..]);

                self.reach_probs_mapping
                    .insert(river_board_key, river_reach_probs_mapping);

                self.add_range_for_board(river_hands, river_board_key);
            }

            self.add_range_for_board(turn_hands, turn_board_key);
        }
    }

    fn init_ranges_from_turn(&mut self, initial_board: &Board) {
        for river in 0..52 {
            if check_card_overlap(river, initial_board) {
                continue;
            }

            let mut river_board = *initial_board;
            river_board[4] = river;

            let river_board_key = get_key(&river_board);

            let mut river_hands: Vec<Combination> = vec![];

            let mut board_hand = Hand::default();
            for board_card in river_board.iter() {
                board_hand += CARDS[usize::from(*board_card)];
            }

            for hand in self.starting_combinations.iter() {
                if check_hand_overlap(hand.hand, &river_board) {
                    continue;
                }

                let river_hand = board_hand
                    + CARDS[usize::from(hand.hand[0])]
                    + CARDS[usize::from(hand.hand[1])];

                river_hands.push(Combination::new(
                    hand.hand,
                    evaluate(&river_hand),
                    hand.combos,
                ));
            }

            let mut river_reach_probs_mapping = vec![0; river_hands.len()];

            let mut j = 0;
            for i in 0..river_hands.len() {
                while river_hands[i] != self.starting_combinations[j] {
                    j += 1;
                }

                river_reach_probs_mapping[i] = j;
            }

            let permute = permutation::sort_by(&river_hands[..], |a, b| a.rank.cmp(&b.rank));
            let river_reach_probs_mapping = permute.apply_slice(&river_reach_probs_mapping[..]);
            let river_hands = permute.apply_slice(&river_hands[..]);

            self.reach_probs_mapping
                .insert(river_board_key, river_reach_probs_mapping);

            self.add_range_for_board(river_hands, river_board_key);
        }
    }

    fn init_ranges_from_river(&mut self, initial_board: &Board) {
        let river_board_key = get_key(initial_board);

        let mut river_hands: Vec<Combination> = vec![];

        let mut board_hand = Hand::default();
        for board_card in initial_board.iter() {
            board_hand += CARDS[usize::from(*board_card)];
        }

        for hand in self.starting_combinations.iter() {
            if check_hand_overlap(hand.hand, initial_board) {
                continue;
            }

            let river_hand =
                board_hand + CARDS[usize::from(hand.hand[0])] + CARDS[usize::from(hand.hand[1])];

            river_hands.push(Combination::new(
                hand.hand,
                evaluate(&river_hand),
                hand.combos,
            ));
        }

        river_hands.sort_by_key(|k| k.rank);

        self.add_range_for_board(river_hands, river_board_key);
    }

    fn add_range_for_board(&mut self, range: Vec<Combination>, board_key: u64) {
        self.ranges.insert(board_key, range);
    }
}

impl RangeManager for DefaultRangeManager {
    fn merge_canonical_utilities(&self, _board: &Board, _utility: &mut Vec<f32>) {
        // noop
    }

    fn map_utility_backwards(
        &self,
        new_board: &Board,
        utility: &[f32],
        mapped_utility: &mut Vec<f32>,
    ) {
        let board_key = get_key(new_board);
        let map = &self.reach_probs_mapping[&board_key];

        utility.iter().zip(map.iter()).for_each(|(util, map_idx)| {
            mapped_utility[*map_idx] += util;
        });
    }

    fn get_next_reach_probs(&self, new_board: &Board, opp_reach_probs: &[f32]) -> Vec<f32> {
        let board_key = get_key(new_board);
        let map = &self.reach_probs_mapping[&board_key];

        let mut new_reach_probs = vec![0.0; map.len()];

        new_reach_probs
            .iter_mut()
            .zip(map.iter())
            .for_each(|(new_reach, map_idx)| {
                *new_reach = opp_reach_probs[*map_idx];
            });

        new_reach_probs
    }

    fn get_range_for_board(&self, board: &Board) -> &Vec<Combination> {
        let board_key = get_key(board);
        self.ranges.get(&board_key).unwrap()
    }

    fn get_reach_probs_mapping(&self, board: &Board) -> &Vec<usize> {
        &self.reach_probs_mapping[&get_key(board)]
    }

    fn get_starting_combinations(&self) -> Vec<Combination> {
        self.starting_combinations.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::ranges::utility::construct_starting_range_from_string;

    use super::*;

    #[test]
    fn test_get_key1() {
        let board: Board = [2, 6, 20, 52, 52];
        assert_eq!(get_key(&board), 307210000);
    }

    #[test]
    fn test_get_key2() {
        let board: Board = [2, 6, 20, 12, 52];
        assert_eq!(get_key(&board), 307211300);
    }

    #[test]
    fn test_get_key3() {
        let board: Board = [2, 6, 20, 12, 40];
        assert_eq!(get_key(&board), 307211341);
    }

    /*
    #[test]
    fn test_rm_from_river() {
        let board: Board = [2, 6, 20, 12, 40];
        let starting_combinations =
            construct_starting_range_from_string("random".to_string(), &board);

        let rm = RangeManager::new(starting_combinations, board);
        assert_eq!(rm.starting_combinations.len(), 1081);
        assert_eq!(rm.ranges.len(), 1);

        let idx = get_key(&board);

        let hands = rm.ranges.get(&idx).unwrap();

        for i in 1..hands.len() {
            assert!(hands[i - 1].rank <= hands[i].rank);
        }
    }

    #[test]
    fn test_rm_from_turn() {
        let board: Board = [2, 6, 20, 12, 52];
        let _board_key = get_key(&board);
        let starting_combinations =
            construct_starting_range_from_string("random".to_string(), &board);

        let rm = RangeManager::new(starting_combinations, board);
        // TODO: check this
        assert_eq!(rm.starting_combinations.len(), 1128);
        assert_eq!(rm.ranges.len(), 49);

        for river in 0..52 {
            if !board.contains(&river) {
                let river_board: Board = [2, 6, 20, 12, river];
                let idx = get_key(&river_board);

                let hands = rm.ranges.get(&idx).unwrap();

                for i in 1..hands.len() {
                    assert!(hands[i - 1].rank <= hands[i].rank);
                }
            }
        }

        let turn_hands = rm.get_range_for_board(&board);
        for river in 0..52 {
            if !board.contains(&river) {
                let river_board: Board = [2, 6, 20, 12, river];
                let river_key = get_key(&river_board);
                let river_mapping = rm.get_mapping_for_board(river_key);
                let river_hands = rm.get_range_for_board(&river_board);

                assert_eq!(river_hands.len(), river_mapping.len());

                for (idx, value) in river_mapping.iter().enumerate() {
                    assert_eq!(river_hands[idx], turn_hands[*value]);
                }
            }
        }
    }

    #[test]
    fn test_rm_from_flop() {
        let board: Board = [2, 6, 20, 52, 52];
        let starting_combinations =
            construct_starting_range_from_string("random".to_string(), &board);

        let rm = RangeManager::new(starting_combinations, board);
        // TODO: check this
        assert_eq!(rm.starting_combinations.len(), 1176);
        assert_eq!(rm.ranges.len(), 2402);

        // ensure that the rankings are correct
        for turn in 0..52 {
            if !board.contains(&turn) {
                for river in turn + 1..52 {
                    if !board.contains(&river) {
                        let river_board: Board = [2, 6, 20, turn, river];
                        let idx = get_key(&river_board);

                        let hands = rm.ranges.get(&idx).unwrap();

                        for i in 1..hands.len() {
                            assert!(hands[i - 1].rank <= hands[i].rank);
                        }
                    }
                }
            }
        }

        // ensure that the forward mapping is correct
        let flop_hands = rm.get_range_for_board(&board);
        for turn in 0..52 {
            if !board.contains(&turn) {
                let turn_board: Board = [2, 6, 20, turn, 52];
                let turn_key = get_key(&turn_board);
                let turn_hands = rm.get_range_for_board(&turn_board);
                let turn_mapping = rm.get_mapping_for_board(turn_key);

                assert_eq!(turn_hands.len(), turn_mapping.len());

                for (idx, value) in turn_mapping.iter().enumerate() {
                    assert_eq!(turn_hands[idx], flop_hands[*value]);
                }

                for river in turn + 1..52 {
                    if !board.contains(&river) {
                        let river_board: Board = [2, 6, 20, turn, river];
                        let river_key = get_key(&river_board);
                        let river_mapping = rm.get_mapping_for_board(river_key);
                        let river_hands = rm.get_range_for_board(&river_board);

                        assert_eq!(river_hands.len(), river_mapping.len());

                        for (idx, value) in river_mapping.iter().enumerate() {
                            assert_eq!(river_hands[idx], turn_hands[*value]);
                        }
                    }
                }
            }
        }
    }
    */
}
