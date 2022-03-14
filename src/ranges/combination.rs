use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub type Hand = [u8; 2];
pub type Board = [u8; 5];
pub type Range = [Combination];

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Combination {
    pub hand: Hand,
    #[serde(skip_serializing)]
    pub rank: u16,
    pub combos: f32,
    #[serde(skip_serializing)]
    pub weight: i8,
    #[serde(skip_serializing)]
    pub raw_index: usize,
    #[serde(skip_serializing)]
    pub canon_index: usize,
}

impl Combination {
    pub fn new(hand: Hand, rank: u16, combos: f32) -> Self {
        let raw_index = usize::from(hand[0]) * 52 + usize::from(hand[1]);
        Combination {
            hand,
            rank,
            combos,
            weight: 1,
            raw_index,
            canon_index: raw_index,
        }
    }
}

impl PartialEq for Combination {
    fn eq(&self, other: &Self) -> bool {
        self.hand[0] == other.hand[0] && self.hand[1] == other.hand[1]
    }
}

impl Eq for Combination {}

impl Ord for Combination {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank.cmp(&other.rank)
    }
}

impl PartialOrd for Combination {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
