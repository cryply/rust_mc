use clap::Parser;
use std::fs;
use std::io::{self, BufReader, Read};
use std::os::unix::fs::MetadataExt;
use std::sync::atomic::{AtomicUsize};


static FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);




fn finc(){
    FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}


fn dinc(){
    DIR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
    // #[arg(short, long)]
    // outfile: String,
}


fn fmt_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{} {}", bytes, units[0]);
    }
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, units[unit_idx])
}


fn tree_size(path: &str) -> Result<u64, io::Error> {
    let mut ts = 0;
    let metadata = fs::metadata(path)?;


    if metadata.is_dir() {
        dinc();
        let paths = fs::read_dir(path).unwrap();
        for p in paths {
            let new_path =  p.unwrap().path();
            let np = new_path.to_str().unwrap();
            ts += tree_size(np).unwrap();
        }
        println!("Path:{} : {}",path, fmt_bytes(ts));
    }else{
        finc();
        ts = metadata.size();
        println!("File:{} : {}",path, fmt_bytes(ts));
        return Ok(ts);
    }
    Ok(ts)
}

fn main() {
    // let args = Args::parse();
    // match tree_size(&args.path) {
    match tree_size("./") {
        Ok(sz) => println!("treesize: {}", sz),
        Err(e) => eprintln!("Error reading file: {}", e),
    }
    println!("Total files:{}   dirs: {}", 
        FILE_COUNTER.load(std::sync::atomic::Ordering::Relaxed),
        DIR_COUNTER.load(std::sync::atomic::Ordering::Relaxed)
    
    );
}
