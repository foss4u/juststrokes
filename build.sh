#!/bin/bash
# Build optimized binaries for Linux

set -e

echo "Building JustStrokes optimized binaries..."
echo

# Build for Linux x86_64 with glibc (default)
echo "=== Building for Linux x86_64 (glibc) ==="
cargo build --release --target x86_64-unknown-linux-gnu
echo "✓ Built: target/x86_64-unknown-linux-gnu/release/juststrokes-rust"
echo

# Build for Linux x86_64 with musl (static linking)
echo "=== Building for Linux x86_64 (musl, static) ==="
if ! rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then
    echo "Installing musl target..."
    rustup target add x86_64-unknown-linux-musl
fi

cargo build --release --target x86_64-unknown-linux-musl
echo "✓ Built: target/x86_64-unknown-linux-musl/release/juststrokes-rust"
echo

# Show binary sizes
echo "=== Binary Sizes ==="
ls -lh target/x86_64-unknown-linux-gnu/release/juststrokes-rust | awk '{print "glibc:  " $5 " " $9}'
ls -lh target/x86_64-unknown-linux-musl/release/juststrokes-rust | awk '{print "musl:   " $5 " " $9}'
echo

# Strip binaries for smaller size
echo "=== Stripping binaries ==="
strip target/x86_64-unknown-linux-gnu/release/juststrokes-rust
strip target/x86_64-unknown-linux-musl/release/juststrokes-rust
echo "✓ Stripped both binaries"
echo

echo "=== Stripped Binary Sizes ==="
ls -lh target/x86_64-unknown-linux-gnu/release/juststrokes-rust | awk '{print "glibc:  " $5 " " $9}'
ls -lh target/x86_64-unknown-linux-musl/release/juststrokes-rust | awk '{print "musl:   " $5 " " $9}'
echo

echo "Build complete!"
echo
echo "Usage:"
echo "  glibc:  target/x86_64-unknown-linux-gnu/release/juststrokes-rust [data_file] [socket_path]"
echo "  musl:   target/x86_64-unknown-linux-musl/release/juststrokes-rust [data_file] [socket_path]"
