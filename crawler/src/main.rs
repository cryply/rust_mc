// #![deny(warnings)]

use async_channel::{unbounded, Sender, Receiver};
use reqwest::StatusCode;
use reqwest::{Client, header::{CONTENT_TYPE, CONTENT_DISPOSITION}};
use tokio::task::JoinSet;
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use scraper::{Html, Selector};

const N: usize = 5;
const STORAGE: &str = "../data";
static CACHE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn is_text_like(content_type: &str) -> bool {
    // Normalise: lower‑case, trim parameters (e.g. "text/html; charset=utf-8")
    let mime = content_type.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    mime.starts_with("text/")
        || mime == "application/json"
        || mime == "application/xml"
        || mime == "application/javascript"

}

async fn make_filename(url: &str) -> String {
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

    println!("was: {} now: {}", url, &s);
    return format!("{}/{}", STORAGE, s);
}

async fn crawl_url(
    url: String,
    queue: Sender<String>,
    in_flight: Arc<AtomicUsize>,
) -> Result<(), Box<dyn std::error::Error>>{
    // println!("Crawling: {}", url);

    if CACHE.get().unwrap().lock().unwrap().contains(&url) {
        return Ok(())
    }


    let response = reqwest::get(&url).await?;


    if !response.status().is_success() {
        return Ok(());
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .cloned();
        // e.g. "text/html; charset=UTF-8"
    // let content_disposition = response
    //     .headers()
    //     .get(CONTENT_DISPOSITION)
    //     .cloned();


    // Helper to strip charset & other parameters
    let simple_ct = content_type.unwrap();
    let ct2 = simple_ct.to_str().unwrap();
    let mut  body: String = "".to_string();


    CACHE.get().unwrap().lock().unwrap().insert(url.clone());

    let data = match(is_text_like(ct2)){
        true => {
            body = response.text().await?;
            body.clone().into_bytes()},
        false =>  response.bytes().await?.to_vec(),
    };




    let filename = make_filename(url.as_str()).await;
    println!("New file: {}", filename);
    let mut file = File::create(filename).await?;
    file.write_all(&data).await?;
    file.flush().await?;


    let links: Vec<String> = {
      
        let document = Html::parse_document(&body);
        let link_selector = Selector::parse("a").unwrap();

        document
            .select(&link_selector)
            .filter_map(|el| {
                el.value()
                    .attr("href")
                    .filter(|h| !h.is_empty())
                    .map(|h| h.to_string())
            })
            .collect()
    }; 

    
    for link in links {
        in_flight.fetch_add(1, Ordering::SeqCst);
        queue.send(link).await?;
    }

    thread::sleep(Duration::from_secs(1));
    // panic!();
    Ok(())

}

async fn worker(
    id: usize,
    queue: Receiver<String>,
    sender: Sender<String>,
    in_flight: Arc<AtomicUsize>,
) {
    println!("Worker {} started", id);
    
    while let Ok(url) = queue.recv().await {
        if let Err(e) = crawl_url(url, sender.clone(), in_flight.clone()).await {
            eprintln!("Worker {} error: {}", id, e);
        }
        
        // Mark task as complete
        let remaining = in_flight.fetch_sub(1, Ordering::SeqCst) - 1;
        
        if remaining == 0 {
            println!("Worker {} detected completion", id);
            break;
        }
    }
    
    println!("Worker {} finished", id);
}

#[tokio::main]
async fn main(){

    CACHE.get_or_init(|| Mutex::new(HashSet::new()));
    
    let worker_count = N;
    let (tx, rx) = unbounded::<String>();
    let in_flight = Arc::new(AtomicUsize::new(0));

    let urls = vec!["https://sabrinajewson.org/rust-nomicon/atomics/atomics.html"];
    for url in urls {
        in_flight.fetch_add(1, Ordering::SeqCst);
        tx.send(url.to_string()).await.unwrap();
    } 

    let mut workers = JoinSet::new();

    for id in 0..worker_count {
        let rx = rx.clone();
        let tx = tx.clone();
        let in_flight = in_flight.clone();

        workers.spawn(worker(id, rx, tx, in_flight));
    }

    while let Some(res) = workers.join_next().await {
        println!("{:?}", res);
    }

    println!("All work completed...");

}
