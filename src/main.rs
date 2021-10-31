mod cfr;
mod nodes;
mod ranges;

extern crate rayon;

use crate::{
    cfr::{game::Game, game_params::GameParams, traversal::Traversal},
    ranges::{
        combination::Board,
        range_manager::RangeManager,
        utility::{card_to_number, construct_starting_range_from_string},
    },
};

use tonic::{transport::Server, Response};

pub mod solver {
    tonic::include_proto!("regnivon.v1.solver");
    pub(crate) const REFLECTION_SERVICE_DESCRIPTOR: &[u8] =
        tonic::include_file_descriptor_set!("solver_v1");
}

pub mod common {
    tonic::include_proto!("regnivon.v1.common");
}

// use solver::greeter_server::{Greeter, GreeterServer};
// use solver::{HelloReply, HelloRequest};

use solver::solver_service_server::{SolverService, SolverServiceServer};
use solver::SolveGameRequest;
use solver::*;

#[derive(Debug, Default)]
pub struct Solver {}

#[tonic::async_trait]
impl SolverService for Solver {
    // async fn say_hello(
    //     &self,
    //     request: Request<HelloRequest>, // Accept request of type HelloRequest
    // ) -> Result<Response<HelloReply>, Status> {
    //     // Return an instance of type HelloReply
    //     println!("Got a request: {:?}", request);

    //     let reply = solver::HelloReply {
    //         message: format!("Hello {}!", request.into_inner().name).into(), // We must use .into_inner() as the fields of gRPC requests and responses are private
    //     };

    //     let blocking_task = tokio::task::spawn_blocking(run_trainer);
    // }

    async fn solve_game(
        &self,
        request: tonic::Request<SolveGameRequest>,
    ) -> Result<tonic::Response<SolveGameResponse>, tonic::Status> {
        let req = request.into_inner();

        println!("{:?}", req);

        let reply = SolveGameResponse {
            simulation_id: 12,
            status: SimulationStatus::Created as i32,
        };

        tokio::task::spawn_blocking(run_trainer);

        Ok(Response::new(reply))
    }

    async fn solution_status(
        &self,
        request: tonic::Request<SolutionStatusRequest>,
    ) -> Result<tonic::Response<SolutionStatusResponse>, tonic::Status> {
        let reply = SolutionStatusResponse {
            simulation_id: todo!(),
            status: todo!(),
            current_exploitability: todo!(),
        };

        Ok(Response::new(reply))
    }

    async fn get_node_results(
        &self,
        request: tonic::Request<GetNodeResultsRequest>,
    ) -> Result<tonic::Response<GetNodeResultsResponse>, tonic::Status> {
        let reply = GetNodeResultsResponse {
            simulation_id: todo!(),
            result: todo!(),
        };

        Ok(Response::new(reply))
    }

    async fn get_whole_street_solution(
        &self,
        request: tonic::Request<GetWholeStreetSolutionRequest>,
    ) -> Result<tonic::Response<GetWholeStreetSolutionResponse>, tonic::Status> {
        Ok(Response::new(GetWholeStreetSolutionResponse {
            simulation_id: todo!(),
            result: todo!(),
        }))
    }

    async fn get_frequencies_across_runouts(
        &self,
        request: tonic::Request<GetFrequenciesAcrossRunoutsRequest>,
    ) -> Result<tonic::Response<GetFrequenciesAcrossRunoutsResponse>, tonic::Status> {
        Ok(Response::new(GetFrequenciesAcrossRunoutsResponse {
            simulation_id: todo!(),
            results: todo!(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blocking_task = tokio::task::spawn_blocking(run_trainer);
    blocking_task.await.unwrap();

    // let addr = "[::1]:50051".parse()?;
    // let solver_server = Solver {};
    // let service = tonic_reflection::server::Builder::configure()
    //     .register_encoded_file_descriptor_set(solver::REFLECTION_SERVICE_DESCRIPTOR)
    //     .build()
    //     .unwrap();

    // Server::builder()
    //     .add_service(SolverServiceServer::new(solver_server))
    //     .add_service(service)
    //     .serve(addr)
    //     .await?;

    Ok(())
}

fn run_trainer() {
    let board: Board = [
        card_to_number("kc".to_string()),
        card_to_number("7h".to_string()),
        card_to_number("2d".to_string()),
        52, //card_to_number("3d".to_string()),
        52, //card_to_number("2c".to_string()),
    ];
    let starting_combinations = construct_starting_range_from_string("random".to_string(), &board);
    let starting_combinations2 = construct_starting_range_from_string("random".to_string(), &board);
    //let starting_combinations = construct_starting_range_from_string("TT,99,88,77,66,55,44,33,22,ATs,A9s,A8s,A7s,A6s,A5s,A4s,A3s,A2s,KTs,K9s,K8s,K7s,K6s,K5s,K4s,K3s,K2s,QTs,Q9s,Q8s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,JTs,J9s,J8s,J7s,J6s,J5s,J4s,T9s,T8s,T7s,T6s,T5s,T4s,98s,97s,96s,95s,94s,87s,86s,85s,76s,75s,74s,65s,65s,64s,54s,43s,AJo,ATo,A9o,A8o,A7o,A6o,A5o,A4o,KJo,KTo,QJo,QTo,JTo".to_string(), &board);
    //let starting_combinations2 = construct_starting_range_from_string("TT,99,88,77,66,55,44,33,22,ATs,A9s,A8s,A7s,A6s,A5s,A4s,A3s,A2s,KTs,K9s,K8s,K7s,K6s,K5s,K4s,K3s,K2s,QTs,Q9s,Q8s,Q7s,Q6s,Q5s,Q4s,Q3s,Q2s,JTs,J9s,J8s,J7s,J6s,J5s,J4s,T9s,T8s,T7s,T6s,T5s,T4s,98s,97s,96s,95s,94s,87s,86s,85s,76s,75s,74s,65s,65s,64s,54s,43s,AJo,ATo,A9o,A8o,A7o,A6o,A5o,A4o,KJo,KTo,QJo,QTo,JTo".to_string(), &board);
    let rm = RangeManager::new(starting_combinations, board);
    let rm2 = RangeManager::new(starting_combinations2, board);
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

    let trav = Traversal::new(rm, rm2);

    let mut game = Game::new(trav, params, board);

    //2.82
    game.train(50);
}
