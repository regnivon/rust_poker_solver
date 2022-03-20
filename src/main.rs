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

use futures_lite::StreamExt;
use crate::cfr::traversal::build_traversal_from_ranges;
use crate::ranges::utility::number_to_card;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use serde::{Deserialize, Serialize};
use std::str;

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SolutionConfig {
    pub bucket_name: String,
    pub board: String,
    pub oop_range: String,
    pub ip_range: String,
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
    let board: Board = [
        card_to_number("kc".to_string()),
        card_to_number("7h".to_string()),
        card_to_number("2d".to_string()),
        52, //card_to_number("3d".to_string()),
        52, //card_to_number("2c".to_string()),
    ];

    let params = GameParams::new(
        1,
        60.0,
        1000.0,
        20.0,
        0.75,
        vec![vec![]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.33]],
        vec![vec![0.75, 1.5]],
        vec![vec![0.75, 1.5]],
    );
    // AKs@0,AQs@0,AJs@0,ATs@0,A9s@0,A8s@0,A7s@0,A6s@0,A5s@0,A4s@0,A3s@0,A2s@0
//AA,KK,QQ,JJ,TT,99,88,77,66,55,44,33,22,A2s+,K2s+,Q2s+,JTs,J9s,J8s,J7s,T9s,T8s,T7s,T6s,98s,97s,96s,87s,86s,76s,65s,A5o+,KTo+,QTo+
    //let oop = "22+,A2s+,K2s+,Q2s+,J6s+,T6s+,98s,97s,96s,87s,86s,85s,76s,75s,65s,64s,54s,A5o+,K9o+,Q9o+,JTo,J9o,T9o";
    let oop = "77,66,55,44,33,22,A7s,A6s,K8s,K5s,K4s,K3s,K2s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,J6s,J5s,J4s,J3s,J2s,T5s,T4s,T3s,T2s,96s,95s,85s,84s,74s,73s,63s,53s,52s,43s,42s,32s,A9o,A8o,A7o,A6o,A5o,A4o,A3o,KTo,K9o,K8o,K7o,K6o,QTo,Q9o,Q8o,JTo,J9o,J8o,T9o,T8o,98o,87o,76o,65o,A9s@75,A8s@75,A2s@75,K7s@75,K6s@75,Q8s@75,T7s@75,T6s@75,97s@75,86s@75,75s@75,64s@75,QJo@75,88@50,ATs@50,A3s@50,KTs@50,K9s@50,Q9s@50,J7s@50,94s@50,93s@50,54s@50,AJo@50,ATo@50,KQo@50,KJo@50,99@25,A4s@25,KJs@25,QJs@25,QTs@25,98s@25,87s@25,76s@25,65s@25";
    let ip = "22+,A2s+,K2s+,Q2s+,J6s+,T6s+,98s,97s,96s,87s,86s,85s,76s,75s,65s,64s,54s,A5o+,K9o+,Q9o+,JTo,J9o,T9o";
    //run_trainer(board, oop, ip, params, "btn_bb_srp").await;
    run_consumer().await?;
    Ok(())
}

async fn run_trainer(
    board: Board,
    oop_range: &str,
    ip_range: &str,
    params: GameParams,
    bucket_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let traversal = build_traversal_from_ranges(board, oop_range, ip_range);

    let mut game = Game::new(traversal, params, board);

    game.train(300);
    let file_name = format!(
        "{}{}{}.json",
        number_to_card(board[0]),
        number_to_card(board[1]),
        number_to_card(board[2])
    );
    game.output_results(bucket_name, file_name.as_ref()).await?;
    Ok(())
}

async fn run_consumer() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    println!("{}", addr);
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

            let b: Vec<u8> = p
                .board
                .split(",")
                .map(|x| card_to_number(x.to_string()))
                .collect();

            let board: Board = [b[0], b[1], b[2], 52, 52];

            println!("board: {:?}", board);

            run_trainer(
                board,
                &p.oop_range,
                &p.ip_range,
                GameParams::new(
                    1,
                    p.starting_pot,
                    p.starting_stack,
                    p.all_in_cut_off,
                    p.default_bet,
                    p.oop_flop_bets.unwrap_or(vec![vec![]]),
                    p.oop_turn_bets.unwrap_or(vec![vec![]]),
                    p.oop_river_bets.unwrap_or(vec![vec![]]),
                    p.ip_flop_bets.unwrap_or(vec![vec![]]),
                    p.ip_turn_bets.unwrap_or(vec![vec![]]),
                    p.ip_river_bets.unwrap_or(vec![vec![]]),
                ),
                p.bucket_name.as_ref(),
            )
            .await;
            channel
                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                .await?
        }
    }
    Ok(())
}
