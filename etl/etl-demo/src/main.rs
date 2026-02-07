use log::{error, info, warn};

use csv::{ReaderBuilder, Writer};
use serde::Serialize;
use std::{error::Error, io, process, thread::AccessError,
    fs::{File}
};


// ETL Example


#[derive(Debug, thiserror::Error)]
enum EtlError {
    #[error("Invalid data point: id={0}, reason{1}")]
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

fn total_value(data: Vec<CleanData>) -> f64 {
    data.into_iter().fold(0.0, |acc, p| acc + p.value as f64)
}

fn main() {
    env_logger::init();
    let raw = vec![RawData { id: 1, value: 1000 }, RawData { id: 2, value: -5 }];

    let cleaned = transform(extract(raw));

    println!(
        "Clean Data: Id - {:?} Value - {:?}",
        cleaned[0].id, cleaned[0].value
    ); // Accessing the fields

    println!("{:?}", cleaned);

    summary(cleaned.clone());

    load(cleaned);
}

fn summary(data: Vec<CleanData>) {
    let total: f64 = total_value(data.clone());
    let mean: f64 = total / data.len() as f64;
    println!("Total: {total} Mean: {mean}");
    
}


fn write_data(filename: String, data: Vec<CleanData>) -> Result<(), Box<dyn Error>> {
    let mut wrt = Writer::from_path(filename)?;
    for el in data {
        wrt.serialize(el)?;
    }
    wrt.flush();
    Ok(())
}

fn extract(raw: Vec<RawData>) -> Vec<RawData> {
    raw
}


// Perform ETL process
fn transform(raw: Vec<RawData>) -> Vec<CleanData> {
    raw.into_iter()
        .map(|r| {

            if r.id == 0 {

            }

            let ov = r.value;
            let clamped = r.value.clamp(0, 100);

            if clamped != ov {
                warn!("Value clamped {ov} -> {clamped}");
            }

            CleanData {
                id: r.id,
                value: clamped,
            }
        })
        .collect()
}


fn load(data: Vec<CleanData>) -> Result<(), Box<dyn Error>> {
    write_data("1.csv".to_string(), data);
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
        let total = total_value(data.clone());
        assert_eq!(total, 100.0);
    }
}
