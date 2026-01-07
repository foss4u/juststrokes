use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

pub mod csv_data;
pub mod data;

/// 2D point in canvas coordinate space
pub type Point = [f64; 2];

/// Sequence of points forming a single stroke
pub type Stroke = Vec<Point>;

/// Axis-aligned bounding box represented as [min_corner, max_corner]
pub type AABB = [Point; 2];

/// Preprocessed stroke data: 4 sampled points (8 coordinates) + angle + length = 10 values
/// Format: [x0, y0, x1, y1, x2, y2, x3, y3, angle_encoded, length_encoded]
pub type StrokeProcessed = Vec<f64>;

/// Chinese character (CJK ideograph)
pub type Ideograph = String;

/// Coordinate space size for normalization (0-255)
const NUM_POSSIBLE_ENCODED_VALUE: usize = 256;

/// Number of points to sample per stroke during preprocessing
const NUM_ENCODED_POINTS: usize = 4;

/// Utility functions for 2D vector operations
struct VectorFunctions;

impl VectorFunctions {
    /// Squared Euclidean distance between two points
    fn distance2(p0: Point, p1: Point) -> f64 {
        Self::norm2(Self::subtract(p0, p1))
    }

    /// Squared magnitude of a vector (x² + y²)
    fn norm2(p: Point) -> f64 {
        p[0] * p[0] + p[1] * p[1]
    }

    /// Round coordinates to nearest integer values
    fn round(p: Point) -> Point {
        [p[0].round(), p[1].round()]
    }

    /// Vector subtraction (p0 - p1)
    fn subtract(p0: Point, p1: Point) -> Point {
        [p0[0] - p1[0], p0[1] - p1[1]]
    }
}

/// Create affine transformation to map points from source AABB to target AABB
/// Returns a closure that performs the coordinate transformation with rounding
fn create_normalized_project_function(aabb0: AABB, aabb1: AABB) -> impl Fn(Point) -> Point {
    let diff0 = VectorFunctions::subtract(aabb0[1], aabb0[0]);
    let diff1 = VectorFunctions::subtract(aabb1[1], aabb1[0]);
    let diffratio = [diff1[0] / diff0[0], diff1[1] / diff0[1]];
    let aabb0_min = aabb0[0];
    let aabb1_min = aabb1[0];

    move |point: Point| -> Point {
        [
            (diffratio[0] * (point[0] - aabb0_min[0]) + aabb1_min[0]).round(),
            (diffratio[1] * (point[1] - aabb0_min[1]) + aabb1_min[1]).round(),
        ]
    }
}

/// Compute axis-aligned bounding box encompassing all stroke points
fn get_aabb(strokes: &[Stroke]) -> AABB {
    let mut p_min: Point = [f64::INFINITY, f64::INFINITY];
    let mut p_max: Point = [f64::NEG_INFINITY, f64::NEG_INFINITY];

    for stroke in strokes {
        for point in stroke {
            p_min[0] = p_min[0].min(point[0]);
            p_min[1] = p_min[1].min(point[1]);
            p_max[0] = p_max[0].max(point[0]);
            p_max[1] = p_max[1].max(point[1]);
        }
    }

    [p_min, p_max]
}

