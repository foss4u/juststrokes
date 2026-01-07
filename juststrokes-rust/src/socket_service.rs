use crate::{Matcher, Stroke};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

/// Unix socket service for handwriting recognition
pub struct SocketService {
    matcher: Matcher,
    socket_path: String,
}

impl SocketService {
    /// Create new socket service with character database
    pub fn new(matcher: Matcher, socket_path: String) -> Self {
        Self {
            matcher,
            socket_path,
        }
    }

    /// Start listening on Unix socket
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Create socket directory if needed
        if let Some(parent) = Path::new(&self.socket_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // Remove existing socket file
        let _ = fs::remove_file(&self.socket_path);

        // Bind to socket
        let listener = UnixListener::bind(&self.socket_path)?;
        println!("Listening on {}", self.socket_path);

        // Accept connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = self.handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handle single client connection
    fn handle_client(&self, mut stream: UnixStream) -> Result<(), Box<dyn std::error::Error>> {
        // Read request line
        let mut line = String::new();
        {
            let mut reader = BufReader::new(&stream);
            reader.read_line(&mut line)?;
        }

        // Parse CSV input: max_width\tmax_height\tstroke1_points\tstroke2_points\t...
        // Each stroke: x0,y0,x1,y1,...
        let parts: Vec<&str> = line.trim().split('\t').collect();
        if parts.len() < 3 {
            stream.write_all(b"ERROR\tInvalid input format\n")?;
            return Ok(());
        }

        let _max_width: f64 = parts[0].parse()?;
        let _max_height: f64 = parts[1].parse()?;

        // Parse strokes
        let mut strokes: Vec<Stroke> = Vec::new();
        for stroke_str in &parts[2..] {
            let coords: Vec<f64> = stroke_str
                .split(',')
                .filter_map(|s| s.parse().ok())
                .collect();

            if !coords.len().is_multiple_of(2) {
                stream.write_all(b"ERROR\tInvalid stroke coordinates\n")?;
                return Ok(());
            }

            let mut stroke: Stroke = Vec::new();
            for i in (0..coords.len()).step_by(2) {
                stroke.push([coords[i], coords[i + 1]]);
            }
            strokes.push(stroke);
        }

        // Match strokes
        let candidates = self.matcher.match_strokes(&strokes, 10);

        // Return results as CSV: char1\tscore1\tchar2\tscore2\t...
        // Note: We don't have scores in current API, so just return characters
        for (i, candidate) in candidates.iter().enumerate() {
            if i > 0 {
                stream.write_all(b"\t")?;
            }
            stream.write_all(candidate.as_bytes())?;
        }
        stream.write_all(b"\n")?;

        Ok(())
    }
}

/// Get default socket path based on user ID
pub fn default_socket_path() -> String {
    let uid = unsafe { libc::getuid() };
    format!("/run/user/{}/handwritten/juststrokes.socket", uid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_data::load_graphics_csv;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_socket_service() {
        // Load database from JSON (more reliable)
        let data = crate::data::load_graphics_json("graphics.json")
            .expect("Failed to load database");
        let matcher = Matcher::new(data, None);

        // Use test socket path
        let socket_path = "/tmp/juststrokes_test.socket".to_string();
        let socket_path_clone = socket_path.clone();

        // Start service in background thread
        let service = SocketService::new(matcher, socket_path.clone());
        thread::spawn(move || {
            let _ = service.start();
        });

        // Wait for socket to be ready
        thread::sleep(Duration::from_millis(100));

        // Connect and send test request
        let mut stream = UnixStream::connect(&socket_path).expect("Failed to connect");

        // Send simple stroke data: max_width, max_height, stroke1, stroke2
        let request = "400\t400\t0,0,100,100,200,200\t50,50,150,150\n";
        stream
            .write_all(request.as_bytes())
            .expect("Failed to write");

        // Read response
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("Failed to read");

        // Should get some candidates back
        assert!(!response.is_empty());
        assert!(!response.starts_with("ERROR"));

        // Cleanup
        let _ = fs::remove_file(&socket_path_clone);
    }
}
