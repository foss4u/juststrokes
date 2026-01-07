use crate::{Ideograph, StrokeProcessed};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Type alias for character database
pub type CharacterDatabase = Vec<(Ideograph, Vec<StrokeProcessed>)>;

/// Load character database from graphics.json
/// Format: [[char, [[x0,y0,...,angle,length], ...]], ...]
pub fn load_graphics_json<P: AsRef<Path>>(
    path: P,
) -> Result<CharacterDatabase, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let data: Value = serde_json::from_str(&content)?;

    let mut result = Vec::new();

    if let Value::Array(entries) = data {
        for entry in entries {
            if let Value::Array(pair) = entry
                && pair.len() == 2
            {
                // Extract character
                let character = pair[0]
                    .as_str()
                    .ok_or("Invalid character format")?
                    .to_string();

                // Extract strokes
                let mut strokes = Vec::new();
                if let Value::Array(stroke_data) = &pair[1] {
                    for stroke in stroke_data {
                        if let Value::Array(values) = stroke {
                            let processed: StrokeProcessed =
                                values.iter().filter_map(|v| v.as_f64()).collect();
                            strokes.push(processed);
                        }
                    }
                }

                result.push((character, strokes));
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_graphics_json() {
        let data = load_graphics_json("graphics.json");
        assert!(data.is_ok());
        let data = data.unwrap();
        assert!(!data.is_empty());

        // Check first entry structure
        let (character, strokes) = &data[0];
        assert!(!character.is_empty());
        assert!(!strokes.is_empty());

        // Each stroke should have 10 values (8 coords + angle + length)
        for stroke in strokes {
            assert_eq!(stroke.len(), 10);
        }
    }
}
