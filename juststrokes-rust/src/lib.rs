use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

pub mod data;

/// A 2D point with integer coordinates
pub type Point = [f64; 2];

/// A stroke is a sequence of points
pub type Stroke = Vec<Point>;

/// Axis-aligned bounding box: [min_point, max_point]
pub type AABB = [Point; 2];

/// Processed stroke: [x0,y0,x1,y1,x2,y2,x3,y3,angle_encoded,length_encoded]
/// 4 sampled points (8 coords) + angle (1) + length (1) = 10 values
pub type StrokeProcessed = Vec<f64>;

/// CJK Character (ideograph)
pub type Ideograph = String;

const NUM_POSSIBLE_ENCODED_VALUE: usize = 256;
const NUM_ENCODED_POINTS: usize = 4;

/// Vector utility functions
struct VectorFunctions;

impl VectorFunctions {
    /// Calculate squared distance between two points
    fn distance2(p0: Point, p1: Point) -> f64 {
        Self::norm2(Self::subtract(p0, p1))
    }

    /// Calculate squared norm (magnitude) of a point/vector
    fn norm2(p: Point) -> f64 {
        p[0] * p[0] + p[1] * p[1]
    }

    /// Round point coordinates to nearest integer
    fn round(p: Point) -> Point {
        [p[0].round(), p[1].round()]
    }

    /// Subtract two points
    fn subtract(p0: Point, p1: Point) -> Point {
        [p0[0] - p1[0], p0[1] - p1[1]]
    }
}

/// Map point from one AABB to another
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

/// Calculate bounding box for all strokes
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

/// Downsample stroke to specified number of points
fn process_stroke(stroke: &Stroke, how_many_points_to_sample: usize) -> Stroke {
    let mut result_stroke: Stroke = Vec::new();
    let mut stroke_length = 0.0;

    // Calculate total stroke length
    for i in 0..stroke.len() - 1 {
        stroke_length += VectorFunctions::distance2(stroke[i], stroke[i + 1]).sqrt();
    }

    let mut h = 0;
    let mut point_candidate = stroke[0];
    let mut u = 0.0;

    // Sample points at equal intervals along the stroke
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

/// Adjust AABB to meet minimum width and aspect ratio constraints
fn do_something_to_aabb(mut aabb: AABB, max_ratio: f64, min_width: f64) -> AABB {
    aabb[0] = VectorFunctions::round(aabb[0]);
    aabb[1] = VectorFunctions::round(aabb[1]);

    let mut e = VectorFunctions::subtract(aabb[1], aabb[0]);
    if e[0] < 0.0 || e[1] < 0.0 {
        panic!("Invalid AABB: {:?}", aabb);
    }

    // Apply minimum width constraint
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

    // Apply aspect ratio constraint
    if max_ratio > 0.0 {
        e = VectorFunctions::subtract(aabb[1], aabb[0]);
        if e[0] < e[1] / max_ratio {
            let i = ((e[1] / max_ratio - e[0]) / 2.0).ceil();
            aabb[0][0] -= i;
            aabb[1][0] += i;
        } else if e[1] < e[0] / max_ratio {
            let i = ((e[0] / max_ratio - e[1]) / 2.0).ceil();
            aabb[0][1] -= i;
            aabb[1][1] += i;
        }
    }

    aabb
}

/// Preprocess strokes into encoded format
fn preprocess_strokes(strokes: &[Stroke], opts: &MatcherOptions) -> Vec<StrokeProcessed> {
    if strokes.is_empty() || strokes.iter().any(|s| s.is_empty()) {
        panic!("Invalid medians list");
    }

    let side_length = NUM_POSSIBLE_ENCODED_VALUE as f64;
    let aabb_after = do_something_to_aabb(get_aabb(strokes), opts.max_ratio, opts.min_width);
    let target_aabb: AABB = [[0.0, 0.0], [255.0, 255.0]];
    let project = create_normalized_project_function(aabb_after, target_aabb);

    strokes
        .iter()
        .map(|stroke| {
            // Project and downsample stroke
            let projected: Stroke = stroke.iter().map(|&p| project(p)).collect();
            let stroke_processed = process_stroke(&projected, NUM_ENCODED_POINTS);

            // Calculate stroke span (from first to last point)
            let stroke_span = VectorFunctions::subtract(
                stroke_processed[stroke_processed.len() - 1],
                stroke_processed[0],
            );

            // Encode angle: map [-PI, PI] to [0, 256)
            let stroke_angle = stroke_span[1].atan2(stroke_span[0]);
            let angle_encoded =
                (((stroke_angle + PI) * side_length) / (2.0 * PI)).round() as i32 % 256;

            // Encode length: sqrt(norm2 / 2)
            let length_encoded = (VectorFunctions::norm2(stroke_span) / 2.0).sqrt().round() as i32;

            // Flatten stroke points and append angle and length
            let mut result: StrokeProcessed = stroke_processed.into_iter().flatten().collect();
            result.push(angle_encoded as f64);
            result.push(length_encoded as f64);
            result
        })
        .collect()
}

/// Calculate similarity score between input and reference strokes
fn score_similarity(input: &[StrokeProcessed], reference: &[StrokeProcessed]) -> f64 {
    let mut score = 0.0;

    for i in 0..input.len() {
        let input_stroke = &input[i];
        let ref_stroke = &reference[i];

        // Compare sampled point positions
        for s in 0..NUM_ENCODED_POINTS {
            score -= (input_stroke[2 * s] - ref_stroke[2 * s]).abs();
            score -= (input_stroke[2 * s + 1] - ref_stroke[2 * s + 1]).abs();
        }

        // Compare angles (circular distance)
        let c = (input_stroke[2 * NUM_ENCODED_POINTS] - ref_stroke[2 * NUM_ENCODED_POINTS]).abs();
        let angle_similarity = c.min(NUM_POSSIBLE_ENCODED_VALUE as f64 - c);

        // Weight by average length
        let lengthy = (input_stroke[2 * NUM_ENCODED_POINTS + 1]
            + ref_stroke[2 * NUM_ENCODED_POINTS + 1])
            / NUM_POSSIBLE_ENCODED_VALUE as f64;

        const MAGIC_PER_STROKE_WEIGHT: f64 = 4.0;
        score -= MAGIC_PER_STROKE_WEIGHT * NUM_ENCODED_POINTS as f64 * lengthy * angle_similarity;
    }

    score
}

/// Matcher configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
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
