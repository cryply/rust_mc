use log::{info, warn};

use csv::Writer;
use serde::Serialize;
use std::error::Error;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
enum EtlError {
    #[error("Invalid data point: id={0}, reason: {1}")]
    InvalidData(u32, String),
}

#[derive(Debug)]
struct RawData {
    id: u32,
    value: i32,
}

#[derive(Debug, Clone, Serialize)]
struct CleanData {
    id: u32,
    value: i32,
}

fn total_value(data: &[CleanData]) -> f64 {
    data.iter().fold(0.0, |acc, p| acc + p.value as f64)
}

fn main() {
    env_logger::init();
    
    info!("Starting ETL process");
    
    let raw = vec![
        RawData { id: 1, value: 1000 },
        RawData { id: 2, value: -5 },
        RawData { id: 3, value: 50 },
        RawData { id: 4, value: 75 },
    ];

    info!("Extracted {} raw records", raw.len());
    
    let cleaned = transform(extract(raw));

    info!("Transformed to {} clean records", cleaned.len());
    
    summary(&cleaned);

    if let Err(e) = load(&cleaned) {
        log::error!("Failed to load data: {e}");
        std::process::exit(1);
    }
    
    info!("ETL process completed successfully");
}

fn summary(data: &[CleanData]) {
    let total = total_value(data);
    let mean = total / data.len() as f64;
    info!("Summary - Total: {total}, Mean: {mean:.2}, Count: {}", data.len());
}

fn write_data(filename: &str, data: &[CleanData]) -> Result<(), Box<dyn Error>> {
    let mut wrt = Writer::from_path(filename)?;
    for el in data {
        wrt.serialize(el)?;
    }
    wrt.flush()?;
    info!("Wrote {} records to {}", data.len(), filename);
    Ok(())
}

fn extract(raw: Vec<RawData>) -> Vec<RawData> {
    raw
}

fn transform(raw: Vec<RawData>) -> Vec<CleanData> {
    raw.into_iter()
        .filter_map(|r| {
            if r.id == 0 {
                warn!("Skipping record with id=0");
                return None;
            }

            let original_value = r.value;
            let clamped = r.value.clamp(0, 100);

            if clamped != original_value {
                warn!("Value clamped for id={}: {} -> {}", r.id, original_value, clamped);
            }

            Some(CleanData {
                id: r.id,
                value: clamped,
            })
        })
        .collect()
}

fn load(data: &[CleanData]) -> Result<(), Box<dyn Error>> {
    // Use /data directory for output (mounted volume in Docker)
    let output_dir = if Path::new("/data").exists() {
        "/data"
    } else {
        "."
    };
    
    let filename = format!("{}/output.csv", output_dir);
    write_data(&filename, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_high() {
        let raw = vec![RawData { id: 1, value: 1000 }];
        let cleaned = transform(extract(raw));
        assert_eq!(cleaned[0].value, 100);
    }

    #[test]
    fn test_clamp_low() {
        let raw = vec![RawData { id: 1, value: -5 }];
        let cleaned = transform(extract(raw));
        assert_eq!(cleaned[0].value, 0);
    }

    #[test]
    fn test_no_clamp_needed() {
        let raw = vec![RawData { id: 1, value: 50 }];
        let cleaned = transform(extract(raw));
        assert_eq!(cleaned[0].value, 50);
    }

    #[test]
    fn test_summary_calculation() {
        let data = vec![
            CleanData { id: 1, value: 100 },
            CleanData { id: 2, value: 0 },
        ];
        let total = total_value(&data);
        assert_eq!(total, 100.0);
    }
    
    #[test]
    fn test_skip_zero_id() {
        let raw = vec![
            RawData { id: 0, value: 50 },
            RawData { id: 1, value: 50 },
        ];
        let cleaned = transform(extract(raw));
        assert_eq!(cleaned.len(), 1);
        assert_eq!(cleaned[0].id, 1);
    }
}
