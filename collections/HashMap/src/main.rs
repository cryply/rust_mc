/*
This example code counts the frequency of each number in the vector.
 */
use std::collections::{BTreeMap, HashMap};

fn logic(numbers: Vec<i32>) -> Vec<(i32, u32)> {
    let mut frequencies = HashMap::new();

    for num in numbers {
        let frequency = frequencies.entry(num).or_insert(0);
        *frequency += 1;
    }

    let mut result = Vec::new();

    let bf: BTreeMap<_, _> = frequencies.into_iter().collect();

    for (&num, frequency) in bf.iter() {
        println!("{num}");
        result.push((num, *frequency as u32));
    }

    result
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 3];
    let result = logic(numbers);
    //print the results in a human readable format that explains what the result is.
    println!(
        "The frequency of each number in the vector is: {:?}",
        result
    );
}