/// Resample stroke to exactly N evenly-spaced points along its path
/// Uses linear interpolation to maintain stroke shape while reducing point count
fn process_stroke(stroke: &Stroke, how_many_points_to_sample: usize) -> Stroke {
    let mut result_stroke: Stroke = Vec::new();
    let mut stroke_length = 0.0;

    // Compute total arc length of the stroke
    for i in 0..stroke.len() - 1 {
        stroke_length += VectorFunctions::distance2(stroke[i], stroke[i + 1]).sqrt();
    }

    let mut h = 0; // Current segment index
    let mut point_candidate = stroke[0]; // Current interpolation position
    let mut u = 0.0; // Distance traveled so far

    // Generate N-1 evenly-spaced sample points (last point added separately)
    for i in 0..how_many_points_to_sample - 1 {
        let s = (i as f64 * stroke_length) / (how_many_points_to_sample - 1) as f64;
        while s > u {
            let c = VectorFunctions::distance2(point_candidate, stroke[h + 1]).sqrt();
            if s > u + c {
                // Target is beyond current segment, advance to next
                h += 1;
                point_candidate = stroke[h];
                u += c;
            } else {
                // Interpolate within current segment
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
    // Always include the original last point to preserve stroke endpoint
    result_stroke.push(stroke[stroke.len() - 1]);
    result_stroke
}

/// Expand AABB to satisfy minimum size and aspect ratio constraints
/// Ensures bounding box is large enough and not too elongated for normalization
fn normalize_aabb(mut aabb: AABB, max_ratio: f64, min_width: f64) -> AABB {
    aabb[0] = VectorFunctions::round(aabb[0]);
    aabb[1] = VectorFunctions::round(aabb[1]);

    let mut e = VectorFunctions::subtract(aabb[1], aabb[0]);
    if e[0] < 0.0 || e[1] < 0.0 {
        panic!("Invalid AABB: {:?}", aabb);
    }

    // Expand to minimum width if needed (prevents division by zero)
    if e[0] < min_width {
        let i = ((min_width - e[0]) / 2.0).ceil();
        aabb[0][0] -= i;
        aabb[1][0] += i;
    }
    if e[1] < min_width {
        let i = ((min_width - e[1]) / 2.0).ceil();
        aabb[0][1] -= i;
        aabb[1][1] += i;
    }

    // Enforce aspect ratio constraint (prevents extreme elongation)
    if max_ratio > 0.0 {
        e = VectorFunctions::subtract(aabb[1], aabb[0]);
        if e[0] < e[1] / max_ratio {
            // Too tall, expand width
            let i = ((e[1] / max_ratio - e[0]) / 2.0).ceil();
            aabb[0][0] -= i;
            aabb[1][0] += i;
        } else if e[1] < e[0] / max_ratio {
            // Too wide, expand height
            let i = ((e[0] / max_ratio - e[1]) / 2.0).ceil();
            aabb[0][1] -= i;
            aabb[1][1] += i;
        }
    }

    aabb
}

/// Transform raw strokes into normalized feature vectors for matching
/// Steps: normalize coordinates → resample → encode angle and length
fn preprocess_strokes(strokes: &[Stroke], opts: &MatcherOptions) -> Vec<StrokeProcessed> {
    if strokes.is_empty() || strokes.iter().any(|s| s.is_empty()) {
        panic!("Invalid stroke data: empty strokes not allowed");
    }

    let side_length = NUM_POSSIBLE_ENCODED_VALUE as f64;
    let aabb_after = normalize_aabb(get_aabb(strokes), opts.max_ratio, opts.min_width);
    let target_aabb: AABB = [[0.0, 0.0], [255.0, 255.0]];
    let project = create_normalized_project_function(aabb_after, target_aabb);

    strokes
        .iter()
        .map(|stroke| {
            // Transform to normalized [0, 255] coordinate space
            let projected: Stroke = stroke.iter().map(|&p| project(p)).collect();
            let stroke_processed = process_stroke(&projected, NUM_ENCODED_POINTS);

            // Compute stroke direction vector (first point to last point)
            let stroke_span = VectorFunctions::subtract(
                stroke_processed[stroke_processed.len() - 1],
                stroke_processed[0],
            );

            // Encode stroke angle as integer in range [0, 256)
            let stroke_angle = stroke_span[1].atan2(stroke_span[0]);
            let angle_encoded =
                (((stroke_angle + PI) * side_length) / (2.0 * PI)).round() as i32 % 256;

            // Encode stroke length (scaled by 1/√2 for normalization)
            let length_encoded = (VectorFunctions::norm2(stroke_span) / 2.0).sqrt().round() as i32;

            // Flatten sampled points and append encoded features
            let mut result: StrokeProcessed = stroke_processed.into_iter().flatten().collect();
            result.push(angle_encoded as f64);
            result.push(length_encoded as f64);
            result
        })
        .collect()
}

/// Compute similarity score between two stroke sequences (higher = more similar)
/// Combines point position differences with angle and length-weighted penalties
#[inline]
fn score_similarity(input: &[StrokeProcessed], reference: &[StrokeProcessed]) -> f64 {
    let mut score = 0.0;
    const MAGIC_PER_STROKE_WEIGHT: f64 = 4.0;
    const NUM_ENCODED_POINTS_F64: f64 = NUM_ENCODED_POINTS as f64;
    const NUM_POSSIBLE_ENCODED_VALUE_F64: f64 = NUM_POSSIBLE_ENCODED_VALUE as f64;

    for i in 0..input.len() {
        let input_stroke = &input[i];
        let ref_stroke = &reference[i];

        // Penalize coordinate differences for each sampled point
        for s in 0..NUM_ENCODED_POINTS {
            let idx = 2 * s;
            score -= (input_stroke[idx] - ref_stroke[idx]).abs();
            score -= (input_stroke[idx + 1] - ref_stroke[idx + 1]).abs();
        }

        // Penalize angle difference (using circular distance for wraparound)
        let angle_idx = 2 * NUM_ENCODED_POINTS;
        let c = (input_stroke[angle_idx] - ref_stroke[angle_idx]).abs();
        let angle_similarity = c.min(NUM_POSSIBLE_ENCODED_VALUE_F64 - c);

        // Scale angle penalty by average stroke length (longer strokes matter more)
        let length_idx = angle_idx + 1;
        let lengthy =
            (input_stroke[length_idx] + ref_stroke[length_idx]) / NUM_POSSIBLE_ENCODED_VALUE_F64;

        score -= MAGIC_PER_STROKE_WEIGHT * NUM_ENCODED_POINTS_F64 * lengthy * angle_similarity;
    }

    score
}

/// Matcher configuration options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MatcherOptions {
    pub max_ratio: f64,
    pub min_width: f64,
}

impl Default for MatcherOptions {
    fn default() -> Self {
        Self {
            max_ratio: 1.0,
            min_width: 8.0,
        }
    }
}

/// Main matcher for handwriting recognition
pub struct Matcher {
    params: MatcherOptions,
    medians: Vec<(Ideograph, Vec<StrokeProcessed>)>,
}

impl Matcher {
    /// Create a new matcher with character database
    pub fn new(
        medians: Vec<(Ideograph, Vec<StrokeProcessed>)>,
        options: Option<MatcherOptions>,
    ) -> Self {
        Self {
            medians,
            params: options.unwrap_or_default(),
        }
    }

    /// Preprocess user input strokes
    #[inline]
    pub fn preprocess(&self, strokes: &[Stroke]) -> Vec<StrokeProcessed> {
        preprocess_strokes(strokes, &self.params)
    }

    /// Match input strokes against database and return top candidates
    pub fn match_strokes(&self, strokes: &[Stroke], how_many_candidates: usize) -> Vec<Ideograph> {
        if strokes.is_empty() {
            return Vec::new();
        }

        let mut candidates: Vec<Ideograph> = Vec::new();
        let mut scores: Vec<f64> = Vec::new();
        let strokes2 = self.preprocess(strokes);

        // Compare against all characters in database
        for candidate in &self.medians {
            if candidate.1.len() == strokes2.len() {
                let score = score_similarity(&strokes2, &candidate.1);

                // Insert in sorted order (higher scores first)
                let mut f = scores.len();
                while f > 0 && score > scores[f - 1] {
                    f -= 1;
                }

                if how_many_candidates > f {
                    candidates.insert(f, candidate.0.clone());
                    scores.insert(f, score);
                    if candidates.len() > how_many_candidates {
                        candidates.pop();
                        scores.pop();
                    }
                }
            }
        }

        candidates
    }

    /// Match preprocessed strokes directly (for testing)
    pub fn match_preprocessed(
        &self,
        strokes_processed: &[StrokeProcessed],
        how_many_candidates: usize,
    ) -> Vec<Ideograph> {
        if strokes_processed.is_empty() {
            return Vec::new();
        }

        let mut candidates: Vec<Ideograph> = Vec::new();
        let mut scores: Vec<f64> = Vec::new();

        // Compare against all characters in database
        for candidate in &self.medians {
            if candidate.1.len() == strokes_processed.len() {
                let score = score_similarity(strokes_processed, &candidate.1);

                // Insert in sorted order (higher scores first)
                let mut f = scores.len();
                while f > 0 && score > scores[f - 1] {
                    f -= 1;
                }

                if how_many_candidates > f {
                    candidates.insert(f, candidate.0.clone());
                    scores.insert(f, score);
                    if candidates.len() > how_many_candidates {
                        candidates.pop();
                        scores.pop();
                    }
                }
            }
        }

        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_functions() {
        let p1: Point = [0.0, 0.0];
        let p2: Point = [3.0, 4.0];

        assert_eq!(VectorFunctions::distance2(p1, p2), 25.0);
        assert_eq!(VectorFunctions::norm2(p2), 25.0);
        assert_eq!(VectorFunctions::subtract(p2, p1), [3.0, 4.0]);
        assert_eq!(VectorFunctions::round([3.7, 4.2]), [4.0, 4.0]);
    }

    #[test]
    fn test_get_aabb() {
        let strokes = vec![
            vec![[0.0, 0.0], [10.0, 10.0]],
            vec![[5.0, 5.0], [15.0, 20.0]],
        ];
        let aabb = get_aabb(&strokes);
        assert_eq!(aabb[0], [0.0, 0.0]);
        assert_eq!(aabb[1], [15.0, 20.0]);
    }
}
