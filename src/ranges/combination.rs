use std::cmp::Ordering;

pub type Hand = [u8; 2];
pub type Board = [u8; 5];
pub type Range = Vec<Combination>;

#[derive(Clone, Copy, Debug)]
pub struct Combination {
    pub hand: Hand,
    pub rank: u16,
    pub combos: f64,
}

impl Combination {
    pub fn new(hand: Hand, rank: u16, combos: f64) -> Self {
        Combination { hand, rank, combos }
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
