use std::collections::HashMap;
use std::env;
use std::fs;

fn main() {
    let filename = "data.txt";
    let content = fs::read_to_string(filename).unwrap();
    let lines: Vec<_> = content.split("\n").collect();
    let mut word_freq: HashMap<String, usize> = HashMap::new();
    lines.iter().for_each(|&s| {
        println!("{}", s);
        let words: Vec<_> = s.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation()  ).filter(|&s| !s.is_empty()).collect();
        words.iter().for_each(|key| {
            let lc_word = key.to_lowercase().to_string();

            *word_freq.entry(lc_word).or_insert(0 as usize) += 1;
        });
    });

    let mut sorted_vec: Vec<(_, _)> = word_freq.iter().collect();
    sorted_vec.sort_by(|a, b| b.1.cmp(a.1));

    println!("{:?}", sorted_vec);
}
