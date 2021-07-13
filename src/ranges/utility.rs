use rust_poker::hand_range::HandRange;
use rust_poker::hand_range::{char_to_rank, char_to_suit};

use super::combination::{Board, Combination, Hand, Range};

pub fn board_has_turn(board: &Board) -> bool {
    board[3] != 52
}

pub fn board_has_river(board: &Board) -> bool {
    board[4] != 52
}

pub fn check_card_overlap(card: u8, board: &Board) -> bool {
    board.iter().any(|&c| c == card)
}

pub fn check_hand_overlap(hand: Hand, board: &Board) -> bool {
    board.iter().any(|&c| c == hand[0] || c == hand[1])
}

pub fn check_hands_overlap(hand1: &Hand, hand2: &Hand) -> bool {
    hand1.iter().any(|&c| c == hand2[0] || c == hand2[1])
}

pub fn construct_starting_range_from_string(
    range_string: String,
    board: &Board,
) -> Vec<Combination> {
    let starting_range = HandRange::from_strings([range_string].to_vec());
    let mut starting_combinations = vec![];
    for hand in starting_range[0].hands.iter() {
        let combo = Combination::new([hand.0, hand.1], 0, 1.0);
        if !check_hand_overlap(combo.hand, board) {
            starting_combinations.push(combo)
        }
    }
    starting_combinations
}

pub fn range_relative_probabilities(rng: &Range, opp_range: &Range) -> Vec<f64> {
    let mut normalizing_value = 0.0;
    let mut relatives = vec![0.0; rng.len()];

    for i in 0..rng.len() {
        let mut probability = 0.0;

        for j in 0..opp_range.len() {
            if !check_hands_overlap(&rng[i].hand, &opp_range[j].hand) {
                probability += opp_range[j].combos;
            }
        }
        relatives[i] = probability * rng[i].combos;
        normalizing_value += relatives[i];
    }

    for item in relatives.iter_mut() {
        *item /= normalizing_value;
    }
    relatives
}

pub fn unblocked_hands(rng: &Range, opp_range: &Range) -> Vec<f64> {
    let mut hand_counts = vec![0.0; rng.len()];

    for i in 0..rng.len() {
        let mut counts = 0.0;

        for j in 0..opp_range.len() {
            if !check_hands_overlap(&rng[i].hand, &opp_range[j].hand) {
                counts += opp_range[j].combos;
            }
        }
        hand_counts[i] = counts;
    }

    hand_counts
}

pub fn card_to_number(card: String) -> u8 {
    let chars: Vec<char> = card.chars().collect();
    let rank = char_to_rank(chars[0]);
    let suit = char_to_suit(chars[1]);

    4 * rank + suit
}
