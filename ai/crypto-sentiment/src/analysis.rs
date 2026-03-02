use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SentimentScore {
    pub sentiment: f64,
}

#[derive(Debug, Serialize)]
pub struct Analysis {
    pub text: String,
    pub btc: SentimentScore,
    pub eth: SentimentScore,
}

/// Check if text mentions BTC/Bitcoin (case-insensitive).
pub fn mentions_btc(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("btc") || lower.contains("bitcoin")
}

/// Check if text mentions ETH/Ethereum (case-insensitive).
pub fn mentions_eth(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("eth") || lower.contains("ethereum")
}

/// Build a TweetAnalysis from tweet data and sentiment score.
/// Returns None if the tweet mentions neither BTC nor ETH.
pub fn build_analysis(text: &str, sentiment: f64) -> Analysis {
    Analysis {
        text: text.to_string(),
        btc: SentimentScore { sentiment },
        eth: SentimentScore { sentiment },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mentions_btc() {
        assert!(mentions_btc("BTC is up"));
        assert!(mentions_btc("bitcoin breaking out"));
        assert!(mentions_btc("I love BITCOIN"));
        assert!(!mentions_btc("Ethereum is cool"));
    }

    #[test]
    fn test_mentions_eth() {
        assert!(mentions_eth("ETH looking strong"));
        assert!(mentions_eth("Ethereum merge complete"));
        assert!(!mentions_eth("BTC only"));
    }

    #[test]
    fn test_build_analysis_always_has_both() {
        let a = build_analysis("BTC pumping hard", 0.72);
        assert!((a.btc.sentiment - 0.72).abs() < 1e-9);
        assert!((a.eth.sentiment - 0.72).abs() < 1e-9);
    }

    #[test]
    fn test_build_analysis_unrelated_text() {
        let a = build_analysis("stocks are boring", 0.1);
        assert!((a.btc.sentiment - 0.1).abs() < 1e-9);
        assert!((a.eth.sentiment - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_json_output_format() {
        let a = Analysis {
            text: "BTC breaking out".to_string(),
            btc: SentimentScore { sentiment: 0.72 },
            eth: SentimentScore { sentiment: 0.72 },
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"sentiment\":0.72"));
        assert!(json.contains("\"btc\""));
        assert!(json.contains("\"eth\""));
    }
}
