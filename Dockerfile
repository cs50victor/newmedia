FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye as builder

WORKDIR /usr/src/app

# Install necessary build dependencies
RUN apt-get update && apt-get install -y \   
    pkg-config \    
    libasound2-dev \
    libudev-dev \   
    mesa-utils \ 
    vulkan-tools \
    libwayland-dev \
    libxkbcommon-dev \   
    libvulkan1 \    
    libvulkan-dev \ 
    libegl1-mesa-dev \   
    libgles2-mesa-dev \  
    libx11-dev \  
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libxrandr-dev \
    libxcb1-dev \
    libxcb-icccm4-dev \
    libxcb-image0-dev \
    libxcb-keysyms1-dev \
    libxcb-randr0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxcb-xkb-dev \
    libegl1-mesa \
    libgl1-mesa-glx \
    libgl1-mesa-dri \
    libglu1-mesa-dev \
    libglu1-mesa \
    libgles2-mesa \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*


COPY . .

# Build the application 
RUN cargo build --release --features docker

# Use a minimal Ubuntu image as the final base image
FROM ubuntu:latest

# Set the working directory inside the container
WORKDIR /usr/src/app

# Install necessary build dependencies
RUN apt-get update && apt-get install -y \   
    pkg-config \    
    libasound2-dev \
    libudev-dev \   
    mesa-utils \ 
    vulkan-tools \
    libwayland-dev \
    libxkbcommon-dev \   
    libvulkan1 \    
    libvulkan-dev \ 
    libegl1-mesa-dev \   
    libgles2-mesa-dev \  
    libx11-dev \    
    libxcursor-dev \
    libxrandr-dev \
    libxi-dev \
    libxrandr-dev \
    libxcb1-dev \
    libxcb-icccm4-dev \
    libxcb-image0-dev \
    libxcb-keysyms1-dev \
    libxcb-randr0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxcb-xkb-dev \
    libegl1-mesa \
    libgl1-mesa-glx \
    libgl1-mesa-dri \
    libglu1-mesa-dev \
    libglu1-mesa \
    libgles2-mesa \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Copy only the necessary files from the builder stage
COPY --from=builder /usr/src/app/target/release/new_media .


ENV XDG_RUNTIME_DIR=/tmp
ENV RUST_BACKTRACE=1
EXPOSE 8080
CMD ["./new_media"]
