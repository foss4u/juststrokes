use crate::{Ideograph, StrokeProcessed};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Database of characters with their preprocessed stroke features
pub type CharacterDatabase = Vec<(Ideograph, Vec<StrokeProcessed>)>;

/// Convert graphics.json to CSV format
/// Format: character\tx0,y0,x1,y1,x2,y2,x3,y3,angle,length\t...
pub fn json_to_csv<P: AsRef<Path>, Q: AsRef<Path>>(
    json_path: P,
    csv_path: Q,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = crate::data::load_graphics_json(json_path)?;
    let mut file = fs::File::create(csv_path)?;

    for (character, strokes) in data {
        write!(file, "{}", character)?;
        for stroke in strokes {
            write!(file, "\t")?;
            for (i, value) in stroke.iter().enumerate() {
                if i > 0 {
                    write!(file, ",")?;
                }
                write!(file, "{}", value)?;
            }
        }
        writeln!(file)?;
    }

    Ok(())
}

/// Load preprocessed character database from CSV file
/// Format: character\tx0,y0,x1,y1,x2,y2,x3,y3,angle,length\t...
/// Tab-delimited, UTF-8 encoded
pub fn load_graphics_csv<P: AsRef<Path>>(
    path: P,
) -> Result<CharacterDatabase, Box<dyn std::error::Error>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut result = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split('\t');

        // First column is the character
        let character = parts.next().ok_or("Missing character column")?.to_string();

        // Remaining columns are stroke data
        let mut strokes = Vec::new();
        for stroke_str in parts {
            let values: Result<Vec<f64>, _> =
                stroke_str.split(',').map(|s| s.parse::<f64>()).collect();
            strokes.push(values?);
        }

        result.push((character, strokes));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_csv_conversion() {
        // Convert JSON to CSV
        json_to_csv("graphics.json", "graphics.csv").expect("Failed to convert to CSV");

        // Load both formats
        let json_data =
            crate::data::load_graphics_json("graphics.json").expect("Failed to load JSON");
        let csv_data = load_graphics_csv("graphics.csv").expect("Failed to load CSV");

        // Verify same number of characters
        assert_eq!(json_data.len(), csv_data.len());

        // Verify first few characters match
        for i in 0..10.min(json_data.len()) {
            assert_eq!(json_data[i].0, csv_data[i].0);
            assert_eq!(json_data[i].1.len(), csv_data[i].1.len());

            // Check stroke data matches
            for (j, (json_stroke, csv_stroke)) in
                json_data[i].1.iter().zip(csv_data[i].1.iter()).enumerate()
            {
                assert_eq!(
                    json_stroke.len(),
                    csv_stroke.len(),
                    "Stroke {} length mismatch for character '{}'",
                    j,
                    json_data[i].0
                );

                for (k, (json_val, csv_val)) in
                    json_stroke.iter().zip(csv_stroke.iter()).enumerate()
                {
                    assert!(
                        (json_val - csv_val).abs() < 0.0001,
                        "Value mismatch at stroke {}, index {} for character '{}': {} vs {}",
                        j,
                        k,
                        json_data[i].0,
                        json_val,
                        csv_val
                    );
                }
            }
        }
    }

    #[test]
    fn test_csv_loading() {
        // Ensure CSV exists
        if !std::path::Path::new("graphics.csv").exists() {
            json_to_csv("graphics.json", "graphics.csv").expect("Failed to convert to CSV");
        }

        let data = load_graphics_csv("graphics.csv").expect("Failed to load CSV");
        assert!(!data.is_empty());

        // Check structure
        let (character, strokes) = &data[0];
        assert!(!character.is_empty());
        assert!(!strokes.is_empty());

        // Each stroke should have 10 values
        for stroke in strokes {
            assert_eq!(stroke.len(), 10);
        }
    }
}
