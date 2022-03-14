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

use crate::cfr::traversal::build_traversal_from_ranges;
use crate::ranges::utility::number_to_card;
use futures_lite::stream::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use serde::{Deserialize, Serialize};
use std::str;

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SolutionConfig {
    pub bucket_name: String,
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
        1.0,
        0.75,
        vec![vec![]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
    );

    run_trainer(board, "random", "random", params, "btn_bb_srp").await;
    //run_consumer().await?;
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

    game.train(100);
    let file_name = format!(
        "{}{}{}.json",
        number_to_card(board[0]),
        number_to_card(board[1]),
        number_to_card(board[2])
    );
    //game.output_results(bucket_name, file_name.as_ref()).await?;
    Ok(())
}

async fn run_consumer() -> Result<(), Box<dyn std::error::Error>> {
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

            let b: Vec<u8> = p
                .board
                .split(",")
                .map(|x| card_to_number(x.to_string()))
                .collect();

            let board: Board = [b[0], b[1], b[2], 52, 52];

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
