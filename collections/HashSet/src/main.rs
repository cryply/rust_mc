use rand::prelude::*;
use std::collections::HashSet;

fn generate_fruit() -> String {
    let fruits = [
        "banana",
        "pear",
        "pineapple",
        "mango",
        "grapes",
        "passion fruit",
        "dates",
        "apple",
    ];

    // let x: u8 = rand::random();
    // let ind = x as usize % fruits.len();

    // let mut rng = rand::rng();
    // let ind: usize = rng.random_range(0..fruits.len());
    // String::from(fruits[ind])

    let mut rng = rand::rng();
    fruits.choose(&mut rng).unwrap().to_string()
}

fn main() {
    let mut hs = HashSet::new();
    println!("Hello, {}", generate_fruit());

    for _ in 1..100 {
        let my_fruit = generate_fruit();
        println!("adding {}", &my_fruit);
        hs.insert(my_fruit);
    }

    println!("{:?}", hs);
}
