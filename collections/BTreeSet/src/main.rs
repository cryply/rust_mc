use rand::prelude::*;
use std::collections::BTreeSet;

fn main() {
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

    let mut vs_fruits = fruits.clone();

    let mut bts_fruits = BTreeSet::new();
    let mut rng = rand::rng();

    vs_fruits.shuffle(&mut rng);

    println!("Shuffled: {:?}", vs_fruits);

    for _ in 0..1000 {
        bts_fruits.insert(fruits.choose(&mut rng).unwrap());
    }

    println!("{:?}", bts_fruits);

    if !bts_fruits.contains(&"rotten banana") {
        println!("rotten banana not found");
    } else {
        println!("rotten banana  found");
    }

    bts_fruits.remove(&"banana");
    println!("{:?}", bts_fruits);

    bts_fruits.iter().for_each(|x| {
        println!("{}", &x);
    });

    println!("-----------------");
    for &el in bts_fruits.range("dates"..="pear") {
        println!("{}", el);
    }
}
