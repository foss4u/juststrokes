# Technical Q&A

This document answers technical questions about the algorithm implementation.

## Q1: How is the angle calculated and normalized?

### Calculation

```rust
let stroke_span = VectorFunctions::subtract(last_point, first_point);
let stroke_angle = stroke_span[1].atan2(stroke_span[0]);
// Range: [-π, π]
```

### Encoding to 8-bit (0-255)

```rust
let angle_encoded = round(((stroke_angle + π) × 256) / (2π)) % 256;
```

### Mapping Table

| Angle (radians) | Direction | Encoded Value |
|-----------------|-----------|---------------|
| -π | Left (←) | 0 |
| -π/2 | Down (↓) | 64 |
| 0 | Right (→) | 128 |
| π/2 | Up (↑) | 192 |
| π | Left (←) | 0 (wraps) |

### Visual Representation

```
        192 (↑)
          |
          |
128 (→) --+-- 0/256 (←)
          |
          |
        64 (↓)
```

### Circular Distance in Scoring

```rust
let c = abs(input_angle - ref_angle);
let angle_similarity = min(c, 256 - c);
```

**Why `min(c, 256 - c)`?**

Angles wrap around a circle. Examples:
- `0` vs `0`: `min(0, 256) = 0` (identical)
- `0` vs `128`: `min(128, 128) = 128` (opposite)
- `0` vs `255`: `min(255, 1) = 1` (almost identical, wrapping)

---

## Q2: How is length calculated and normalized?

### Calculation

```rust
let stroke_span = subtract(last_point, first_point);
let norm2 = span[0]² + span[1]²;
let length_encoded = round(sqrt(norm2 / 2));
```

### Why Divide by 2?

Compresses the range by factor of √2:
- **Diagonal strokes:** `length ≈ euclidean_distance`
- **Horizontal/Vertical:** `length ≈ 0.707 × euclidean_distance`

### Examples

| Span | Euclidean | Encoded Length |
|------|-----------|----------------|
| [10, 0] | 10.00 | 7 |
| [10, 10] | 14.14 | 10 |
| [100, 0] | 100.00 | 71 |
| [255, 0] | 255.00 | 180 |
| [255, 255] | 360.62 | 255 |

### Overflow Analysis

**Theoretical Maximum:**
- Coordinate space: [0, 255]
- Max diagonal: [255, 255]
- Max length: `round(sqrt((255² + 255²) / 2)) = 255`

**Real Data (9,574 characters, 112,617 strokes):**
- Min: 1
- Max: 248
- Median: 53
- 99th percentile: 173
- **Overflow count: 0**

### Usage in Scoring

```rust
// Length is NOT directly compared
// Instead, it weights angle differences
let lengthy = (input_length + ref_length) / 256;
let angle_penalty = 4 × 4 × lengthy × angle_similarity;
```

**Meaning:** Longer strokes have more weight in angle comparison.

---

## Q3: Does user input overflow when drawing (0,0) → (400,400)?

### Answer: No

### Transformation Pipeline

```
1. Capture Raw Input
   [[0,0], [400,400]] (canvas coordinates)
   ↓
2. Calculate AABB
   [[0,0], [400,400]] (size: 400×400)
   ↓
3. Normalize AABB
   [[0,0], [400,400]] (unchanged, meets constraints)
   ↓
4. Project to [0, 255] Space ← PREVENTS OVERFLOW
   [[0,0], [255,255]] (normalized!)
   Ratio: 255/400 = 0.6375
   ↓
5. Calculate Length
   span = [255, 255]
   length = round(sqrt((255² + 255²) / 2))
   length = 255 ✅
```

### Test Results

| User Input | Projected Span | Length | Status |
|------------|----------------|--------|--------|
| (0,0) → (400,400) | [255, 255] | 255 | ✅ OK |
| (0,0) → (400,0) | [255, 0] | 180 | ✅ OK |
| (0,0) → (0,400) | [0, 255] | 180 | ✅ OK |
| (100,100) → (300,300) | [255, 255] | 255 | ✅ OK |

### Conclusion

Overflow is **mathematically impossible** from user input because:
1. AABB projection happens **before** length calculation
2. All coordinates are normalized to [0, 255]
3. Maximum possible length is 255

---

## Q4: Does projection rounding affect accuracy?

### Answer: Yes, but acceptably

### Rounding Occurs

```rust
// Projection uses round()
let projected_x = round(ratio_x × (x - min_x) + target_min_x);
```

**Maximum Error:** ±0.5 per coordinate

### Impact on Length

**Worst Case Analysis:**

| Span | Perfect Length | With ±1 Error | Max Error |
|------|----------------|---------------|-----------|
| [255, 255] | 255 | 254-256 | ±1 (0.4%) |
| [180, 0] | 127 | 127-128 | ±1 (0.8%) |
| [100, 100] | 100 | 99-101 | ±1 (1.0%) |
| [50, 50] | 50 | 49-51 | ±1 (2.0%) |

### Why Acceptable?

