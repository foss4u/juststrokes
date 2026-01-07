use juststrokes_rust::data::load_graphics_json;

#[test]
fn debug_nei_character() {
    let data = load_graphics_json("graphics.json").expect("Failed to load graphics.json");

    // Find '內' and '内' in the database
    let nei_traditional = data.iter().find(|(c, _)| c == "內");
    let nei_simplified = data.iter().find(|(c, _)| c == "内");

    if let Some((char_trad, strokes_trad)) = nei_traditional {
        println!("Found '{}' with {} strokes", char_trad, strokes_trad.len());
        println!("Traditional data: {:?}", strokes_trad);

        // Check if simplified version exists
        if let Some((char_simp, strokes_simp)) = nei_simplified {
            println!(
                "\nFound '{}' with {} strokes",
                char_simp,
                strokes_simp.len()
            );
            println!("Simplified data: {:?}", strokes_simp);

            // Compare stroke by stroke
            for (i, (trad_stroke, simp_stroke)) in
                strokes_trad.iter().zip(strokes_simp.iter()).enumerate()
            {
                println!("\nStroke {}:", i);
                println!("  Trad: {:?}", trad_stroke);
                println!("  Simp: {:?}", simp_stroke);

                // Check if identical
                let identical = trad_stroke == simp_stroke;
                println!("  Identical: {}", identical);
            }
        }
    }
}
