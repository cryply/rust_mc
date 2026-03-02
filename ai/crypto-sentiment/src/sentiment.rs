use serde::Deserialize;

pub struct SentimentClient {
    http: reqwest::Client,
    api_token: String,
}

#[derive(Debug, Deserialize)]
struct HfLabel {
    label: String,
    score: f64,
}

impl SentimentClient {
    pub fn new(api_token: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_token: api_token.to_string(),
        }
    }

    /// Call ProsusAI/finbert via HuggingFace Inference API.
    /// Returns a score in [-1.0, 1.0] where positive_prob - negative_prob.
    pub async fn analyze(&self, text: &str) -> anyhow::Result<f64> {
        let url = "https://router.huggingface.co/hf-inference/models/ProsusAI/finbert";

        let payload = serde_json::json!({ "inputs": text });
        let resp = self
            .http
            .post(url)
            .bearer_auth(&self.api_token)
            .json(&payload)
            .send()
            .await?;

        let body = resp.text().await?;
        // HF returns [[{label, score}, ...]] — outer array per input
        let outer: Vec<Vec<HfLabel>> = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("HF API response parse error: {e}\nbody: {body}"))?;

        let labels = outer
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty HF response"))?;

        let mut positive = 0.0_f64;
        let mut negative = 0.0_f64;

        for item in &labels {
            match item.label.to_lowercase().as_str() {
                "positive" => positive = item.score,
                "negative" => negative = item.score,
                _ => {}
            }
        }

        Ok(positive - negative)
    }
}

#[cfg(test)]
mod tests {
    fn compute_score(positive: f64, negative: f64) -> f64 {
        positive - negative
    }

    #[test]
    fn test_score_positive() {
        let score = compute_score(0.85, 0.05);
        assert!((score - 0.80).abs() < 1e-9);
        assert!(score > 0.0);
    }

    #[test]
    fn test_score_negative() {
        let score = compute_score(0.1, 0.8);
        assert!((score - (-0.7)).abs() < 1e-9);
        assert!(score < 0.0);
    }

    #[test]
    fn test_score_neutral() {
        let score = compute_score(0.33, 0.33);
        assert!(score.abs() < 1e-9);
    }

    #[test]
    fn test_score_bounds() {
        assert!((compute_score(1.0, 0.0) - 1.0).abs() < 1e-9);
        assert!((compute_score(0.0, 1.0) - (-1.0)).abs() < 1e-9);
    }
}
