use std::time::Duration;

use pharia_skill::{Completion, CompletionParams, Csi, FinishReason};
use ureq::{json, serde_json::Value, Agent, AgentBuilder};

pub struct MockCsi {
    response: String,
}

impl MockCsi {
    #[must_use]
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }
}

impl Csi for MockCsi {
    fn complete(
        &self,
        _model: impl Into<String>,
        _prompt: impl ToString,
        _params: CompletionParams,
    ) -> Completion {
        Completion {
            text: self.response.clone(),
            finish_reason: FinishReason::Stop,
        }
    }
}

pub struct SaboteurCsi;

impl Csi for SaboteurCsi {
    fn complete(
        &self,
        _model: impl Into<String>,
        _prompt: impl ToString,
        _params: CompletionParams,
    ) -> Completion {
        panic!("sabotage")
    }
}

/// A Csi implementation that can be used for testing within normal Rust targets.
pub struct TestCsi {
    address: String,
    agent: Agent,
    token: String,
}

impl TestCsi {
    #[must_use]
    pub fn new(address: impl Into<String>, token: impl Into<String>) -> Self {
        let agent = AgentBuilder::new().timeout(Duration::from_secs(60)).build();
        Self {
            address: address.into(),
            agent,
            token: token.into(),
        }
    }

    pub fn aleph_alpha(token: impl Into<String>) -> Self {
        let address = "https://api.aleph-alpha.com";
        Self::new(address, token)
    }

    fn complete_reqest(
        &self,
        model: impl Into<String>,
        prompt: &str,
        params: CompletionParams,
    ) -> Completion {
        let CompletionParams {
            max_tokens,
            temperature,
            top_k,
            top_p,
            stop,
        } = params;
        let json = json!({
            "prompt": prompt.to_string(),
            "model": model.into(),
            "maximum_tokens": max_tokens,
            "temperature": temperature,
            "top_k": top_k,
            "top_p": top_p,
            "stop_sequences": stop
        });
        let resp = self
            .agent
            .post(&format!("{}/complete", &self.address))
            .set("Authorization", &format!("Bearer {}", self.token))
            .send_json(json)
            .unwrap()
            .into_json::<Value>()
            .unwrap();

        let completion = &resp["completions"][0];

        let finish_reason = match completion["finish_reason"].as_str().unwrap() {
            "stop" | "end_of_text" => FinishReason::Stop,
            "length" | "maximum_tokens" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            s => panic!("Invalid FinishReason: {s}"),
        };
        Completion {
            text: completion["completion"].as_str().unwrap().to_owned(),
            finish_reason,
        }
    }
}

impl Csi for TestCsi {
    fn complete(
        &self,
        model: impl Into<String>,
        prompt: impl ToString,
        params: CompletionParams,
    ) -> Completion {
        self.complete_reqest(model, &prompt.to_string(), params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_make_request() {
        drop(dotenvy::dotenv());

        let token = std::env::var("SPIN_VARIABLE_AA_API_TOKEN").unwrap();
        let csi = TestCsi::aleph_alpha(token);

        let response = csi.complete(
            "llama-3.1-8b-instruct",
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>

Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant<|eot_id|><|start_header_id|>user<|end_header_id|>

What is the capital of France?<|eot_id|><|start_header_id|>assistant<|end_header_id|>",
            CompletionParams {
                stop: vec![
                    "<|start_header_id|>".to_owned(),
                    "<|eom_id|>".to_owned(),
                    "<|eot_id|>".to_owned(),
                ],
                ..Default::default()
            },
        );
        assert_eq!(response.text.trim(), "The capital of France is Paris.");
    }
}
