use serde::Deserialize;

pub struct OllamaClient {
    http: reqwest::Client,
    base_url: String,
    model: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.to_string(),
            model: "qwen2.5:7b".to_string(),
        }
    }

    /// Score sentiment for a specific asset in the given text.
    /// Returns a score in [-1.0, 1.0]: -1 very bearish, +1 very bullish.
    pub async fn analyze_for_asset(&self, text: &str, asset: &str) -> anyhow::Result<f64> {
        let prompt = format!(
            "You are a crypto financial analyst. \
            Rate the sentiment towards {asset} in the news headline below. \
            Reply with ONLY a single decimal number from -1.0 (very bearish) to 1.0 (very bullish). \
            No explanation, no units, just the number.\n\
            Headline: {text}"
        );

        let payload = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": { "temperature": 0 }
        });

        let resp = self
            .http
            .post(format!("{}/api/generate", self.base_url))
            .json(&payload)
            .send()
            .await?;

        let body: OllamaResponse = resp.json().await?;
        let score: f64 = body.response.trim().parse().map_err(|_| {
            anyhow::anyhow!("Failed to parse score from model output: {:?}", body.response)
        })?;

        Ok(score.clamp(-1.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_clamp_overflow() {
        let score = 1.5_f64.clamp(-1.0, 1.0);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_clamp_underflow() {
        let score = (-1.5_f64).clamp(-1.0, 1.0);
        assert_eq!(score, -1.0);
    }

    #[test]
    fn test_parse_valid_score() {
        let raw = " -0.75 ";
        let score: f64 = raw.trim().parse().unwrap();
        assert!((score - (-0.75)).abs() < 1e-9);
    }
}