1. **Length only weights angle differences** (not directly compared)
2. **Database has same rounding** (consistent treatment)
3. **Algorithm designed for fuzzy matching** (tolerates small variations)
4. **Error is within tolerance** (±1 unit is negligible)

### Example: Intermediate Points

```
User stroke (diagonal with 9 points):
[0,0], [50,50], [100,100], ..., [400,400]

Projected (exact):
[0.00, 0.00], [31.87, 31.87], [63.75, 63.75], ..., [255.00, 255.00]

Projected (rounded):
[0, 0], [32, 32], [64, 64], ..., [255, 255]

Rounding errors:
Point 0: [0.0000, 0.0000]
Point 1: [-0.1250, -0.1250]
Point 2: [-0.2500, -0.2500]
Point 4: [0.5000, 0.5000]  ← Maximum error
Point 8: [-0.0000, -0.0000]
```

**Key Insight:** Length uses first and last points only. Intermediate point errors don't affect length encoding.

---

## Q5: Why does '內' match '内' with higher score?

### Background

- '內' (traditional) and '内' (simplified) are very similar
- Both have 4 strokes
- Database contains preprocessed data for both

### Data Comparison

```
內 (traditional):
Stroke 0: [38, 72, 50, 128, 46, 188, 38, 248, 192, 124]
Stroke 1: [58, 78, 183, 63, 215, 168, 172, 240, 167, 140]
Stroke 2: [112, 0, 128, 63, 110, 133, 63, 186, 202, 136]
Stroke 3: [130, 118, 147, 134, 167, 149, 179, 170, 161, 51]

内 (simplified):
Stroke 0: [38, 72, 51, 128, 46, 188, 38, 248, 192, 124]  ← diff: 1px
Stroke 1: [58, 77, 184, 63, 216, 170, 169, 240, 168, 139]  ← diff: 1-3px
Stroke 2: [113, 0, 127, 63, 112, 134, 63, 186, 203, 136]  ← diff: 1-2px
Stroke 3: [130, 122, 149, 139, 169, 155, 181, 177, 162, 53]  ← diff: 2-7px
```

### Analysis

Characters differ by only 1-7 pixels per coordinate. When reconstructing from preprocessed data:
1. We lose precision (only 4 sampled points)
2. Reprocessing introduces rounding errors
3. Very similar characters become ambiguous

### Solution

Use `match_preprocessed()` to compare preprocessed data directly:
- No reconstruction
- No reprocessing
- No additional rounding errors
- **Result:** 100% success rate (9,574/9,574)

### Lesson

The database contains **already preprocessed** data. Testing should use it directly, not reconstruct and reprocess.

---

## Q6: What are the magic constants?

### NUM_POSSIBLE_ENCODED_VALUE = 256

- Coordinate space size after normalization
- Chosen for 8-bit encoding efficiency
- Allows angle to fit in single byte

### NUM_ENCODED_POINTS = 4

- Number of points sampled per stroke
- Balance between accuracy and performance
- Fewer points = faster matching, less precision
- More points = slower matching, more precision

### MAGIC_PER_STROKE_WEIGHT = 4.0

- Weight for angle penalty in scoring
- Empirically determined in original algorithm
- Balances point position vs angle importance
- Higher value = angles matter more

### Formula

```rust
angle_penalty = 4.0 × 4 × lengthy × angle_similarity
              = 16 × lengthy × angle_similarity
```

Where:
- `lengthy` = average stroke length / 256
- `angle_similarity` = circular angle difference

---

## Q7: How does the similarity scoring work?

### Scoring Formula

```rust
score = 0.0;

for each stroke:
    // Penalize point position differences
    for each of 4 sampled points:
        score -= abs(input_x - ref_x)
        score -= abs(input_y - ref_y)
    
    // Penalize angle difference (weighted by length)
    angle_diff = circular_distance(input_angle, ref_angle)
    avg_length = (input_length + ref_length) / 256
    score -= 16 × avg_length × angle_diff

return score  // Higher is better (less negative)
```

### Components

1. **Point Position Penalty:** Direct coordinate differences
2. **Angle Penalty:** Circular distance, weighted by stroke length
3. **No Length Penalty:** Length only used as weight

### Example

```
Input stroke: [0,0], [100,100], [200,200], [255,255]
Reference stroke: [0,0], [100,100], [200,200], [255,255]

Point penalties: 0 (identical)
Angle penalty: 0 (identical)
Total score: 0 (perfect match)
```

```
Input stroke: [0,0], [100,100], [200,200], [255,255]
Reference stroke: [10,10], [110,110], [210,210], [245,245]

Point penalties: 
  - Point 0: abs(0-10) + abs(0-10) = 20
  - Point 1: abs(100-110) + abs(100-110) = 20
  - Point 2: abs(200-210) + abs(200-210) = 20
  - Point 3: abs(255-245) + abs(255-245) = 20
  Total: -80

Angle penalty: ~0 (same direction)
Total score: -80
```

### Interpretation

- **Score = 0:** Perfect match
- **Score < 0:** Some differences (more negative = less similar)
- **Comparison:** Higher score = better match

---

*See also: [03-ALGORITHM-DETAILS.md](03-ALGORITHM-DETAILS.md)*
