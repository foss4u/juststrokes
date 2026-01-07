use juststrokes_rust::{Matcher, csv_data, socket_service};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let data_file = args.get(1).map(|s| s.as_str()).unwrap_or("graphics.csv");
    let socket_path = args
        .get(2)
        .map(|s| s.to_string())
        .unwrap_or_else(socket_service::default_socket_path);

    println!("JustStrokes Handwriting Recognition Service");
    println!("Loading character database from {}...", data_file);

    // Load character database
    let data = if data_file.ends_with(".csv") {
        csv_data::load_graphics_csv(data_file)?
    } else {
        juststrokes_rust::data::load_graphics_json(data_file)?
    };

    println!("Loaded {} characters", data.len());

    // Create matcher
    let matcher = Matcher::new(data, None);

    // Start socket service
    println!("Starting Unix socket service at {}", socket_path);
    let service = socket_service::SocketService::new(matcher, socket_path);
    service.start()?;

    Ok(())
}
