#!/bin/bash
# Arkanoo Release Script
#
# Builds self-contained Linux and Windows packages. SDL2 and its add-ons are
# linked statically inside a container (see packaging/Dockerfile), so players
# don't need to install anything. Requires Docker with BuildKit, or Podman
# (run with ENGINE=podman).
set -e

# Extract version from Cargo.toml
VERSION=$(grep -m 1 "^version =" Cargo.toml | cut -d '"' -f 2)
if [ -z "$VERSION" ]; then
    echo "Error: Could not extract version from Cargo.toml"
    exit 1
fi

echo "Preparing release for Arkanoo version: $VERSION"

ENGINE=${ENGINE:-docker}
BIN_DIR="target/release-pkg"
BUILD_DIR="release_build_tmp"
PKG_NAME="arkanoo-$VERSION"

echo "Running clippy..."
cargo clippy

# The first run builds the SDL toolchain image (a few minutes); later runs
# reuse it along with cached cargo builds.
echo "Building Linux and Windows binaries with $ENGINE..."
rm -rf "$BIN_DIR"
"$ENGINE" build -f packaging/Dockerfile --output "type=local,dest=$BIN_DIR" .

rm -rf "$BUILD_DIR"
mkdir -p dist

# Assemble a package directory for one platform: package <platform> <binary>
package() {
    local platform=$1
    local binary=$2
    local dir="$BUILD_DIR/$platform/$PKG_NAME"

    mkdir -p "$dir"
    cp "$BIN_DIR/$binary" "$dir/"
    cp -r assets patterns "$dir/"
    cp -r "$BIN_DIR/licenses" "$dir/"
    cp LICENSE "$dir/" 2>/dev/null || touch "$dir/LICENSE"
    cp README.md "$dir/" 2>/dev/null || touch "$dir/README.md"
    echo "$VERSION" > "$dir/VERSION"

    # Use standard format (hash filename) for sha256sum
    (cd "$dir" && sha256sum "$binary" > "$binary.sha256")

    # Generate .wrl checksum (Whirlpool)
    if command -v whirlpoolsum >/dev/null; then
        (cd "$dir" && whirlpoolsum "$binary" > "$binary.wrl")
    else
        echo "Warning: No whirlpool hashing tool found. Skipping $binary.wrl."
    fi
}

echo "Assembling packages..."
package linux arkanoo
package windows arkanoo.exe

LINUX_PACKAGE="arkanoo-$VERSION-linux.tar.gz"
WINDOWS_PACKAGE="arkanoo-$VERSION-windows.zip"

echo "Creating dist/$LINUX_PACKAGE"
tar -czf "dist/$LINUX_PACKAGE" -C "$BUILD_DIR/linux" "$PKG_NAME"

echo "Creating dist/$WINDOWS_PACKAGE"
rm -f "dist/$WINDOWS_PACKAGE"
(cd "$BUILD_DIR/windows" && zip -qr "../../dist/$WINDOWS_PACKAGE" "$PKG_NAME")

# Clean up
rm -rf "$BUILD_DIR"

echo "--------------------------------------------------"
echo "Release successfully created:"
echo "  - dist/$LINUX_PACKAGE"
echo "  - dist/$WINDOWS_PACKAGE"
echo "Each package contains:"
echo "  - arkanoo / arkanoo.exe (SDL2 statically linked)"
echo "  - assets/ and patterns/ directories"
echo "  - licenses/ (third-party licenses for bundled libraries)"
echo "  - LICENSE & README.md"
echo "  - VERSION file"
echo "  - SHA-256 and Whirlpool hashes of the binary"
echo "--------------------------------------------------"
echo
(cd dist && sha256sum "$LINUX_PACKAGE" "$WINDOWS_PACKAGE")
