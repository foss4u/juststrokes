# Initial Conversion: TypeScript to Rust

## Overview

This document details the process of converting the TypeScript handwriting recognition algorithm to Rust while maintaining bug-to-bug compatibility.

## Conversion Strategy

### Principles

1. **Preserve Logic:** Keep algorithm exactly as-is
2. **Type Safety:** Leverage Rust's type system
3. **No Optimization:** Focus on correctness first
4. **Test-Driven:** Validate every step

### Approach

```
TypeScript Source
    │
    ├─→ Analyze types and data structures
    ├─→ Map to Rust equivalents
    ├─→ Translate function by function
    ├─→ Add tests
    └─→ Verify correctness
```

## Type Mappings

### Basic Types

| TypeScript | Rust | Notes |
|------------|------|-------|
| `number` | `f64` | Floating-point arithmetic |
| `[number, number]` | `[f64; 2]` | Fixed-size array |
| `number[]` | `Vec<f64>` | Dynamic array |
| `string` | `String` | Owned string |
| `Array<T>` | `Vec<T>` | Generic vector |

### Custom Types

```typescript
// TypeScript
type Point = [number, number];
type Stroke = Point[];
type AABB = [Point, Point];
type StrokeProcessed = number[];
type Ideograph = string;
```

```rust
// Rust
pub type Point = [f64; 2];
pub type Stroke = Vec<Point>;
pub type AABB = [Point; 2];
pub type StrokeProcessed = Vec<f64>;
pub type Ideograph = String;
```

## Function Translation

### Example 1: Vector Operations

**TypeScript:**
```typescript
const VectorFunctions = {
  distance2(p0: Point, p1: Point) {
    return VectorFunctions.norm2(VectorFunctions.subtract(p0, p1))
  },
  norm2(p: Point) {
    return p[0] * p[0] + p[1] * p[1]
  },
  subtract(p0: Point, p1: Point): Point {
    return [p0[0] - p1[0], p0[1] - p1[1]]
  }
}
```

**Rust:**
```rust
struct VectorFunctions;

impl VectorFunctions {
    fn distance2(p0: Point, p1: Point) -> f64 {
        Self::norm2(Self::subtract(p0, p1))
    }

    fn norm2(p: Point) -> f64 {
        p[0] * p[0] + p[1] * p[1]
    }

    fn subtract(p0: Point, p1: Point) -> Point {
        [p0[0] - p1[0], p0[1] - p1[1]]
    }
}
```

**Changes:**
- Object → Struct with impl block
- Arrow functions → Associated functions
- Implicit return → Explicit return

### Example 2: Stroke Processing

**TypeScript:**
```typescript
function process_stroke(stroke: Stroke, how_many_points_to_sample: number): Stroke {
  var result_stroke: Stroke = []
  var stroke_length = 0
  
  for (var i = 0; i < stroke.length - 1; i++)
    stroke_length += Math.sqrt(VectorFunctions.distance2(stroke[i], stroke[i + 1]))
  
  var h = 0, point_candidate = stroke[0], u = 0
  
  for (var i = 0; how_many_points_to_sample - 1 > i; i++) {
    for (var s = (i * stroke_length) / (how_many_points_to_sample - 1); s > u;) {
      var c = Math.sqrt(VectorFunctions.distance2(point_candidate, stroke[h + 1]))
      if (s > u + c) {
        h += 1
        point_candidate = stroke[h]
        u += c
      } else {
        var f = (s - u) / c
        point_candidate = [
          (1 - f) * point_candidate[0] + f * stroke[h + 1][0],
          (1 - f) * point_candidate[1] + f * stroke[h + 1][1],
        ]
        u = s
      }
    }
    result_stroke.push(VectorFunctions.round(point_candidate))
  }
  result_stroke.push(stroke[stroke.length - 1])
  return result_stroke
}
```

