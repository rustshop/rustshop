FROM rust:1.61 AS chef
RUN apt update && apt install -y git && cargo install cargo-chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# need to build ssl
RUN apt-get update && apt-get install -y build-essential musl-tools
ENV PKG_CONFIG_ALLOW_CROSS=1

# Build dependencies - this is the caching Docker layer!
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
# Build application
COPY --from=planner /app/ /app/
RUN cargo build --release --target x86_64-unknown-linux-musl --bin aws-bootstrap

FROM debian AS runtime
RUN apt-get update && apt-get install -y awscli
WORKDIR app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/aws-bootstrap /usr/local/bin/
ENTRYPOINT [ "/usr/local/bin/aws-bootstrap" ]
CMD [ ]
EXPOSE 3000
