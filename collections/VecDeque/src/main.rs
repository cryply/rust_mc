
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

    for f in fruits {
        vec_fruits.push(f);
    }

    println!("{:?}", vec_fruits);
}