**Rust:**
```rust
fn process_stroke(stroke: &Stroke, how_many_points_to_sample: usize) -> Stroke {
    let mut result_stroke: Stroke = Vec::new();
    let mut stroke_length = 0.0;

    for i in 0..stroke.len() - 1 {
        stroke_length += VectorFunctions::distance2(stroke[i], stroke[i + 1]).sqrt();
    }

    let mut h = 0;
    let mut point_candidate = stroke[0];
    let mut u = 0.0;

    for i in 0..how_many_points_to_sample - 1 {
        let s = (i as f64 * stroke_length) / (how_many_points_to_sample - 1) as f64;
        while s > u {
            let c = VectorFunctions::distance2(point_candidate, stroke[h + 1]).sqrt();
            if s > u + c {
                h += 1;
                point_candidate = stroke[h];
                u += c;
            } else {
                let f = (s - u) / c;
                point_candidate = [
                    (1.0 - f) * point_candidate[0] + f * stroke[h + 1][0],
                    (1.0 - f) * point_candidate[1] + f * stroke[h + 1][1],
                ];
                u = s;
            }
        }
        result_stroke.push(VectorFunctions::round(point_candidate));
    }
    result_stroke.push(stroke[stroke.len() - 1]);
    result_stroke
}
```

**Changes:**
- `var` → `let mut` (explicit mutability)
- `number` → `usize` for array indices
- Explicit type casts (`as f64`, `as usize`)
- Borrow checker: `&Stroke` parameter
- Explicit `.0` for floating literals

### Example 3: Matcher Class

**TypeScript:**
```typescript
export class Matcher {
  private _params: MatcherOptions
  private _medians: Array<[Ideograph, StrokeProcessed[]]>

  constructor(medians: Array<[Ideograph, StrokeProcessed[]]>, options?: Partial<MatcherOptions>) {
    this._medians = medians
    this._params = {
      max_ratio: 1,
      min_width: 8,
      ...options
    }
  }

  match(strokes: Stroke[], how_many_candidates = 1) {
    if (0 === strokes.length) return []
    let candidates: Ideograph[] = []
    let scores: number[] = []
    const strokes2 = this.preprocess(strokes)
    // ... matching logic
    return candidates
  }
}
```

**Rust:**
```rust
pub struct Matcher {
    params: MatcherOptions,
    medians: Vec<(Ideograph, Vec<StrokeProcessed>)>,
}

impl Matcher {
    pub fn new(
        medians: Vec<(Ideograph, Vec<StrokeProcessed>)>,
        options: Option<MatcherOptions>,
    ) -> Self {
        Self {
            medians,
            params: options.unwrap_or_default(),
        }
    }

    pub fn match_strokes(&self, strokes: &[Stroke], how_many_candidates: usize) -> Vec<Ideograph> {
        if strokes.is_empty() {
            return Vec::new();
        }
        let mut candidates: Vec<Ideograph> = Vec::new();
        let mut scores: Vec<f64> = Vec::new();
        let strokes2 = self.preprocess(strokes);
        // ... matching logic
        candidates
    }
}
```

**Changes:**
- Private fields don't need `_` prefix in Rust
- `Partial<T>` → `Option<T>`
- Spread operator → `unwrap_or_default()`
- `Array<[A, B]>` → `Vec<(A, B)>` (tuple)
- Default parameters → explicit `usize` parameter

## Data Loading

### JSON Parsing

**TypeScript:**
```typescript
const data = await (await fetch('graphics.json')).json()
```

**Rust:**
```rust
use serde_json::Value;
use std::fs;

let content = fs::read_to_string("graphics.json")?;
let data: Value = serde_json::from_str(&content)?;
```

**Challenges:**
- No async/await needed (file I/O)
- Explicit error handling with `?`
- Type annotations required

### Data Structure Parsing

**TypeScript:**
```typescript
for (const [character, strokes] of data) {
  // character: string
  // strokes: number[][]
}
```

**Rust:**
```rust
if let Value::Array(entries) = data {
    for entry in entries {
        if let Value::Array(pair) = entry
            && pair.len() == 2
        {
            let character = pair[0].as_str()?.to_string();
            let strokes = /* parse strokes */;
        }
    }
}
```

