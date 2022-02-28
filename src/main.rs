mod cfr;
mod nodes;
mod ranges;

extern crate rayon;

use crate::{
    cfr::{game::Game, game_params::GameParams, traversal::Traversal},
    ranges::{
        combination::Board,
        range_manager::{DefaultRangeManager, IsomorphicRangeManager, RangeManagers},
        utility::{
            build_initial_suit_groups, card_to_number, construct_starting_range_from_string,
        },
    },
};

use futures_lite::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use serde::{Deserialize, Serialize};
use std::str;

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SolutionConfig {
    pub board: String,
    pub range: String,
    pub starting_pot: f32,
    pub starting_stack: f32,
    pub all_in_cut_off: f32,
    pub default_bets: Option<Vec<Vec<f32>>>,
    pub default_bet: f32,
    pub ip_flop_bets: Option<Vec<Vec<f32>>>,
    pub oop_flop_bets: Option<Vec<Vec<f32>>>,
    pub ip_turn_bets: Option<Vec<Vec<f32>>>,
    pub oop_turn_bets: Option<Vec<Vec<f32>>>,
    pub ip_river_bets: Option<Vec<Vec<f32>>>,
    pub oop_river_bets: Option<Vec<Vec<f32>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());

    let connection_props = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);
    let conn = Connection::connect(&addr, connection_props).await?;
    let channel = conn.create_channel().await?;
    let queue = channel
        .queue_declare(
            "sims",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    println!("Declared queue {:?}", queue);

    let mut consumer = channel
        .basic_consume(
            "sims",
            "my_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("rmq consumer connected, waiting for messages");
    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let data = str::from_utf8(&delivery.data).expect("can't convert message to string");
            let p: SolutionConfig = serde_json::from_str(data)?;

            println!("received msg: {:?}", p);

            let b: Vec<u8> = p.board.split(",").map(|x| card_to_number(x.to_string())).collect();

            let board: Board = [
                b[0],
                b[1],
                b[2],
                52,
                52
            ];

            println!("board: {:?}", board);

            run_trainer(
                board,
                &p.range,
                &p.range,
                GameParams::new(
                    1,
                    p.starting_pot,
                    p.starting_stack,
                    p.all_in_cut_off,
                    p.default_bet,
                    vec![vec![]],
                    vec![vec![0.75]],
                    vec![vec![0.75]],
                    vec![vec![0.75]],
                    vec![vec![0.75]],
                    vec![vec![0.75]],
                ),
            );
            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await?
        }
    }

    Ok(())
}

fn run_trainer(board: Board, oop_range: &str, ip_range: &str, params: GameParams) {
    println!("Params: {:?}", params);
    println!("range: {} range: {}", oop_range, ip_range);
    let starting_combinations = construct_starting_range_from_string(oop_range.to_string(), &board);
    let starting_combinations2 = construct_starting_range_from_string(ip_range.to_string(), &board);
    let sg = build_initial_suit_groups(&board);
    let mut iso = false;
    for suit in 0u8..4 {
        if sg[usize::from(suit)] != suit {
            iso = true;
        }
    }
    let rm = if iso {
        RangeManagers::from(IsomorphicRangeManager::new(starting_combinations, board))
    } else {
        RangeManagers::from(DefaultRangeManager::new(starting_combinations, board))
    };

    let rm2 = if iso {
        RangeManagers::from(IsomorphicRangeManager::new(starting_combinations2, board))
    } else {
        RangeManagers::from(DefaultRangeManager::new(starting_combinations2, board))
    };

    let trav = Traversal::new(rm, rm2);

    let mut game = Game::new(trav, params, board);

    /*
    Iteration 0 OOP BR 139.70445 IP BR 146.03627 exploitability = 238.11726 percent of the pot
    Iteration 25 OOP BR 2.500492 IP BR 7.2116675 exploitability = 8.093467 percent of the pot
    Iteration 50 OOP BR -1.7358472 IP BR 3.756436 exploitability = 1.6838242 percent of the pot

    real    0m54.853s
    user    11m19.130s
    sys     0m5.169s
     */
    game.train(75);
}
