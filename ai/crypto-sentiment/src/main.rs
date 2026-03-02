mod analysis;
mod sentiment;

use clap::Parser;

#[derive(Parser)]
#[command(name = "crypto-sentiment", about = "Crypto tweet sentiment analyzer")]
struct Args {
    /// Text to analyze
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args = Args::parse();
    let hf_token = std::env::var("HF_API_TOKEN")
        .expect("HF_API_TOKEN must be set in .env or environment");

    let sentiment_client = sentiment::SentimentClient::new(&hf_token);

    let score = sentiment_client.analyze(&args.text).await?;

    let result = analysis::build_analysis(&args.text, score);
    let json = serde_json::to_string_pretty(&result)?;
    println!("{json}");

    Ok(())
}
