mod analysis;
mod sentiment;

use clap::Parser;

#[derive(Parser)]
#[command(name = "crypto-sentiment", about = "Crypto news sentiment analyzer (Qwen via Ollama)")]
struct Args {
    /// News headline or text to analyze
    text: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let args = Args::parse();
    let ollama_url = std::env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    let client = sentiment::OllamaClient::new(&ollama_url);

    let (btc_score, eth_score) = tokio::try_join!(
        client.analyze_for_asset(&args.text, "Bitcoin"),
        client.analyze_for_asset(&args.text, "Ethereum"),
    )?;

    let result = analysis::build_analysis(&args.text, btc_score, eth_score);
    let json = serde_json::to_string_pretty(&result)?;
    println!("{json}");

    Ok(())
}
