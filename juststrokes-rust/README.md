# JustStrokes Rust

A Rust implementation of Chinese character (Hanzi) handwriting recognition with Unix socket API for local integration.

## Overview

Bug-to-bug translation of the original TypeScript algorithm, now with a Unix socket service for use as a local API backend. Matches user-drawn strokes against a database of ~9,500 Chinese characters.

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
# Using default paths
./juststrokes-rust

# Custom data file and socket path
./juststrokes-rust graphics.csv /run/user/$UID/handwritten/juststrokes.socket
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

- `serde` - Serialization/deserialization
- `serde_json` - JSON parsing
- `libc` - Unix socket operations

## Translation Notes

Faithful translation from TypeScript to Rust:
- Preserves original algorithm logic and magic constants
- Maintains floating-point arithmetic behavior
- Bug-for-bug compatibility with original
- Enhanced with Rust's type safety and memory guarantees

## License

Same as the original project.
