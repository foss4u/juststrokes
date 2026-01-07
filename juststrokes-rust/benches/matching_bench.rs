use juststrokes_rust::{Matcher, data::load_graphics_json};
use std::time::Instant;

fn main() {
    println!("=== Handwriting Recognition Benchmark ===\n");

    // Load data once before benchmarking
    println!("Loading graphics.json...");
    let load_start = Instant::now();
    let data = load_graphics_json("graphics.json").expect("Failed to load graphics.json");
    let load_duration = load_start.elapsed();
    println!("Loaded {} characters in {:?}\n", data.len(), load_duration);

    // Create matcher
    let matcher = Matcher::new(data.clone(), None);

    // Run benchmark 3 times
    let mut durations = Vec::new();

    for run in 1..=3 {
        println!("Run {}/3:", run);
        let start = Instant::now();

        let mut total_tested = 0;
        let mut total_passed = 0;

        // Test each character
        for (expected_char, strokes_processed) in &data {
            let candidates = matcher.match_preprocessed(strokes_processed, 5);
            total_tested += 1;
            if !candidates.is_empty() && &candidates[0] == expected_char {
                total_passed += 1;
            }
        }

        let duration = start.elapsed();
        durations.push(duration);

        println!(
            "  Tested: {}, Passed: {}, Duration: {:?}",
            total_tested, total_passed, duration
        );
        println!(
            "  Throughput: {:.2} chars/sec\n",
            total_tested as f64 / duration.as_secs_f64()
        );
    }

    // Calculate statistics
    let total_ms: f64 = durations.iter().map(|d| d.as_secs_f64() * 1000.0).sum();
    let avg_ms = total_ms / durations.len() as f64;
    let min_ms = durations
        .iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_ms = durations
        .iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    println!("=== Summary ===");
    println!("Average: {:.2} ms", avg_ms);
    println!("Min: {:.2} ms", min_ms);
    println!("Max: {:.2} ms", max_ms);
    println!(
        "Average throughput: {:.2} chars/sec",
        data.len() as f64 / (avg_ms / 1000.0)
    );

    // Save benchmark results
    let results = format!(
        "Baseline Benchmark Results\n\
         ==========================\n\
         Date: {}\n\
         Characters: {}\n\
         Runs: {}\n\
         Average: {:.2} ms\n\
         Min: {:.2} ms\n\
         Max: {:.2} ms\n\
         Throughput: {:.2} chars/sec\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        data.len(),
        durations.len(),
        avg_ms,
        min_ms,
        max_ms,
        data.len() as f64 / (avg_ms / 1000.0)
    );

    std::fs::write("benchmark_results.txt", results).expect("Failed to write benchmark results");
    println!("\nResults saved to benchmark_results.txt");
}
