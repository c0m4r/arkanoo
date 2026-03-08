#!/bin/bash
# Arkanoo Release Script
set -e

# Extract version from Cargo.toml
VERSION=$(grep -m 1 "^version =" Cargo.toml | cut -d '"' -f 2)
if [ -z "$VERSION" ]; then
    echo "Error: Could not extract version from Cargo.toml"
    exit 1
fi

echo "Preparing release for Arkanoo version: $VERSION"

# Ensure the project is built in release mode
echo "Building the project..."
chmod +x ./build.sh
./build.sh

# Create the VERSION file
echo "$VERSION" > VERSION.tmp

# Prepare the release package structure in a temporary directory
PKG_NAME="arkanoo-$VERSION"
BUILD_DIR="release_build_tmp"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/$PKG_NAME"

# Copy all required files to the package directory
echo "Assembling package..."
cp target/release/arkanoo "$BUILD_DIR/$PKG_NAME/"
cp -r assets "$BUILD_DIR/$PKG_NAME/"
cp -r patterns "$BUILD_DIR/$PKG_NAME/"
cp LICENSE "$BUILD_DIR/$PKG_NAME/" 2>/dev/null || touch "$BUILD_DIR/$PKG_NAME/LICENSE"
cp README.md "$BUILD_DIR/$PKG_NAME/" 2>/dev/null || touch "$BUILD_DIR/$PKG_NAME/README.md"
cp VERSION.tmp "$BUILD_DIR/$PKG_NAME/VERSION"
cp install_sdl2.sh "$BUILD_DIR/$PKG_NAME/"

# Generate checksums for the binary
echo "Generating checksums..."
cd "$BUILD_DIR/$PKG_NAME"
# Use standard format (hash filename) for sha256sum
sha256sum arkanoo > arkanoo.sha256

# Generate .wrl checksum (Whirlpool)
if command -v whirlpoolsum >/dev/null; then
    whirlpoolsum arkanoo > arkanoo.wrl
else
    echo "Warning: No whirlpool hashing tool found. Skipping arkanoo.wrl."
fi
cd ../..

# Create dist directory if it doesn't exist
mkdir -p dist

# Create the final tar.gz package
PACKAGE_FILE="arkanoo-$VERSION-linux.tar.gz"
echo "Creating tarball: dist/$PACKAGE_FILE"
tar -czf "dist/$PACKAGE_FILE" -C "$BUILD_DIR" "$PKG_NAME"

# Clean up
rm -rf "$BUILD_DIR"
rm VERSION.tmp

echo "--------------------------------------------------"
echo "Release successfully created: dist/$PACKAGE_FILE"
echo "Contents included:"
echo "  - arkanoo binary"
echo "  - assets/ directory"
echo "  - patterns/ directory"
echo "  - LICENSE & README.md"
echo "  - VERSION file"
echo "  - install_sdl2.sh"
echo "  - arkanoo.sha256 (SHA-256 hash)"
echo "  - arkanoo.wrl (Whirlpool hash)"
echo "--------------------------------------------------"
