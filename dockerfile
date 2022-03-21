FROM rustlang/rust:nightly as builder

WORKDIR /usr

RUN apt-get update -y && apt-get install -y pkg-config openssl cmake clang

WORKDIR /usr/poker-solver

ADD . ./

RUN rm ./target/release/deps/poker_solver*
RUN cargo +nightly build --release

CMD [ "./target/release/poker-solver" ]

