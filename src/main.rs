use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use std::sync::Arc;
use lru::LruCache;
use reqwest::Client;
use log::{info, error};
use env_logger;
use base64::{decode};
use std::time::{SystemTime, Duration};

//RUST_LOG=info cargo run | tee proxy_logs.txt RUST_BACKTRACE=1 to run the program

struct CacheEntry {
    response: String,
    timestamp: SystemTime,
}

type SharedCache = Arc<Mutex<LruCache<String, CacheEntry>>>;

async fn handle_client(mut client_stream: TcpStream, cache: SharedCache, client: Client) {
    info!("Handling client on thread {:?}", std::thread::current().id());

    let mut buffer = [0; 1024];
    let bytes_read = client_stream.read(&mut buffer).await.unwrap();

    if bytes_read == 0 {
        return;
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = request.split("\r\n").collect();
    for line in &lines {
        info!("Received header: {}", line);
    }

    if !is_authenticated(&lines) {
        let response = "HTTP/1.1 401 Unauthorized\r\n\r\nAuthentication required";
        if let Err(e) = client_stream.write_all(response.as_bytes()).await {
            error!("Failed to send authentication response: {}", e);
        }
        return;
    }

    let url = extract_url_from_request(&request);
    info!("Received request for URL: {}", url);

    let expiration_duration = Duration::new(60, 0); // Cache expiration duration (60 seconds)

    let (response, source) = {
        let mut cache = cache.lock().await;
        if let Some(entry) = cache.get(&url) {
            if entry.timestamp.elapsed().unwrap() < expiration_duration {
                info!("Cache hit for URL: {}", url);
                (entry.response.clone(), "Cache")
            } else {
                info!("Cache expired for URL: {}", url);
                let response = fetch_from_origin(&url, &client).await;
                cache.put(url.clone(), CacheEntry { response: response.clone(), timestamp: SystemTime::now() });
                (response, "Origin")
            }
        } else {
            info!("Cache miss for URL: {}", url);
            let response = fetch_from_origin(&url, &client).await;
            cache.put(url.clone(), CacheEntry { response: response.clone(), timestamp: SystemTime::now() });
            (response, "Origin")
        }
    };

    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        response.len(),
        response
    );

    if let Err(e) = client_stream.write_all(http_response.as_bytes()).await {
        error!("Failed to send response: {}", e);
    } else {
        info!("Response sent from {}: {}", source, url);
    }
}

fn extract_url_from_request(request: &str) -> String {
    info!("Request>>>>> {}", request);

    let lines: Vec<&str> = request.split("\r\n").collect();
    let raw_url = lines[0].split_whitespace().nth(1).unwrap();
    let url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
        raw_url.to_string()
    } else {
        raw_url[1..].to_string() // Remove the leading slash
    };
    url
}

fn is_authenticated(lines: &Vec<&str>) -> bool {
    for line in lines {
        if line.starts_with("Authorization: Basic ") {
            let encoded_creds = line.trim_start_matches("Authorization: Basic ");
            if let Ok(decoded) = decode(encoded_creds) {
                if let Ok(credentials) = String::from_utf8(decoded) {
                    if credentials == "username:password" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

async fn fetch_from_origin(url: &str, client: &Client) -> String {
    let response = client.get(url).send().await.unwrap().text().await.unwrap();
    response
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    info!("Proxy server running on 127.0.0.1:8080");

    let cache = Arc::new(Mutex::new(LruCache::new(100)));
    let client = Client::new();

    loop {
        let (client_stream, _) = listener.accept().await.unwrap();
        let cache = Arc::clone(&cache);
        let client = client.clone();
        tokio::spawn(async move {
            handle_client(client_stream, cache, client).await;
        });
    }
}
