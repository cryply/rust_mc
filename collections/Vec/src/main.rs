// De facto Stack.

fn main() {
    let fruits = vec![
        "banana",
        "pear",
        "pineapple",
        "mango",
        "grapes",
        "passion fruit",
        "dates",
        "apple",
    ];

    let mut vec_fruits = Vec::new();

    for f in &fruits {
        vec_fruits.push(f);
    }

    println!("{:?}", vec_fruits);

    vec_fruits.pop();

    println!("{:?}", vec_fruits);

    println!("First element? {}", vec_fruits[0]);

    let new = ["mushrooms"];

    // Splice inserting subarray in array
    // returning replaced part.
    let u: Vec<_> = vec_fruits.splice(1..3, &new).collect();

    println!("1. {:?}", u);
    println!("2. {:?}", vec_fruits);

    // Insert
    vec_fruits.insert(1, &"passion fruit");

    println!("{:?}", vec_fruits);

    // Retain
    vec_fruits.retain(|s| s.len() > 5);
    println!("{:?}", vec_fruits);

    let mut sv: Vec<_> = vec_fruits.clone();
    sv.sort();

    // sv.remove("banana");
    sv.retain(|s| **s != "banana");
    println!("{:?}", sv);

}
