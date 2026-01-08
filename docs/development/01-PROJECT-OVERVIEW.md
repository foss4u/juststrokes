# Project Overview

## Introduction

JustStrokes Rust is a high-performance handwriting recognition system for Chinese characters (Hanzi), translated from the original TypeScript implementation with additional features for production use.

## Project Goals

### Primary Goals
1. **Bug-to-bug compatibility** with TypeScript implementation
2. **100% test coverage** on character database (9,574 characters)
3. **Production-ready** Unix socket service
4. **Optimized binaries** for Linux deployment

### Secondary Goals
1. Performance improvements where possible
2. Better documentation and code clarity
3. Multiple data format support (JSON, CSV)
4. Portable static binaries

## Requirements

### Functional Requirements
- Recognize Chinese characters from stroke input
- Match against database of 9,574 characters
- Return top N candidates ranked by similarity
- Support raw stroke input (unpreprocessed)

### Non-Functional Requirements
- **Performance:** < 2 seconds for full database scan
- **Accuracy:** 100% self-matching on database
- **Binary Size:** < 1MB stripped
- **Memory:** Reasonable for embedded systems
- **Portability:** Static linking option

### Technical Requirements
- Rust 2024 edition
- Latest stable dependencies
- Zero clippy warnings
- Formatted with cargo fmt
- Comprehensive test suite

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Client Application                    │
│                  (Handwriting Input UI)                  │
└────────────────────────┬────────────────────────────────┘
                         │ Unix Socket
                         │ (CSV Protocol)
┌────────────────────────▼────────────────────────────────┐
│                  Unix Socket Service                     │
│  ┌────────────────────────────────────────────────────┐ │
│  │              Request Handler                       │ │
│  │  • Parse CSV input                                 │ │
│  │  • Convert to Stroke format                        │ │
│  │  • Call matcher                                    │ │
│  │  • Format CSV output                               │ │
│  └────────────────────┬───────────────────────────────┘ │
└─────────────────────────┼─────────────────────────────────┘
                          │
┌─────────────────────────▼─────────────────────────────────┐
│                    Matcher Core                           │
│  ┌──────────────────────────────────────────────────┐    │
│  │           Preprocessing Pipeline                 │    │
│  │  1. Calculate AABB                               │    │
│  │  2. Normalize to [0, 255]                        │    │
│  │  3. Resample to 4 points                         │    │
│  │  4. Encode angle & length                        │    │
│  └──────────────────────┬───────────────────────────┘    │
│                         │                                 │
│  ┌──────────────────────▼───────────────────────────┐    │
│  │           Similarity Scoring                     │    │
│  │  • Compare point positions                       │    │
│  │  • Calculate angle difference                    │    │
│  │  • Weight by stroke length                       │    │
│  └──────────────────────┬───────────────────────────┘    │
│                         │                                 │
│  ┌──────────────────────▼───────────────────────────┐    │
│  │           Candidate Ranking                      │    │
│  │  • Sort by score (higher = better)               │    │
│  │  • Return top N candidates                       │    │
│  └──────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────┘
                          │
┌─────────────────────────▼─────────────────────────────────┐
│                  Character Database                       │
│  • 9,574 characters                                       │
│  • Preprocessed stroke data                               │
│  • JSON or CSV format                                     │
└───────────────────────────────────────────────────────────┘
```

### Module Structure

```
juststrokes-rust/
├── src/
│   ├── lib.rs              # Core algorithm
│   ├── data.rs             # JSON data loading
│   ├── csv_data.rs         # CSV data loading
│   ├── socket_service.rs   # Unix socket service
│   └── main.rs             # Binary entry point
├── tests/
│   ├── integration_test.rs # Full database tests
│   └── debug_test.rs       # Debug utilities
├── benches/
│   └── matching_bench.rs   # Performance benchmarks
└── docs/
    └── development/        # This documentation