**Challenges:**
- Pattern matching required
- Explicit type conversions
- Error handling at each step

## Testing Strategy

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_functions() {
        let p1: Point = [0.0, 0.0];
        let p2: Point = [3.0, 4.0];
        assert_eq!(VectorFunctions::distance2(p1, p2), 25.0);
    }
}
```

### Integration Tests

**File:** `tests/integration_test.rs`

```rust
use juststrokes_rust::{data::load_graphics_json, Matcher};

#[test]
fn test_all_characters_match_themselves() {
    let data = load_graphics_json("graphics.json")
        .expect("Failed to load graphics.json");
    let matcher = Matcher::new(data.clone(), None);

    for (expected_char, strokes_processed) in &data {
        let candidates = matcher.match_preprocessed(strokes_processed, 5);
        assert!(!candidates.is_empty());
        assert_eq!(&candidates[0], expected_char);
    }
}
```

## Challenges and Solutions

### Challenge 1: Preprocessed Data

**Problem:** Tests failed because we reconstructed strokes from preprocessed data, then reprocessed them.

**Solution:** Added `match_preprocessed()` method to use preprocessed data directly.

```rust
pub fn match_preprocessed(
    &self,
    strokes_processed: &[StrokeProcessed],
    how_many_candidates: usize,
) -> Vec<Ideograph> {
    // Compare preprocessed data directly
}
```

### Challenge 2: Borrow Checker

**Problem:** Cannot move out of borrowed content.

```rust
// Error: cannot move out of `stream`
let mut reader = BufReader::new(&stream);
let mut writer = stream; // Error!
```

**Solution:** Use scoping to drop borrow.

```rust
let mut line = String::new();
{
    let mut reader = BufReader::new(&stream);
    reader.read_line(&mut line)?;
} // reader dropped here
// Now can use stream
stream.write_all(response.as_bytes())?;
```

### Challenge 3: Floating Point Comparisons

**Problem:** Direct equality checks fail due to precision.

```rust
// May fail
assert_eq!(calculated_value, expected_value);
```

**Solution:** Use epsilon comparison.

```rust
assert!((calculated_value - expected_value).abs() < 0.0001);
```

### Challenge 4: Mutable vs Immutable

**Problem:** TypeScript `var` is always mutable, Rust requires explicit `mut`.

**Solution:** Analyze variable usage and add `mut` where needed.

```rust
let mut h = 0;  // Modified in loop
let stroke_length = 0.0;  // Never modified after initialization
```

## Verification

### Correctness Checks

1. **Unit Tests:** Test individual functions
2. **Integration Tests:** Test full pipeline
3. **Self-Matching:** All 9,574 characters match themselves
4. **Clippy:** Zero warnings
5. **Formatting:** Consistent style

### Test Results

```
running 10 tests
test csv_data::tests::test_csv_loading ... ok
test csv_data::tests::test_json_to_csv_conversion ... ok
test data::tests::test_load_graphics_json ... ok
test socket_service::tests::test_socket_service ... ok
test tests::test_get_aabb ... ok
test tests::test_vector_functions ... ok
test integration_test::test_all_characters_match_themselves ... ok
test integration_test::test_different_stroke_counts ... ok
test integration_test::test_sample_characters ... ok
test debug_test::debug_nei_character ... ok

test result: ok. 10 passed; 0 failed
```

## Commit

**SHA:** `df0137b`

**Message:**
```
Add Rust implementation of handwriting recognition algorithm

- Bug-to-bug translation from TypeScript to Rust
- Implements core matching algorithm with AABB normalization,
  stroke downsampling, and similarity scoring
- Uses preprocessed graphics.json data directly for testing
- All 9,574 characters pass self-matching test (100% success rate)
- Rust 2024 edition with latest stable dependencies
- Passes cargo clippy with no warnings
- Formatted with cargo fmt

Co-authored-by: Ona <no-reply@ona.com>
```

---

*Next: [03-ALGORITHM-DETAILS.md](03-ALGORITHM-DETAILS.md)*
