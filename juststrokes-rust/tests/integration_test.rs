use juststrokes_rust::{Matcher, data::load_graphics_json};

#[test]
fn test_all_characters_match_themselves() {
    // Load the character database
    let data = load_graphics_json("graphics.json").expect("Failed to load graphics.json");
    println!("Loaded {} characters from database", data.len());

    // Create matcher with the database
    let matcher = Matcher::new(data.clone(), None);

    let mut total_tested = 0;
    let mut total_passed = 0;
    let mut failed_chars = Vec::new();

    // Test each character
    for (expected_char, strokes_processed) in &data {
        // Use preprocessed data directly - graphics.json already contains processed strokes
        // No need to reconstruct and reprocess, which would introduce errors
        let candidates = matcher.match_preprocessed(strokes_processed, 5);

        total_tested += 1;

        // Check if the character matches itself with highest score
        if !candidates.is_empty() && &candidates[0] == expected_char {
            total_passed += 1;
        } else {
            failed_chars.push((expected_char.clone(), candidates.clone()));
            if failed_chars.len() <= 10 {
                println!(
                    "FAIL: Expected '{}', got candidates: {:?}",
                    expected_char, candidates
                );
            }
        }
    }

    println!("\n=== Test Results ===");
    println!("Total tested: {}", total_tested);
    println!("Passed: {}", total_passed);
    println!("Failed: {}", total_tested - total_passed);
    println!(
        "Success rate: {:.2}%",
        (total_passed as f64 / total_tested as f64) * 100.0
    );

    if !failed_chars.is_empty() {
        println!("\nFirst 10 failures:");
        for (expected, candidates) in failed_chars.iter().take(10) {
            println!("  Expected: '{}', Got: {:?}", expected, candidates);
        }
    }

    // Assert that all characters match themselves
    assert_eq!(
        total_passed,
        total_tested,
        "Not all characters matched themselves. {} out of {} failed.",
        total_tested - total_passed,
        total_tested
    );
}

#[test]
fn test_sample_characters() {
    // Load database
    let data = load_graphics_json("graphics.json").expect("Failed to load graphics.json");
    let matcher = Matcher::new(data.clone(), None);

    // Test first 10 characters
    for (expected_char, strokes_processed) in data.iter().take(10) {
        let candidates = matcher.match_preprocessed(strokes_processed, 1);

        assert!(
            !candidates.is_empty(),
            "No candidates found for '{}'",
            expected_char
        );
        assert_eq!(
            &candidates[0], expected_char,
            "Character '{}' did not match itself",
            expected_char
        );
    }
}

#[test]
fn test_different_stroke_counts() {
    let data = load_graphics_json("graphics.json").expect("Failed to load graphics.json");
    let matcher = Matcher::new(data.clone(), None);

    // Find characters with different stroke counts
    let mut stroke_count_map: std::collections::HashMap<usize, Vec<String>> =
        std::collections::HashMap::new();

    for (character, strokes) in &data {
        stroke_count_map
            .entry(strokes.len())
            .or_default()
            .push(character.clone());
    }

    println!("Stroke count distribution:");
    let mut counts: Vec<_> = stroke_count_map.keys().collect();
    counts.sort();
    for count in counts.iter().take(10) {
        println!(
            "  {} strokes: {} characters",
            count,
            stroke_count_map[count].len()
        );
    }

    // Test that characters with different stroke counts don't match
    if let Some(chars_1_stroke) = stroke_count_map.get(&1)
        && let Some(chars_2_stroke) = stroke_count_map.get(&2)
        && !chars_1_stroke.is_empty()
        && !chars_2_stroke.is_empty()
    {
        // Get a 1-stroke character
        let char_1 = &chars_1_stroke[0];
        let strokes_1 = data
            .iter()
            .find(|(c, _)| c == char_1)
            .map(|(_, s)| s)
            .unwrap();

        let candidates = matcher.match_preprocessed(strokes_1, 5);

        // Should only match characters with same stroke count
        assert!(
            !candidates.is_empty(),
            "Should find candidates for 1-stroke character"
        );
        assert_eq!(
            &candidates[0], char_1,
            "1-stroke character should match itself"
        );
    }
}
