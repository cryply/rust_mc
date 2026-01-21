// #![deny(warnings)]

use async_channel::{unbounded, Receiver, Sender};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client as S3Client;
use clap::{Parser, ValueEnum};
use reqwest::header::CONTENT_TYPE;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task::JoinSet;

const N: usize = 5;
const LOCAL_STORAGE: &str = "../data";
static CACHE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[derive(Debug, Clone, ValueEnum)]
enum StorageType {
    Local,
    S3,
}

#[derive(Parser, Debug)]
#[command(name = "crawler")]
#[command(about = "A web crawler with local or S3 storage options")]
struct Args {
    /// Storage type: local or s3
    #[arg(short, long, value_enum, default_value = "local")]
    storage: StorageType,

    /// S3 bucket name (required if storage is s3)
    #[arg(short, long)]
    bucket: Option<String>,

    /// S3 key prefix (optional, defaults to "crawled/")
    #[arg(short, long, default_value = "crawled/")]
    prefix: String,

    /// Starting URLs to crawl
    #[arg(short, long, default_value = "https://sabrinajewson.org/rust-nomicon/atomics/atomics.html")]
    url: Vec<String>,

    /// Number of worker tasks
    #[arg(short, long, default_value_t = N)]
    workers: usize,
}

#[derive(Clone)]
enum Storage {
    Local { path: String },
    S3 { client: S3Client, bucket: String, prefix: String },
}

impl Storage {
    async fn save(&self, filename: &str, data: &[u8], content_type: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self {
            Storage::Local { path } => {
                let filepath = format!("{}/{}", path, filename);
                println!("Saving to local: {}", filepath);
                
                // Ensure directory exists
                if let Some(parent) = Path::new(&filepath).parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                
                let mut file = File::create(&filepath).await?;
                file.write_all(data).await?;
                file.flush().await?;
                Ok(())
            }
            Storage::S3 { client, bucket, prefix } => {
                let key = format!("{}{}", prefix, filename);
                println!("Saving to S3: s3://{}/{}", bucket, key);
                
                let mut request = client
                    .put_object()
                    .bucket(bucket)
                    .key(&key)
                    .body(ByteStream::from(data.to_vec()));
                
                if let Some(ct) = content_type {
                    request = request.content_type(ct);
                }
                
                match request.send().await {
                    Ok(_) => {
                        println!("Successfully uploaded: {}", key);
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("S3 upload error: {:?}", e);
                        eprintln!("Raw error: {:#?}", e.raw_response());
                        Err(e.into())
                    }
                }
            }
        }
    }
}

fn is_text_like(content_type: &str) -> bool {
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    mime.starts_with("text/")
        || mime == "application/json"
        || mime == "application/xml"
        || mime == "application/javascript"
}

fn make_filename(url: &str) -> String {
    let mut s = Path::new(url)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(url)
        .replace("https://", "")
        .replace([':', '/'], "_-");

    let patterns = [".gif", ".css", ".html", ".json", ".js", ".pdf"];

    if !patterns.iter().any(|&pat| s.ends_with(pat)) {
        s.push_str(".html");
    }

    println!("URL: {} -> Filename: {}", url, &s);
    s
}

async fn crawl_url(
    url: String,
    queue: Sender<String>,
    in_flight: Arc<AtomicUsize>,
    storage: Storage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if CACHE.get().unwrap().lock().unwrap().contains(&url) {
        return Ok(());
    }

    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Ok(());
    }

    let content_type = response.headers().get(CONTENT_TYPE).cloned();

    let ct_str = content_type
        .as_ref()
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("application/octet-stream");

    CACHE.get().unwrap().lock().unwrap().insert(url.clone());

    let mut body = String::new();
    let data = if is_text_like(ct_str) {
        body = response.text().await?;
        body.clone().into_bytes()
    } else {
        response.bytes().await?.to_vec()
    };

    let filename = make_filename(url.as_str());
    storage.save(&filename, &data, Some(ct_str)).await?;

    // Extract links from HTML content
    if is_text_like(ct_str) && ct_str.contains("html") {
        let links: Vec<String> = {
            let document = Html::parse_document(&body);
            let link_selector = Selector::parse("a").unwrap();

            document
                .select(&link_selector)
                .filter_map(|el| {
                    el.value()
                        .attr("href")
                        .filter(|h| !h.is_empty() && h.starts_with("http"))
                        .map(|h| h.to_string())
                })
                .collect()
        };

        for link in links {
            in_flight.fetch_add(1, Ordering::SeqCst);
            queue.send(link).await?;
        }
    }

    thread::sleep(Duration::from_secs(1));
    Ok(())
}

async fn worker(
    id: usize,
    queue: Receiver<String>,
    sender: Sender<String>,
    in_flight: Arc<AtomicUsize>,
    storage: Storage,
) {
    println!("Worker {} started", id);

    while let Ok(url) = queue.recv().await {
        if let Err(e) = crawl_url(url, sender.clone(), in_flight.clone(), storage.clone()).await {
            eprintln!("Worker {} error: {}", id, e);
        }

        let remaining = in_flight.fetch_sub(1, Ordering::SeqCst) - 1;

        if remaining == 0 {
            println!("Worker {} detected completion", id);
            break;
        }
    }

    println!("Worker {} finished", id);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    CACHE.get_or_init(|| Mutex::new(HashSet::new()));

    let storage = match args.storage {
        StorageType::Local => {
            println!("Using local storage: {}", LOCAL_STORAGE);
            // Ensure local directory exists
            tokio::fs::create_dir_all(LOCAL_STORAGE).await?;
            Storage::Local {
                path: LOCAL_STORAGE.to_string(),
            }
        }
        StorageType::S3 => {
            let bucket = args.bucket.expect("S3 bucket name is required when using S3 storage. Use --bucket <name>");
            println!("Using S3 storage: s3://{}/{}", bucket, args.prefix);
            
            let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
            let client = S3Client::new(&config);
            
            Storage::S3 {
                client,
                bucket,
                prefix: args.prefix,
            }
        }
    };

    let worker_count = args.workers;
    let (tx, rx) = unbounded::<String>();
    let in_flight = Arc::new(AtomicUsize::new(0));

    for url in &args.url {
        in_flight.fetch_add(1, Ordering::SeqCst);
        tx.send(url.to_string()).await?;
    }

    let mut workers = JoinSet::new();

    for id in 0..worker_count {
        let rx = rx.clone();
        let tx = tx.clone();
        let in_flight = in_flight.clone();
        let storage = storage.clone();

        workers.spawn(worker(id, rx, tx, in_flight, storage));
    }

    while let Some(res) = workers.join_next().await {
        println!("{:?}", res);
    }

    println!("All work completed...");
    Ok(())
}