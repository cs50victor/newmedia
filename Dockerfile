FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye as builder

WORKDIR /usr/src/app

COPY . .

# Build the application 
RUN cargo build --release --features docker

#  --------------------------------------------------
# FROM fedora:latest

# WORKDIR /usr/src/app

# USER root

# ENV DEBIAN_FRONTEND=noninteractive

# # # Install necessary build dependencies
# RUN dnf update -y && dnf install -y gcc-c++ libX11-devel alsa-lib-devel systemd-devel wayland-devel libxkbcommon-devel weston libXcursor libXrandr
# # RUN dnf update -y && dnf install -y \   
# #     pkg-config \    
# #     weston \
# #     xwayland \
# #     kbd \
# #     dos2unix \
# #     libasound2-dev \
# #     libudev-dev \   
# #     mesa-utils \ 
# #     vulkan-tools \
# #     libwayland-dev \
# #     libxkbcommon-dev \   
# #     libvulkan1 \    
# #     libvulkan-dev \ 
# #     libegl1-mesa-dev \   
# #     libgles2-mesa-dev \  
# #     libx11-dev \    
# #     libxcursor-dev \
# #     libxrandr-dev \
# #     libxi-dev \
# #     libxrandr-dev \
# #     libxcb1-dev \
# #     libxcb-icccm4-dev \
# #     libxcb-image0-dev \
# #     libxcb-keysyms1-dev \
# #     libxcb-randr0-dev \
# #     libxcb-shape0-dev \
# #     libxcb-xfixes0-dev \
# #     libxcb-xkb-dev \
# #     libegl1-mesa \
# #     libgl1-mesa-glx \
# #     libgl1-mesa-dri \
# #     libglu1-mesa-dev \
# #     libglu1-mesa \
# #     libgles2-mesa \
# #     && apt-get clean \
# #     && rm -rf /var/lib/apt/lists/*

# RUN mkdir tmp/

# ENV XDG_RUNTIME_DIR=tmp
# ENV WLR_BACKENDS=headless
# ENV WLR_LIBINPUT_NO_DEVICES=1
# ENV WAYLAND_DISPLAY=wayland-0
# ENV DISPLAY=:1

# COPY --from=builder /usr/src/app/target/release/new_media .
# ENV RUST_BACKTRACE=1

# EXPOSE 8080

# CMD ["./new_media"]


# ----------------- TESTING

FROM registry.fedoraproject.org/fedora-minimal:37
RUN microdnf install -y --setopt install_weak_deps=0 busybox spice-html5 python3-websockify novnc weston labwc sway wayvnc dbus-daemon procps-ng foot wofi bemenu google-noto-naskh-arabic-fonts dejavu-fonts-all ; microdnf clean all 

RUN mkdir /opt/busybox; \
    /sbin/busybox --install -s /opt/busybox
ENV PATH=/usr/share/Modules/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/opt/busybox
RUN cp /usr/share/weston/background.png /usr/share/backgrounds/default.png ; \
    busybox adduser -D app ; \
    busybox passwd -l app ; \
    mkdir -p /home/app/tmp ; busybox chown app:app /home/app/tmp
ADD sway /etc/sway/config.d/sway
ADD labwc /etc/xdg/labwc

USER app
ENV SHELL=/bin/bash
ENV PATH=/home/app/.local/bin:/home/app/bin:/bin:/usr/bin:/usr/local/sbin:/usr/sbin:/opt/busybox
ENV XDG_RUNTIME_DIR=/home/app/tmp
ENV WLR_BACKENDS=headless
ENV WLR_LIBINPUT_NO_DEVICES=1
ENV WAYLAND_DISPLAY=wayland-1

ADD start.sh /start.sh

EXPOSE 5900
EXPOSE 8080


ENTRYPOINT ["/start.sh"]


