FROM rustlang/rust:nightly-buster-slim as builder

WORKDIR /usr

RUN USER=root cargo new --bin poker-solver

WORKDIR /usr/poker-solver

COPY ./Cargo.toml ./Cargo.toml
RUN cargo +nightly build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/poker_solver*
RUN cargo +nightly build --release

CMD [ "./target/release/poker-solver" ]

