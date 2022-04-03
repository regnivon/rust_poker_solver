#![feature(core_intrinsics)]
#![feature(portable_simd)]
#![feature(test)]
#![feature(stdsimd)]
mod cfr;
mod nodes;
mod ranges;
mod messaging;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

extern crate rayon;

use std::error::Error;
use crate::{
    cfr::{game_params::GameParams, game::run_trainer},
    ranges::{
        combination::Board,
        utility::{
            card_to_number
        },
    },
};
use tracing::info;
use crate::messaging::run_consumer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let board: Board = [
        card_to_number("kc".to_string()),
        card_to_number("7h".to_string()),
        card_to_number("2d".to_string()),
        52,
        52,
    ];

    // time to beat: 140 seconds

    let params = GameParams::new(
        1,
        60.0,
        1000.0,
        20.0,
        0.75,
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
        vec![vec![0.75]],
    );
    // AKs@0,AQs@0,AJs@0,ATs@0,A9s@0,A8s@0,A7s@0,A6s@0,A5s@0,A4s@0,A3s@0,A2s@0
//AA,KK,QQ,JJ,TT,99,88,77,66,55,44,33,22,A2s+,K2s+,Q2s+,JTs,J9s,J8s,J7s,T9s,T8s,T7s,T6s,98s,97s,96s,87s,86s,76s,65s,A5o+,KTo+,QTo+
    //let oop = "22+,A2s+,K2s+,Q2s+,J6s+,T6s+,98s,97s,96s,87s,86s,85s,76s,75s,65s,64s,54s,A5o+,K9o+,Q9o+,JTo,J9o,T9o";
    let oop = "77,66,55,44,33,22,A7s,A6s,K8s,K5s,K4s,K3s,K2s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,J6s,J5s,J4s,J3s,J2s,T5s,T4s,T3s,T2s,96s,95s,85s,84s,74s,73s,63s,53s,52s,43s,42s,32s,A9o,A8o,A7o,A6o,A5o,A4o,A3o,KTo,K9o,K8o,K7o,K6o,QTo,Q9o,Q8o,JTo,J9o,J8o,T9o,T8o,98o,87o,76o,65o,A9s@75,A8s@75,A2s@75,K7s@75,K6s@75,Q8s@75,T7s@75,T6s@75,97s@75,86s@75,75s@75,64s@75,QJo@75,88@50,ATs@50,A3s@50,KTs@50,K9s@50,Q9s@50,J7s@50,94s@50,93s@50,54s@50,AJo@50,ATo@50,KQo@50,KJo@50,99@25,A4s@25,KJs@25,QJs@25,QTs@25,98s@25,87s@25,76s@25,65s@25";
    let ip = "22+,A2s+,K2s+,Q2s+,J6s+,T6s+,98s,97s,96s,87s,86s,85s,76s,75s,65s,64s,54s,A5o+,K9o+,Q9o+,JTo,J9o,T9o";

    // let oop = "AA,KK,QQ,JJ,TT,99,88,77,66,55,44,33,22,A2s+,K2s+,Q2s+,JTs,J9s,J8s,J7s,T9s,T8s,T7s,T6s,98s,97s,96s,87s,86s,76s,65s,A5o+,KTo+,QTo+";
    // let ip = "AA,KK,QQ,JJ,TT,99,88,77,66,55,44,33,22,A2s+,K2s+,Q2s+,JTs,J9s,J8s,J7s,T9s,T8s,T7s,T6s,98s,97s,96s,87s,86s,76s,65s,A5o+,KTo+,QTo+";
    match run_trainer(board, oop, ip, params, "btn_bb_srp").await {
        Ok(_) => {}
        Err(e) => info!("Error during execution {}", e)
    }
    // run_consumer().await;
    Ok(())
}
