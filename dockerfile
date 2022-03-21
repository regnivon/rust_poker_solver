FROM rustlang/rust:nightly as builder

WORKDIR /usr

RUN apt-get update -y && apt-get install -y pkg-config openssl cmake clang

WORKDIR /usr/poker-solver

ADD . ./

RUN cargo +nightly build --release

CMD [ "./target/release/poker-solver" ]

