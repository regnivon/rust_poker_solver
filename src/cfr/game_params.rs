pub struct GameParams {
    pub parallel_street: u8,
    pub starting_pot: f64,
    pub starting_stack: f64,
    pub all_in_cut_off: f64,
    pub default_bet: f64,
    pub default_bets: Vec<Vec<f64>>,
    pub ip_flop_bets: Vec<Vec<f64>>,
    pub oop_flop_bets: Vec<Vec<f64>>,
    pub ip_turn_bets: Vec<Vec<f64>>,
    pub oop_turn_bets: Vec<Vec<f64>>,
    pub ip_river_bets: Vec<Vec<f64>>,
    pub oop_river_bets: Vec<Vec<f64>>,
}

impl GameParams {
    pub fn new(
        parallel_street: u8,
        starting_pot: f64,
        starting_stack: f64,
        all_in_cut_off: f64,
        default_bet: f64,
        ip_flop_bets: Vec<Vec<f64>>,
        oop_flop_bets: Vec<Vec<f64>>,
        ip_turn_bets: Vec<Vec<f64>>,
        oop_turn_bets: Vec<Vec<f64>>,
        ip_river_bets: Vec<Vec<f64>>,
        oop_river_bets: Vec<Vec<f64>>,
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
