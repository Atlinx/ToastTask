FROM rust:1.70.0

RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch

WORKDIR /usr/src/app

ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000

# Caching crates
VOLUME ["/usr/local/cargo"]