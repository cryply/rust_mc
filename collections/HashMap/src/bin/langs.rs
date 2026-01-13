use std::collections::HashMap;
use itertools::Itertools;

fn init_languages() -> HashMap<String, (i32, u64)> {
    let mut languages = HashMap::new();
    languages.insert("JavaScript".to_string(), (1995, 10000000));
    languages.insert("HTML/CSS".to_string(), (1990, 2000000));
    languages.insert("Python".to_string(), (1991, 20303003));
    languages.insert("SQL".to_string(), (1974, 12123));
    languages.insert("TypeScript".to_string(), (2012, 1234));
    languages.insert("Bash/Shell".to_string(), (1989, 3232));
    languages.insert("Java".to_string(), (1995, 434334));
    languages.insert("C#".to_string(), (2000, 343434));
    languages.insert("C++".to_string(), (1985, 2323));
    languages.insert("C".to_string(), (1972, 343435));
    languages.insert("PHP".to_string(), (1995, 2343445));
    languages.insert("PowerShell".to_string(), (2006, 232));
    languages.insert("Go".to_string(), (2007, 344));
    languages.insert("Rust".to_string(), (2010, 45));

    languages
}

fn calculate_weights(years_active: &mut HashMap<String,(i32, u64)>) -> HashMap<String, i32> {
    // Subtract the creation year from 2024 to get the number of years active.
    for (year, _) in years_active.values_mut() {
        *year = 2025 - *year;
    }

    let min_year = *years_active.values().min_by(|a,b| a.0.cmp(&b.0)).map(|(year, _)| year).unwrap_or(&0);
    let max_year = *years_active.values().max_by(|a,b| a.0.cmp(&b.0)).map(|(year, _)| year).unwrap_or(&0);

    let mut weights = HashMap::new();

    for (language, (year, _)) in years_active.iter() {
        let normalized_year = (year - min_year) as f64 / (max_year - min_year) as f64;
        let weight = (normalized_year * 99.0) as i32 + 1; // weight between 1 and 100
        weights.insert(language.to_string(), weight);
    }

    weights
}

fn main() {
    let mut languages = init_languages();
    let mut weights : Vec<_> = calculate_weights(&mut languages).into_iter().collect();

    // weights.sort_by(|a,b| a.1.cmp(&b.1));
    weights.sort_by_key(|item| item.1);    

    println!("Language weighing from 1-100 by age (1 is newest and 100 is oldest):");
    for (language, weight) in weights {
        println!("{}: {}", language, weight);
    }
}