```

## Technology Stack

### Core Dependencies
- **serde** (1.0) - Serialization framework
- **serde_json** (1.0) - JSON parsing
- **libc** (0.2) - Unix socket operations

### Development Dependencies
- **chrono** (0.4) - Timestamp generation for benchmarks

### Build Tools
- **rustc** 1.92.0 (Rust 2024 edition)
- **cargo** - Build system and package manager
- **clippy** - Linting
- **rustfmt** - Code formatting

## Data Flow

### Recognition Pipeline

```
User Input (Raw Strokes)
    │
    ├─→ [AABB Calculation]
    │       │
    │       └─→ Bounding box: [[x_min, y_min], [x_max, y_max]]
    │
    ├─→ [AABB Normalization]
    │       │
    │       ├─→ Apply min_width constraint (8px)
    │       ├─→ Apply aspect_ratio constraint (1:1)
    │       └─→ Expanded AABB
    │
    ├─→ [Coordinate Projection]
    │       │
    │       └─→ Map to [0, 255] space
    │
    ├─→ [Stroke Resampling]
    │       │
    │       ├─→ Calculate arc length
    │       ├─→ Sample 4 evenly-spaced points
    │       └─→ Linear interpolation
    │
    ├─→ [Feature Encoding]
    │       │
    │       ├─→ Angle: atan2(dy, dx) → [0, 256)
    │       └─→ Length: sqrt(dx² + dy²) / √2
    │
    └─→ [Similarity Matching]
            │
            ├─→ For each database character:
            │   ├─→ Compare point positions
            │   ├─→ Compare angles (circular)
            │   └─→ Weight by length
            │
            └─→ Return top N candidates
```

## Key Design Decisions

### 1. Bug-to-Bug Translation

**Decision:** Preserve original algorithm exactly, including quirks.

**Rationale:**
- Ensures compatibility with existing data
- Avoids introducing new bugs
- Allows direct comparison with TypeScript version

**Examples:**
- Keep variable name `point_candidate` (typo in original)
- Preserve magic constant `MAGIC_PER_STROKE_WEIGHT = 4.0`
- Maintain exact floating-point arithmetic

### 2. Preprocessed Data Format

**Decision:** Use preprocessed stroke data directly in tests.

**Rationale:**
- Database contains already-processed strokes
- Reprocessing introduces rounding errors
- Direct comparison is more accurate

**Impact:**
- Added `match_preprocessed()` method
- 100% test success rate achieved

### 3. Unix Socket API

**Decision:** Use CSV protocol over Unix sockets.

**Rationale:**
- Simple, human-readable format
- Easy to debug with standard tools
- No binary protocol complexity
- Language-agnostic

**Alternative Considered:** Binary protocol (rejected for complexity)

### 4. Multiple Data Formats

**Decision:** Support both JSON and CSV.

**Rationale:**
- JSON: Standard format, easy to generate
- CSV: 29% smaller, faster parsing
- Let users choose based on needs

### 5. Static Linking Option

**Decision:** Provide both glibc and musl builds.

**Rationale:**
- glibc: Smaller, standard on most systems
- musl: Portable, works on any Linux
- Users choose based on deployment needs

## Success Criteria

### Must Have
- ✅ All 9,574 characters pass self-matching test
- ✅ Zero clippy warnings
- ✅ Formatted with cargo fmt
- ✅ Unix socket service functional
- ✅ Optimized binaries < 1MB

### Should Have
- ✅ Performance improvement over baseline
- ✅ Comprehensive documentation
- ✅ CSV format support
- ✅ Build automation

### Nice to Have
- ✅ Static binary option
- ✅ Detailed technical analysis
- ✅ Benchmark infrastructure

## Project Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Initial Conversion | 2 hours | ✅ Complete |
| Performance Optimization | 1 hour | ✅ Complete |
| Code Quality | 30 min | ✅ Complete |
| CSV Format | 30 min | ✅ Complete |
| Unix Socket Service | 1 hour | ✅ Complete |
| Build System | 30 min | ✅ Complete |
| Documentation | 1 hour | ✅ Complete |
| **Total** | **6.5 hours** | ✅ **Complete** |

## Deliverables

### Code
- ✅ Core library (`src/lib.rs`)
- ✅ Data loaders (JSON, CSV)
- ✅ Unix socket service
- ✅ Main binary
- ✅ Test suite (10 tests)
- ✅ Benchmark suite

### Binaries
- ✅ Linux x86_64 glibc (506KB)
- ✅ Linux x86_64 musl (593KB)

### Documentation
- ✅ README with usage examples
- ✅ API documentation (inline)
- ✅ Development documentation (this)
- ✅ Build instructions

### Data
- ✅ graphics.json (5.5MB)
- ✅ graphics.csv (3.9MB)

## Next Steps

### Potential Enhancements
1. **WebAssembly target** - Run in browser
2. **GPU acceleration** - Parallel matching
3. **Incremental matching** - Match as user draws
4. **Character variants** - Support traditional/simplified
5. **Stroke order validation** - Penalize wrong order

### Maintenance
1. Keep dependencies updated
2. Monitor performance regressions
3. Add more test cases
4. Improve documentation

---

*See [02-INITIAL-CONVERSION.md](02-INITIAL-CONVERSION.md) for implementation details.*
