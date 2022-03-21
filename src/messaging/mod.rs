use futures_lite::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use serde::{Deserialize, Serialize};
use std::str;
use lapin::message::Delivery;
use tracing::{error, info};
use crate::{Board, card_to_number, GameParams};
use crate::cfr::game::run_trainer;

pub async fn run_consumer() {
    loop {
        match build_and_run_consumer().await {
            Ok(_) => {}
            Err(e) => {
                error!("Error while running consumer, retrying {}", e);
            }
        }
    }
}


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

async fn build_and_run_consumer() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    info!("{}", addr);
    let connection_props = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);
    let conn = Connection::connect(&addr, connection_props).await?;
    let channel = conn.create_channel().await?;

    let mut consumer = channel
        .basic_consume(
            "sims",
            "solver",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    info!("rmq consumer connected, waiting for messages");
    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => process_delivery(delivery).await?,
            Err(e) => error!("Error consuming next {}", e),
        }
    }
    Ok(())
}

async fn process_delivery(delivery: Delivery) -> Result<(), Box<dyn std::error::Error>> {
    let data = str::from_utf8(&delivery.data).expect("can't convert message to string");
    let p: SolutionConfig = serde_json::from_str(data)?;

    info!("received msg: {:?}", p);

    let b: Vec<u8> = p
        .board
        .split(',')
        .map(|x| card_to_number(x.to_string()))
        .collect();

    let board: Board = [b[0], b[1], b[2], 52, 52];

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
            p.oop_flop_bets.unwrap_or_else(|| vec![vec![]]),
            p.oop_turn_bets.unwrap_or_else(|| vec![vec![]]),
            p.oop_river_bets.unwrap_or_else(|| vec![vec![]]),
            p.ip_flop_bets.unwrap_or_else(|| vec![vec![]]),
            p.ip_turn_bets.unwrap_or_else(|| vec![vec![]]),
            p.ip_river_bets.unwrap_or_else(|| vec![vec![]]),
        ),
        p.bucket_name.as_ref(),
    )
        .await?;
    delivery.ack(BasicAckOptions::default()).await?;
    Ok(())
}
