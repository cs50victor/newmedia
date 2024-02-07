# Use a minimal Ubuntu image as the base image
FROM ubuntu:latest as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Install necessary build dependencies
RUN apt-get update && \
    apt-get install -y \
    build-essential \
    curl g++ pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-0 libwayland-dev libxkbcommon-dev

# Install Rust using rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Rust binaries to the PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy the rest of the application source code into the container
COPY . .

# Build the application
RUN cargo build --release  --features docker

# Use a minimal Ubuntu image as the final base image
FROM ubuntu:latest

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy only the necessary files from the builder stage
COPY --from=builder /usr/src/app/target/release/new_media .

# Specify the command to run on container start
CMD ["./new_media"]
