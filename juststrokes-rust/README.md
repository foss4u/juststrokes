# JustStrokes Rust

A Rust implementation of the Chinese character (Hanzi) handwriting recognition algorithm, translated from the TypeScript version.

## Overview

This is a bug-to-bug translation of the original TypeScript/JavaScript handwriting recognition algorithm. It matches user-drawn strokes against a database of ~9,500 Chinese characters.

## Features

- **Stroke-based matching**: Recognizes characters based on stroke order and shape
- **Preprocessed database**: Uses graphics.json with preprocessed character data
- **High accuracy**: 100% test pass rate on database self-matching
- **Rust 2024 edition**: Uses latest Rust features and idioms

## Algorithm

The recognition works in several steps:

1. **AABB Normalization**: Calculate bounding box and normalize to [0, 255] space
2. **Stroke Downsampling**: Resample each stroke to exactly 4 points
3. **Feature Encoding**: Encode angle and length for each stroke
4. **Similarity Scoring**: Compare input against database using point distances and angle differences

## Usage

```rust
use juststrokes_rust::{Matcher, Stroke, data::load_graphics_json};

// Load character database
let data = load_graphics_json("graphics.json")?;

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

## Testing

The test suite validates that every character in the database matches itself with the highest score:

```bash
cargo test
```

All 9,574 characters pass the self-matching test.

## Building

```bash
cargo build --release
```

## Dependencies

- `serde` - Serialization/deserialization
- `serde_json` - JSON parsing for graphics.json

## Translation Notes

This is a faithful translation from TypeScript to Rust, preserving:
- Original variable names (including typos like "point_cadidate")
- Algorithm logic and magic constants
- Floating-point arithmetic behavior
- Bug-for-bug compatibility

The main difference is that Rust's type system provides additional safety guarantees.

## License

Same as the original project.
