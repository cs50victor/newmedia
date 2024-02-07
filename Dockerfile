# FROM rust as planner
# WORKDIR /app
# RUN cargo install cargo-chef
# COPY . .
# RUN cargo chef prepare --recipe-path recipe.json

# FROM rust as cacher
# WORKDIR /app
# RUN cargo install cargo-chef
# COPY --from=planner /app/recipe.json recipe.json
# RUN cargo chef cook --release --recipe-path=recipe.json

# FROM rust as builder
# COPY . /app
# WORKDIR /app
# COPY --from=cacher /app/target target
# COPY --from=cacher /usr/local/cargo /usr/local/cargo
# RUN cargo build --release

# FROM bitnami/minideb:bookworm
# COPY --from=builder /app/target/release/new_media /app/new_media
# WORKDIR /app

# EXPOSE 8080
# CMD ["./new_media"]

#  --------------

# FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
# WORKDIR /app

# FROM chef AS planner
# COPY . .
# RUN cargo chef prepare --recipe-path recipe.json

# FROM chef AS builder 
# COPY --from=planner /app/recipe.json recipe.json
# RUN apt-get update -qq && apt-get -y install --no-install-recommends \
#     libasound2-dev \ 
#     libudev-dev \ 
#     libwayland-dev \
#     libxkbcommon-dev
# # # Build dependencies - this is the caching Docker layer!
# RUN cargo chef cook --release --features docker --recipe-path recipe.json

# # Build application
# COPY . .
# RUN cargo build --release --features docker

# # We do not need the Rust toolchain to run the binary!
# FROM debian:bookworm-slim AS runtime
# WORKDIR /app
# COPY --from=builder /app/target/release/new_media /usr/local/bin

# EXPOSE 8080
# ENTRYPOINT ["/usr/local/bin/new_media"]


FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN apt-get update -qq && apt-get -y install --no-install-recommends \
    libasound2-dev \ 
    libudev-dev \ 
    libwayland-dev \
    libxkbcommon-dev
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --features docker --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --features docker

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/new_media /usr/local/bin
ENTRYPOINT ["/usr/local/bin/app"]