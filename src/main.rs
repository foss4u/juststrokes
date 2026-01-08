use clap::Parser;
use juststrokes_rust::{Matcher, csv_data, socket_service};

/// JustStrokes - Chinese character handwriting recognition service
#[derive(Parser)]
#[command(name = "juststrokes-rust")]
#[command(about = "Chinese character handwriting recognition via Unix socket", long_about = None)]
#[command(version = env!("GIT_VERSION"))]
struct Args {
    /// Path to character database (JSON or CSV format)
    #[arg(short = 'd', long, default_value = "graphics.csv")]
    data_file: String,

    /// Unix socket path for API service
    #[arg(short = 's', long)]
    socket_path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let socket_path = args
        .socket_path
        .unwrap_or_else(socket_service::default_socket_path);

    println!("JustStrokes Handwriting Recognition Service");
    println!("Version: {}", env!("GIT_VERSION"));
    println!("Loading character database from {}...", args.data_file);

    // Load character database
    let data = if args.data_file.ends_with(".csv") {
        csv_data::load_graphics_csv(&args.data_file)?
    } else {
        juststrokes_rust::data::load_graphics_json(&args.data_file)?
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
