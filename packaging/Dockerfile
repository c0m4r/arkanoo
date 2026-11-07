# syntax=docker/dockerfile:1
#
# Builds self-contained Arkanoo binaries for Linux and Windows.
# Used by release.sh; run from the repository root:
#
#   docker build -f packaging/Dockerfile --output target/release-pkg .
#
# Ubuntu 22.04 sets the glibc floor (2.35) for the Linux binary.

FROM ubuntu:22.04 AS toolchain

ARG SDL2_VERSION=2.32.10
ARG SDL2_IMAGE_VERSION=2.8.12
ARG SDL2_MIXER_VERSION=2.8.2
ARG SDL2_TTF_VERSION=2.24.0
ARG RUST_VERSION=stable

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential cmake ninja-build git curl ca-certificates pkg-config \
        gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 binutils-mingw-w64-x86-64 \
        libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev \
        libxss-dev libxinerama-dev libxkbcommon-dev \
        libwayland-dev wayland-protocols libdecor-0-dev \
        libegl-dev libgl-dev libgles-dev libdrm-dev libgbm-dev \
        libasound2-dev libpulse-dev libpipewire-0.3-dev libdbus-1-dev libudev-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build/src
RUN git clone --depth 1 -b release-${SDL2_VERSION} https://github.com/libsdl-org/SDL.git \
    && git clone --depth 1 -b release-${SDL2_IMAGE_VERSION} https://github.com/libsdl-org/SDL_image.git \
    && git -C SDL_image submodule update --init --depth 1 external/libwebp \
    && git clone --depth 1 -b release-${SDL2_MIXER_VERSION} https://github.com/libsdl-org/SDL_mixer.git \
    && git clone --depth 1 -b release-${SDL2_TTF_VERSION} https://github.com/libsdl-org/SDL_ttf.git \
    && git -C SDL_ttf submodule update --init --depth 1 external/freetype

COPY packaging/build-sdl.sh packaging/mingw-w64.cmake /build/
RUN /build/build-sdl.sh linux /opt/sdl-linux
RUN /build/build-sdl.sh windows /opt/sdl-windows

ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal \
        --default-toolchain ${RUST_VERSION} --target x86_64-pc-windows-gnu

FROM toolchain AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock build.rs icon.ico ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    PKG_CONFIG_LIBDIR=/opt/sdl-linux/lib/pkgconfig \
        cargo build --release --locked --features static-sdl \
    && PKG_CONFIG_ALLOW_CROSS=1 PKG_CONFIG_LIBDIR=/opt/sdl-windows/lib/pkgconfig \
       CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc \
        cargo build --release --locked --features static-sdl --target x86_64-pc-windows-gnu \
    && mkdir /out \
    && cp target/release/arkanoo /out/ \
    && cp target/x86_64-pc-windows-gnu/release/arkanoo.exe /out/

FROM scratch
COPY --from=build /out/ /
COPY --from=toolchain /build/src/SDL/LICENSE.txt /licenses/SDL2.txt
COPY --from=toolchain /build/src/SDL_image/LICENSE.txt /licenses/SDL2_image.txt
COPY --from=toolchain /build/src/SDL_mixer/LICENSE.txt /licenses/SDL2_mixer.txt
COPY --from=toolchain /build/src/SDL_ttf/LICENSE.txt /licenses/SDL2_ttf.txt
COPY --from=toolchain /build/src/SDL_ttf/external/freetype/docs/FTL.TXT /licenses/FreeType.txt
COPY --from=toolchain /build/src/SDL_image/external/libwebp/COPYING /licenses/libwebp.txt
COPY --from=toolchain /build/src/SDL_image/external/libwebp/PATENTS /licenses/libwebp-PATENTS.txt
