use csv::{ReaderBuilder, Writer};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use std::{error::Error, io, process, thread::AccessError};
use rayon::prelude::*;


#[derive(Debug, Serialize, Clone)]
struct Product {
    nr: u32,
    name: String,
    quantity: u32,
    price: f64,
}

fn apply_discounts(products: impl IntoIterator<Item = Product>) -> impl Iterator<Item = Product> {
    products.into_iter().map(|mut p| {
        p.price *= 0.9;
        p
    })
}

fn apply_discount_pure(mut p: Product) -> Product {
    p.price *= 0.9;
    p
}

fn expensive_items(products: impl IntoIterator<Item = Product>) 
    -> impl Iterator<Item =(String, f64)>
{
    products
        .into_iter()
        .filter(|p| p.price > 100.0)
        .map(|p| (p.name, p.price * 0.9))
}


fn total_value(products: impl IntoIterator<Item = Product>) -> f64 {
    products.into_iter().fold(0.0, |acc, p| acc + p.price * p.quantity as f64)
}

fn write_products(file_name: String, products: Vec<Product>) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(file_name)?;

    // wtr.write_record(&["nr", "name", "quantity", "price"])?;

    for product in products {
        wtr.serialize(product)?;
    }

    wtr.flush()?;
    Ok(())
}

fn read_products() -> Result<Vec<Product>, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(std::io::stdin());

    let mut products: Vec<Product> = Vec::new();

    let mut i = 1;

    for result in rdr.records() {
        let record = result?;
        let name = &record[1];
        let price: f64 = record[3].parse().unwrap();

        products.push(Product {
            nr: i,
            name: name.to_string(),
            quantity: record[2].parse().unwrap(),
            price,
        });

        i += 1;
    }

    Ok(products)
}

#[derive(Debug, Serialize, Clone)]
struct Order { items: Vec<Product> }



fn all_discounted_items(orders: impl IntoIterator<Item = Order>) 
    -> impl Iterator<Item = Product> 
{
    orders.into_iter()
        .flat_map(|order| order.items)
        .map(|mut p| { p.price *= 0.9; p })
}
// Moves items out of Vecs — no cloning, no extra alloc.

fn main() {
    if let Ok(products) = read_products() {
        println!("{:?}", products);
        // 1. 
        let discounted: Vec<Product> = products.clone()
            .into_iter()
            .map(|p| Product{
                price: p.price * 0.9,
                ..p
            })
            .collect();
        _ = write_products("01_into_ter_map_classic.csv".to_string(), discounted);

        // 2. 
        _ = write_products("02_impl_trait_apply.csv".to_string(), apply_discounts(products.clone()).collect());

        // 3.

        _ = write_products("03_apply_pure.csv".to_string(), 
            products.clone().into_iter().map(apply_discount_pure).collect()
        );

        // 4. iter() + cloned()
        _ = write_products("04_iter_cloned.csv".to_string(), 
            products
                .iter()
                .cloned()
                .map(|mut p|{p.price *= 0.9 ; p }).collect()
        );

        // 5. for_each

        let mut discount = products.clone();
        discount.iter_mut().for_each(|p| p.price *= 0.9);
        _ = write_products("05_for_each.csv".to_string(), discount);

        // 6. fold pervertion

        let discounted: Vec<Product> = products.clone().into_iter()
            .fold(Vec::new(), |mut acc, mut p|{
                p.price *= 0.9;
                acc.push(p);
                acc
            });
        _ = write_products("06_fold.csv".to_string(), discounted);

        // 7 Scan pervertion
        let discounted: Vec<Product> = products
            .clone()
            .into_iter()
            .scan((), |_, mut p|{
                p.price *= 0.9;
                Some(p)
            })
            .collect();
        _ = write_products("07_scan.csv".to_string(), discounted);

        // 8 Conditional discount

        let discounted : Vec<Product> = products
            .clone()
            .into_iter()
            .map(|mut p|{
                if p.price >= 100.0 {
                    p.price *= 0.9;
                }
                p
            })
            .collect(); 
            _ = write_products("08_conditional_map.csv".to_string(), discounted);

        // 9 Zip

        let prices = products.iter().map(|p| p.price * 0.9);

        let discounted: Vec<Product> = products
            .clone()
            .into_iter()
            .zip(prices)
            .map(|(mut p, new_price)|{
                p.price = new_price;
                p
            })
            .collect();
        _ = write_products("09_zip.csv".to_string(), discounted);
        

        // 10 conditional impl

         for (name, disc) in expensive_items(products.clone()) { 
            println!("{} {:4.2}",  name, disc);
         }

         // 11 Total value

         println!("Total value: {}", total_value(products.clone()));

        // 11 FlatMap 


        let order = Order{
            items: products.clone()
        };
        let orders = vec![order];
        println!("Discounted: {:?}", all_discounted_items(orders).collect::<Vec<_>>());

        let many_products: Vec<_> = products.clone().iter().cycle().cloned().take(5).collect();
        

        let part_sum: Vec<_> = many_products
            .clone()
            .par_iter()
            .fold(|| 0f64, |acc, p| acc + (p.price * (p.quantity as f64)))
            .collect();

        let totsl_sum: f64 = part_sum
            .into_par_iter()
            .sum(); 


        println!("{:?}", &many_products);
        println!("Total sum:{}", totsl_sum);

    }
}
