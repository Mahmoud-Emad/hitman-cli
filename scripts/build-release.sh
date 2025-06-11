#!/bin/bash

# Build script for creating release binaries for all platforms
# Usage: ./scripts/build-release.sh [version]

set -e

VERSION=${1:-"dev"}
BUILD_DIR="dist"
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
    "aarch64-pc-windows-msvc"
)

echo "🚀 Building Hitman v${VERSION} for all platforms..."

# Clean previous builds
rm -rf ${BUILD_DIR}
mkdir -p ${BUILD_DIR}

# Check if cross is installed
if ! command -v cross &> /dev/null; then
    echo "📦 Installing cross..."
    cargo install cross
fi

# Build for each target
for target in "${TARGETS[@]}"; do
    echo "🔨 Building for ${target}..."
    
    if [[ "$target" == *"apple"* ]] && [[ "$OSTYPE" != "darwin"* ]]; then
        echo "⚠️  Skipping ${target} (requires macOS)"
        continue
    fi
    
    # Use cross for Linux targets, cargo for native builds
    if [[ "$target" == *"linux"* ]] && [[ "$OSTYPE" != "linux"* ]]; then
        cross build --release --target ${target}
    else
        # Add target if not already installed
        rustup target add ${target} 2>/dev/null || true
        cargo build --release --target ${target}
    fi
    
    # Package the binary
    binary_name="hitman"
    if [[ "$target" == *"windows"* ]]; then
        binary_name="hitman.exe"
        archive_name="hitman-${VERSION}-${target}.zip"
        cd target/${target}/release
        zip ../../../${BUILD_DIR}/${archive_name} ${binary_name}
        cd - > /dev/null
    else
        archive_name="hitman-${VERSION}-${target}.tar.gz"
        cd target/${target}/release
        tar czf ../../../${BUILD_DIR}/${archive_name} ${binary_name}
        cd - > /dev/null
    fi
    
    echo "✅ Created ${archive_name}"
done

# Generate checksums
echo "🔐 Generating checksums..."
cd ${BUILD_DIR}
sha256sum *.tar.gz *.zip > checksums.txt 2>/dev/null || shasum -a 256 *.tar.gz *.zip > checksums.txt
cd - > /dev/null

echo "🎉 Build complete! Artifacts are in ${BUILD_DIR}/"
ls -la ${BUILD_DIR}/
