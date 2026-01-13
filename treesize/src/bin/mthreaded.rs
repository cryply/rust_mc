use std::{collections::VecDeque, sync::{Mutex, OnceLock}};
use tokio::time::{sleep, Duration};
use tokio::task::JoinSet;
use std::fs;
use std::sync::atomic::{AtomicUsize};
use std::os::unix::fs::MetadataExt;
use clap::Parser;

static TASKS: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();

const NWORKERS: u8 = 5;
static FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);
static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);
static TOTAL_SIZE: AtomicUsize = AtomicUsize::new(0);

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    path: String,
}

fn finc(){
    FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}


fn dinc(){
    DIR_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

fn total_add(size: usize){
    TOTAL_SIZE.fetch_add(size, std::sync::atomic::Ordering::Relaxed);
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

async fn worker(id : u8) {
    let mut left = 100;
    println!("Worker: {id}");
    loop{
        if TASKS.get().unwrap().lock().unwrap().len() > 0 {
            left = 100;
            let path = TASKS.get().unwrap().lock().unwrap().pop_front().unwrap();
            println!("{id}:Got task : {}", &path);

            let mut ts = 0;
            let metadata = fs::metadata(&path).unwrap();

            if metadata.is_dir() {
                dinc();
                let paths = fs::read_dir(path).unwrap();
                for p in paths {
                    let new_path =  p.unwrap().path();
                    let np = new_path.to_str().unwrap();
                    // ts += tree_size(np).unwrap();
                    TASKS.get().unwrap().lock().unwrap().push_back(np.to_string());
                }
                // println!("Path:{} : {}",path, fmt_bytes(ts));
            }else{
                finc();
                ts = metadata.size();
                println!("{id}: File:{} : {}",&path, fmt_bytes(ts));
                total_add(ts as usize);
                // return Ok(ts);
            }            
      
        }
        left -= 1;
        if left == 0 {
            break;
        }

        sleep(Duration::from_millis(10)).await;        
    }
}


#[tokio::main]
async fn main() {

    let args = Args::parse();

    let path = &args.path;

    TASKS.get_or_init(|| Mutex::new(VecDeque::new()));

    TASKS.get().unwrap().lock().unwrap().push_back(path.to_string());
    println!("{:?}", TASKS.get().unwrap().lock().unwrap());

    let mut workers = JoinSet::new();

    for id in 0..NWORKERS {
        workers.spawn(worker(id));
    }

    workers.join_all().await;

        println!("Total files:{}   dirs: {} Treesize: {}", 
        FILE_COUNTER.load(std::sync::atomic::Ordering::Relaxed),
        DIR_COUNTER.load(std::sync::atomic::Ordering::Relaxed),
        fmt_bytes(TOTAL_SIZE.load(std::sync::atomic::Ordering::Relaxed) as u64)
    );
}