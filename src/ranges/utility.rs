use futures_lite::StreamExt;
use rust_poker::constants::{RANK_TO_CHAR, SUIT_TO_CHAR};
use rust_poker::hand_range::HandRange;
use rust_poker::hand_range::{char_to_rank, char_to_suit};

use super::combination::{Board, Combination, Hand, Range};

pub fn build_initial_suit_groups(board: &Board) -> Vec<u8> {
    let mut ranks_used = vec![0u16; 4];
    let mut suit_groups = vec![0; 4];

    for card in board.iter() {
        if *card != 52 {
            ranks_used[usize::from(get_suit(*card))] |= 1 << get_rank(*card);
        }
    }

    for i in 0u8..4 {
        let mut j: u8 = 0;
        while j < i {
            if ranks_used[usize::from(j)] == ranks_used[usize::from(i)] {
                break;
            }
            j += 1;
        }
        suit_groups[usize::from(i)] = j;
    }

    suit_groups
}

pub fn build_next_suit_groups(board: &Board, prior_groups: &Vec<u8>) -> Vec<u8> {
    let mut ranks_used = vec![0u16; 4];
    let mut suit_groups = vec![0; 4];

    for card in board.iter() {
        if *card != 52 {
            ranks_used[usize::from(get_suit(*card))] |= 1 << get_rank(*card);
        }
    }

    for i in 0u8..4 {
        let mut j: u8 = 0;
        while j < i {
            if ranks_used[usize::from(j)] == ranks_used[usize::from(i)]
                && prior_groups[usize::from(j)] == prior_groups[usize::from(i)]
            {
                break;
            }
            j += 1;
        }
        suit_groups[usize::from(i)] = j;
    }

    suit_groups
}

pub fn get_suit(card: u8) -> u8 {
    card & 3
}

pub fn get_rank(card: u8) -> u8 {
    card >> 2
}

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
        let combo = Combination::new([hand.0, hand.1], 0, 0.0);
        if !check_hand_overlap(combo.hand, board) {
            starting_combinations.push(combo)
        }
    }

    starting_combinations
}

// currently invariant is held that ip_hands[i] == oop_hands[i], need to test if this is faster than
// fewer hands w/ maintaining reference into opponent hands for where equivalent hand is (bad locality?)
pub fn build_player_specific_merged_range(
    range_string: String,
    merged_range: &Vec<Combination>,
) -> Vec<Combination> {
    let starting_range = HandRange::from_strings([range_string].to_vec());
    let mut starting_combinations = vec![];
    let mut final_range = vec![];
    for hand in starting_range[0].hands.iter() {
        let combo = Combination::new([hand.0, hand.1], 0, f32::from(hand.2) / 100.0);
        starting_combinations.push(combo)
    }

    for &hand in merged_range.iter() {
        let to_add = match starting_combinations.iter().find(|&h| hand.eq(h)) {
            None => hand,
            Some(&matching_hand) => matching_hand
        };
        final_range.push(to_add)
    }

    final_range
}

pub fn range_relative_probabilities(rng: &Range, opp_range: &Range) -> Vec<f32> {
    let mut normalizing_value = 0.0;
    let mut relatives = vec![0.0; rng.len()];

    for i in 0..rng.len() {
        let mut probability = 0.0;

        for item in opp_range {
            if !check_hands_overlap(&rng[i].hand, &item.hand) {
                probability += item.combos;
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

pub fn unblocked_hands(rng: &Range, opp_range: &Range) -> Vec<f32> {
    let mut hand_counts = vec![0.0; rng.len()];

    for i in 0..rng.len() {
        let mut counts = 0.0;

        for item in opp_range {
            if !check_hands_overlap(&rng[i].hand, &item.hand) {
                counts += item.combos;
            }
        }
        hand_counts[i] = counts;
    }

    hand_counts
}

pub fn hand_to_string(h: &Hand) -> String {
    format!("{}{}", number_to_card(h[0]), number_to_card(h[1]))
}

pub fn card_to_number(card: String) -> u8 {
    let chars: Vec<char> = card.chars().collect();
    let rank = char_to_rank(chars[0]);
    let suit = char_to_suit(chars[1]);

    4 * rank + suit
}

pub fn number_to_card(card: u8) -> String {
    let rank = card >> 2;
    let suit = card & 3;

    let mut card_str = String::new();
    card_str.push(RANK_TO_CHAR[usize::from(rank)]);
    card_str.push(SUIT_TO_CHAR[usize::from(suit)]);

    card_str
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_suit_groups() {
        let board: Board = [
            card_to_number("7c".to_string()),
            card_to_number("7h".to_string()),
            card_to_number("7d".to_string()),
            52, //card_to_number("3d".to_string()),
            52, //card_to_number("2c".to_string()),
        ];
        let sg = build_initial_suit_groups(&board);

        println!("{:?}", sg);
    }
}
