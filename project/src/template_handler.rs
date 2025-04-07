use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

/// Process an HTML template by replacing placeholders with values
pub fn process_template(
    template_path: &str,
    values: HashMap<String, String>,
    proxy_mode: bool,
    upstream_server: Option<String>,
) -> String {
    let mut content = match fs::read_to_string(template_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading template {}: {}", template_path, e);
            return format!("Error: Template not found - {}", e);
        }
    };

    // Replace all placeholders with values
    for (key, value) in values {
        let placeholder = format!("{{{{{}}}}}", key);
        content = content.replace(&placeholder, &value);
    }

    // Handle proxy-specific display values
    if proxy_mode {
        content = content.replace("{{PROXY_DISPLAY}}", "block");
        content = content.replace("{{LOAD_BALANCED}}", "load-balanced");

        if let Some(server) = upstream_server {
            content = content.replace("{{UPSTREAM_SERVER}}", &server);
        } else {
            content = content.replace("{{UPSTREAM_SERVER}}", "Unknown");
        }
    } else {
        content = content.replace("{{PROXY_DISPLAY}}", "none");
        content = content.replace("{{LOAD_BALANCED}}", "");
        content = content.replace("{{UPSTREAM_SERVER}}", "N/A");
    }

    // Handle any remaining placeholders
    content = content.replace("{{REQUEST_ID}}", &generate_request_id());

    content
}

// Generate a unique request ID for tracking
fn generate_request_id() -> String {
    let thread_id = thread::current().id();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    format!("{:?}-{}", thread_id, timestamp)
}

// RoundRobinBalancer implemented with thread safety for web server
pub struct RoundRobinBalancer {
    servers: Vec<String>,
    next: AtomicUsize,
}

impl RoundRobinBalancer {
    pub fn new(servers: Vec<String>) -> Self {
        RoundRobinBalancer {
            servers,
            next: AtomicUsize::new(0),
        }
    }

    pub fn get_next_server(&self) -> String {
        let current = self.next.fetch_add(1, Ordering::SeqCst) % self.servers.len();
        self.servers[current].clone()
    }
}

// Create the template values including server information
pub fn create_template_values(
    port: u16,
    thread_id: &str,
    server_address: &str,
) -> HashMap<String, String> {
    let mut values = HashMap::new();
    values.insert("PORT".to_string(), port.to_string());
    values.insert("THREAD_ID".to_string(), thread_id.to_string());
    values.insert("SERVER_ADDRESS".to_string(), server_address.to_string());
    values
}
