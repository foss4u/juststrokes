# JustStrokes

Chinese character (Hanzi) handwriting recognition service written in Rust with Unix socket API for local integration.

> **Note:** This is the `rust` branch containing only the Rust implementation. For the original TypeScript/JavaScript version, see the `main` branch.

## Overview

High-performance handwriting recognition service translated from TypeScript to Rust. Matches user-drawn strokes against a database of ~9,500 Chinese characters with a Unix socket API for local integration.

## Features

- **Stroke-based matching**: Recognizes characters based on stroke order and shape
- **Multiple data formats**: JSON and CSV (tab-delimited, UTF-8)
- **Unix socket API**: Local service for handwriting applications
- **High accuracy**: 100% test pass rate on database self-matching
- **Optimized builds**: Size-optimized binaries for glibc and musl
- **Rust 2024 edition**: Latest Rust features and idioms

## Algorithm

Recognition pipeline:

1. **AABB Normalization**: Calculate bounding box and normalize to [0, 255] space
2. **Stroke Resampling**: Downsample each stroke to exactly 4 evenly-spaced points
3. **Feature Encoding**: Encode stroke angle and length
4. **Similarity Scoring**: Compare against database using point distances and angle differences

## Unix Socket Service

### Starting the Service

```bash
# Show version
./juststrokes-rust --version
./juststrokes-rust -V

# Show help
./juststrokes-rust --help
./juststrokes-rust -h

# Using default paths
./juststrokes-rust

# Custom data file
./juststrokes-rust --data-file graphics.csv
./juststrokes-rust -d graphics.json

# Custom socket path
./juststrokes-rust --socket-path /tmp/juststrokes.socket
./juststrokes-rust -s /tmp/juststrokes.socket

# Both custom
./juststrokes-rust -d graphics.csv -s /tmp/juststrokes.socket
```

Default socket path: `/run/user/$UID/handwritten/juststrokes.socket`

### API Format

**Input** (CSV, tab-delimited, UTF-8):
```
max_width\tmax_height\tstroke1_points\tstroke2_points\t...
```

Each stroke: `x0,y0,x1,y1,x2,y2,...` (comma-separated coordinates)

Example:
```
400\t400\t0,0,100,100,200,200\t50,50,150,150
```

**Output** (CSV, tab-delimited, UTF-8):
```
candidate1\tcandidate2\tcandidate3\t...
```

Example:
```
一\t丨\t丶\t...
```

### Testing the Service

```bash
# Start service
./juststrokes-rust graphics.csv &

# Send test request
echo -e "400\t400\t0,0,100,100,200,200\t50,50,150,150" | \
  socat - UNIX-CONNECT:/run/user/$UID/handwritten/juststrokes.socket
```

## Library Usage

```rust
use juststrokes_rust::{Matcher, Stroke, csv_data};

// Load character database (CSV or JSON)
let data = csv_data::load_graphics_csv("graphics.csv")?;

// Create matcher
let matcher = Matcher::new(data, None);

// Match user input strokes
let strokes: Vec<Stroke> = vec![
    vec![[10.0, 10.0], [50.0, 50.0], [90.0, 90.0]],
    // ... more strokes
];

let candidates = matcher.match_strokes(&strokes, 10);
println!("Top candidates: {:?}", candidates);
```

## Data Formats

### JSON Format (graphics.json)
```json
[["字", [[x0,y0,x1,y1,x2,y2,x3,y3,angle,length], ...]], ...]
```

### CSV Format (graphics.csv)
```
字\tx0,y0,x1,y1,x2,y2,x3,y3,angle,length\t...
```

CSV is 29% smaller than JSON (3.9MB vs 5.5MB).

## Building

### Development Build
```bash
cargo build
cargo test
```

### Optimized Builds
```bash
./build.sh
```

Produces two optimized binaries:
- **glibc** (506KB): `target/x86_64-unknown-linux-gnu/release/juststrokes-rust`
- **musl** (593KB): `target/x86_64-unknown-linux-musl/release/juststrokes-rust` (static)

The musl binary is statically linked and portable across Linux distributions.

## Testing

```bash
cargo test
```

Test suite includes:
- Vector operations
- AABB calculations
- CSV/JSON data loading
- Character self-matching (9,574 characters)
- Unix socket service

All tests pass (10/10).

## Performance

Optimized build performance:
- **Throughput**: 9,110 characters/sec
- **Average match time**: 1,051ms for 9,574 characters
- **Improvement**: 6.5% faster than baseline

## Dependencies

All dependencies use minimal feature sets for faster builds:

- `serde` - Serialization (derive, alloc only)
- `serde_json` - JSON parsing (std only)
- `libc` - Unix socket operations (no default features)
- `clap` - Command-line parsing (std, help, usage, derive only)

Build time optimized for Linux targets only.

## CI/CD

Automated builds on every commit:
- Builds for Linux x86_64 (glibc and musl)
- Runs full test suite
- Runs clippy linting
- Creates GitHub releases on tags

### Creating a Release

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions will automatically:
1. Build binaries for both targets
2. Run all tests
3. Create a GitHub release
4. Upload binary artifacts

## Translation Notes

Faithful translation from TypeScript to Rust:
- Preserves original algorithm logic and magic constants
- Maintains floating-point arithmetic behavior
- Bug-for-bug compatibility with original
- Enhanced with Rust's type safety and memory guarantees

## License

Same as the original project.
