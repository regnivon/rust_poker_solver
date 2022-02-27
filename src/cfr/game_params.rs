#[derive(Debug, Default)]
pub struct GameParams {
    pub parallel_street: u8,
    pub starting_pot: f32,
    pub starting_stack: f32,
    pub all_in_cut_off: f32,
    pub default_bet: f32,
    pub default_bets: Vec<Vec<f32>>,
    pub ip_flop_bets: Vec<Vec<f32>>,
    pub oop_flop_bets: Vec<Vec<f32>>,
    pub ip_turn_bets: Vec<Vec<f32>>,
    pub oop_turn_bets: Vec<Vec<f32>>,
    pub ip_river_bets: Vec<Vec<f32>>,
    pub oop_river_bets: Vec<Vec<f32>>,
}

impl GameParams {
    pub fn new(
        parallel_street: u8,
        starting_pot: f32,
        starting_stack: f32,
        all_in_cut_off: f32,
        default_bet: f32,
        oop_flop_bets: Vec<Vec<f32>>,
        oop_turn_bets: Vec<Vec<f32>>,
        oop_river_bets: Vec<Vec<f32>>,
        ip_flop_bets: Vec<Vec<f32>>,
        ip_turn_bets: Vec<Vec<f32>>,
        ip_river_bets: Vec<Vec<f32>>,
    ) -> Self {
        Self {
            parallel_street,
            starting_pot,
            starting_stack,
            all_in_cut_off,
            default_bet,
            default_bets: vec![vec![default_bet; 1]],
            ip_flop_bets,
            oop_flop_bets,
            ip_turn_bets,
            oop_turn_bets,
            ip_river_bets,
            oop_river_bets,
        }
    }
}
