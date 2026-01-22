use polars::prelude::*;

fn main() -> PolarsResult<()> {
    // Create a simple DataFrame
    let df = df!(
        "fruit" => ["apple", "banana", "cherry", "apple", "banana"],
        "color" => ["red", "yellow", "red", "green", "yellow"],
        "price" => [1.2, 0.5, 2.0, 1.5, 0.6]
    )?;

    println!("=== Original DataFrame ===");
    println!("{}\n", df);

    // Filter: only red fruits
    let red_fruits = df.clone().lazy()
        .filter(col("color").eq(lit("red")))
        .collect()?;
    
    println!("=== Red Fruits ===");
    println!("{}\n", red_fruits);

    // Group by fruit and calculate mean price
    let avg_prices = df.clone().lazy()
        .group_by([col("fruit")])
        .agg([col("price").mean().alias("avg_price")])
        .sort(["fruit"], Default::default())
        .collect()?;

    println!("=== Average Price by Fruit ===");
    println!("{}\n", avg_prices);

    // Add a computed column
    let with_tax = df.lazy()
        .with_column((col("price") * lit(1.1)).alias("price_with_tax"))
        .collect()?;

    println!("=== With 10% Tax ===");
    println!("{}", with_tax);

    Ok(())
}
