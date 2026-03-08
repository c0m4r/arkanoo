#!/bin/bash

# Exit on error
set -e

echo "Detecting operating system..."

if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
    LIKE=$ID_LIKE
else
    echo "Error: /etc/os-release not found. Unable to detect OS."
    exit 1
fi

echo "OS Detected: $PRETTY_NAME"

case "$OS" in
    ubuntu|debian|linuxmint|pop|kali|raspbian)
        echo "Installing SDL2 libraries for Debian/Ubuntu based system..."
        sudo apt-get update
        sudo apt-get install libsdl2-2.0-0 libsdl2-mixer-2.0-0 libsdl2-ttf-2.0-0 libsdl2-image-2.0-0 \
                             libsdl2-dev libsdl2-mixer-dev libsdl2-ttf-dev libsdl2-image-dev
        ;;
    fedora|nobara|centos|rhel)
        echo "Installing SDL2 libraries for Fedora based system..."
        sudo dnf install SDL2 SDL2_mixer SDL2_ttf SDL2_image \
                        SDL2-devel SDL2_mixer-devel SDL2_ttf-devel SDL2_image-devel
        ;;
    arch|manjaro|endeavouros|garuda)
        echo "Installing SDL2 libraries for Arch based system..."
        sudo pacman -S --needed sdl2 sdl2_mixer sdl2_ttf sdl2_image
        ;;
    void)
        echo "Installing SDL2 libraries for Void Linux..."
        sudo xbps-install -S SDL2 SDL2_mixer SDL2_ttf SDL2_image \
                             SDL2-devel SDL2_mixer-devel SDL2_ttf-devel SDL2_image-devel
        ;;
    alpine)
        echo "Installing SDL2 libraries for Alpine Linux..."
        sudo apk add sdl2 sdl2_mixer sdl2_ttf sdl2_image \
                     sdl2-dev sdl2_mixer-dev sdl2_ttf-dev sdl2_image-dev
        ;;
    gentoo)
        echo "Installing SDL2 libraries for Gentoo..."
        # Gentoo packages include both headers and libraries
        sudo emerge --ask=y media-libs/libsdl2 media-libs/sdl2-mixer media-libs/sdl2-ttf media-libs/sdl2-image
        ;;
    *)
        # Fallback to ID_LIKE check for derivatives not explicitly mentioned
        # Using space around LIKE to ensure full-word matching
        if [[ " $LIKE " == *" debian "* ]] || [[ " $LIKE " == *" ubuntu "* ]]; then
            echo "Detected Debian-like system via ID_LIKE..."
            sudo apt-get update
            sudo apt-get install libsdl2-2.0-0 libsdl2-mixer-2.0-0 libsdl2-ttf-2.0-0 libsdl2-image-2.0-0 \
                                 libsdl2-dev libsdl2-mixer-dev libsdl2-ttf-dev libsdl2-image-dev
        elif [[ " $LIKE " == *" fedora "* ]] || [[ " $LIKE " == *" rhel "* ]] || [[ " $LIKE " == *" centos "* ]]; then
            echo "Detected Fedora-like system via ID_LIKE..."
            sudo dnf install SDL2 SDL2_mixer SDL2_ttf SDL2_image \
                            SDL2-devel SDL2_mixer-devel SDL2_ttf-devel SDL2_image-devel
        elif [[ " $LIKE " == *" arch "* ]]; then
            echo "Detected Arch-like system via ID_LIKE..."
            sudo pacman -S --needed sdl2 sdl2_mixer sdl2_ttf sdl2_image
        elif [[ " $LIKE " == *" void "* ]]; then
            echo "Detected Void-like system via ID_LIKE..."
            sudo xbps-install -S SDL2 SDL2_mixer SDL2_ttf SDL2_image \
                                 SDL2-devel SDL2_mixer-devel SDL2_ttf-devel SDL2_image-devel
        elif [[ " $LIKE " == *" alpine "* ]]; then
            echo "Detected Alpine-like system via ID_LIKE..."
            sudo apk add sdl2 sdl2_mixer sdl2_ttf sdl2_image \
                         sdl2-dev sdl2_mixer-dev sdl2_ttf-dev sdl2_image-dev
        elif [[ " $LIKE " == *" gentoo "* ]]; then
            echo "Detected Gentoo-like system via ID_LIKE..."
            sudo emerge --ask=y media-libs/libsdl2 media-libs/sdl2-mixer media-libs/sdl2-ttf media-libs/sdl2-image
        else
            echo "Unsupported OS: $OS ($LIKE)"
            echo "Please install SDL2 development libraries manually."
            echo "Example (Debian): sudo apt-get install libsdl2-dev libsdl2-mixer-dev libsdl2-ttf-dev libsdl2-image-dev"
            exit 1
        fi
        ;;
esac

echo "Installation complete!"
