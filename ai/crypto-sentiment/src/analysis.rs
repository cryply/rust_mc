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
