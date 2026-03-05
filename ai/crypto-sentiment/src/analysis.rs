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

pub fn build_analysis(text: &str, btc_sentiment: f64, eth_sentiment: f64) -> Analysis {
    Analysis {
        text: text.to_string(),
        btc: SentimentScore { sentiment: btc_sentiment },
        eth: SentimentScore { sentiment: eth_sentiment },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

#[test]
    fn test_build_analysis_separate_scores() {
        let a = build_analysis("BTC pumping while ETH drops", 0.72, -0.45);
        assert!((a.btc.sentiment - 0.72).abs() < 1e-9);
        assert!((a.eth.sentiment - (-0.45)).abs() < 1e-9);
    }

    #[test]
    fn test_build_analysis_unrelated_text() {
        let a = build_analysis("stocks are boring", 0.1, 0.1);
        assert!((a.btc.sentiment - 0.1).abs() < 1e-9);
        assert!((a.eth.sentiment - 0.1).abs() < 1e-9);
    }

    #[test]
    fn test_json_output_format() {
        let a = Analysis {
            text: "BTC breaking out".to_string(),
            btc: SentimentScore { sentiment: 0.72 },
            eth: SentimentScore { sentiment: -0.30 },
        };
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("\"btc\""));
        assert!(json.contains("\"eth\""));
    }
}
