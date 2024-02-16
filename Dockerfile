FROM rust as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
# Build tools
RUN apt-get update && apt-get install -y -qq xorg xauth
RUN cargo chef cook --release --features docker --recipe-path=recipe.json

FROM rust as builder
COPY . /app
WORKDIR /app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
# Build tools
RUN apt-get update && apt-get install -y -qq xorg xauth
RUN cargo build --release --features docker

FROM ubuntu:22.04
COPY --from=builder /app/target/release/new_media /app/new_media
WORKDIR /app

# Build tools
RUN apt-get update
RUN apt-get install -y -qq build-essential software-properties-common pkg-config xorg openbox xauth
# Bevy dependencies
RUN DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends -yq libasound2-dev libudev-dev libxkbcommon-x11-0
RUN apt-get update -y -qq
RUN add-apt-repository ppa:kisak/turtle -y
RUN apt-get update
RUN apt install -y xvfb libegl1-mesa libgl1-mesa-dri libxcb-xfixes0-dev mesa-vulkan-drivers
ENV CARGO_TARGET_DIR="../rust-target"
ENV PATH="/root/.cargo/bin:${PATH}"

EXPOSE 8080

CMD xvfb-run -l -s "-screen 0 1024x768x24" "./new_media"