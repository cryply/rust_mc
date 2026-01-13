use rand::prelude::*;
use std::cmp::*;
use std::collections::BinaryHeap;

#[derive(Eq, PartialEq)]
enum Fruit {
    Fig,
    Other(String),
}

impl Ord for Fruit {
    fn cmp(&self, other: &Self) -> Ordering {
        self.is_fig().cmp(&other.is_fig())
    }
}

impl PartialOrd for Fruit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Fruit {
    fn is_fig(&self) -> bool {
        matches!(self, Fruit::Fig)
    }
}

fn generate_fruit_salad() -> BinaryHeap<Fruit> {
    let fruits = [
        "banana",
        "pear",
        "pineapple",
        "mango",
        "grapes",
        "passion fruit",
        "dates",
        "apple",
        "fig",
        "fig",
        "fig",
        "fig",
        "fig",
        "fig",
    ];

    let mut rng = rand::rng();

    let mut fruit_salad = BinaryHeap::new();

    let mut fig_count = 0;

    while fig_count < 2 {
        let fruit = fruits.choose(&mut rng).unwrap();
        if *fruit == "fig" {
            fig_count += 1;
            fruit_salad.push(Fruit::Fig);
        } else {
            fruit_salad.push(Fruit::Other(fruit.to_string()));
        }
    }

    fruit_salad
}

fn main() {
    let fruit_salad = generate_fruit_salad();
    println!("Random Fruit Salad With Two Servings of Figs:");
    for fruit in fruit_salad {
        match fruit {
            Fruit::Fig => println!(" - Fig"),
            Fruit::Other(fruit_name) => println!(" - {}", fruit_name),
        }
    }
}
