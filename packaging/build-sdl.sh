#!/bin/bash
# Build static SDL2, SDL2_image, SDL2_mixer and SDL2_ttf into a prefix.
# Only the formats Arkanoo uses are enabled (PNG, WebP, MP3, TTF), and every
# codec is either built into the SDL library or vendored, so nothing is
# loaded from the system at runtime except the display/audio backends.
#
# Usage: build-sdl.sh linux|windows PREFIX
set -euo pipefail

TARGET=$1
PREFIX=$2
SRC=/build/src
OUT=/build/out-$TARGET

common=(
    -G Ninja
    -DCMAKE_BUILD_TYPE=Release
    -DCMAKE_INSTALL_PREFIX="$PREFIX"
    -DBUILD_SHARED_LIBS=OFF
    -DCMAKE_POSITION_INDEPENDENT_CODE=ON
)
if [ "$TARGET" = windows ]; then
    common+=(-DCMAKE_TOOLCHAIN_FILE=/build/mingw-w64.cmake)
fi

build() {
    local name=$1; shift
    cmake -S "$SRC/$name" -B "$OUT/$name" "${common[@]}" "$@"
    cmake --build "$OUT/$name"
    cmake --install "$OUT/$name"
}

build SDL \
    -DSDL_SHARED=OFF -DSDL_STATIC=ON -DSDL_STATIC_PIC=ON \
    -DSDL_TEST=OFF -DSDL_IBUS=OFF

common+=(-DSDL2_DIR="$PREFIX/lib/cmake/SDL2")

build SDL_image \
    -DSDL2IMAGE_SAMPLES=OFF -DSDL2IMAGE_STRICT=ON \
    -DSDL2IMAGE_VENDORED=ON -DSDL2IMAGE_DEPS_SHARED=OFF \
    -DSDL2IMAGE_BACKEND_STB=ON -DSDL2IMAGE_BACKEND_WIC=OFF \
    -DSDL2IMAGE_WEBP=ON \
    -DSDL2IMAGE_AVIF=OFF -DSDL2IMAGE_JXL=OFF -DSDL2IMAGE_TIF=OFF
# The generated SDL2_image.pc omits the vendored static libwebp.
sed -i 's/^Libs.private:.*/& -lwebpdemux -lwebp/' "$PREFIX/lib/pkgconfig/SDL2_image.pc"

build SDL_mixer \
    -DSDL2MIXER_SAMPLES=OFF -DSDL2MIXER_VENDORED=OFF -DSDL2MIXER_DEPS_SHARED=OFF \
    -DSDL2MIXER_CMD=OFF -DSDL2MIXER_MIDI=OFF -DSDL2MIXER_MOD=OFF \
    -DSDL2MIXER_OPUS=OFF -DSDL2MIXER_WAVPACK=OFF -DSDL2MIXER_GME=OFF

build SDL_ttf \
    -DSDL2TTF_SAMPLES=OFF -DSDL2TTF_VENDORED=ON -DSDL2TTF_HARFBUZZ=OFF

rm -rf "$OUT"
