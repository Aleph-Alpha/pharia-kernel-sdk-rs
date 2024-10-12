use std::time::Duration;

use pharia_skill::{Completion, CompletionRequest, Csi, FinishReason};
use serde::{de::DeserializeOwned, Serialize};
use ureq::{json, Agent, AgentBuilder};

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
    fn complete(&self, _request: &CompletionRequest<'_>) -> Completion {
        Completion {
            text: self.response.clone(),
            finish_reason: FinishReason::Stop,
        }
    }

    fn chunk(&self, text: &str, _params: &pharia_skill::ChunkParams<'_>) -> Vec<String> {
        vec![text.to_owned()]
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Function {
    Complete,
    CompleteAll,
    Chunk,
}

#[derive(Serialize)]
struct CsiRequest<'a, P: Serialize> {
    version: &'a str,
    function: Function,
    #[serde(flatten)]
    payload: P,
}

/// A Csi implementation that can be used for testing within normal Rust targets.
pub struct DevCsi {
    address: String,
    agent: Agent,
    token: String,
}

impl DevCsi {
    /// The version of the API we are calling against
    const VERSION: &str = "0.2";

    #[must_use]
    pub fn new(address: impl Into<String>, token: impl Into<String>) -> Self {
        let agent = AgentBuilder::new()
            .timeout(Duration::from_secs(60 * 5))
            .build();
        Self {
            address: address.into(),
            agent,
            token: token.into(),
        }
    }

    /// Construct a new [`DevCsi`] that points to the Aleph Alpha hosted Kernel
    pub fn aleph_alpha(token: impl Into<String>) -> Self {
        Self::new("https://pharia-kernel.aleph-alpha.stackit.run", token)
    }

    fn csi_request<R: DeserializeOwned>(&self, function: Function, payload: impl Serialize) -> R {
        let json = CsiRequest {
            version: Self::VERSION,
            function,
            payload,
        };
        self.agent
            .post(&format!("{}/csi", &self.address))
            .set("Authorization", &format!("Bearer {}", self.token))
            .send_json(json)
            .unwrap()
            .into_json::<R>()
            .unwrap()
    }
}

impl Csi for DevCsi {
    fn complete(&self, request: &CompletionRequest<'_>) -> Completion {
        self.csi_request(Function::Complete, request)
    }

    fn complete_all(&self, requests: &[CompletionRequest<'_>]) -> Vec<Completion> {
        self.csi_request(Function::CompleteAll, json!({"requests": requests}))
    }

    fn chunk(&self, text: &str, params: &pharia_skill::ChunkParams<'_>) -> Vec<String> {
        self.csi_request(Function::Chunk, json!({"text": text, "params": params}))
    }
}

#[cfg(test)]
mod tests {
    use pharia_skill::{ChunkParams, CompletionParams};

    use super::*;

    #[test]
    fn can_make_request() {
        drop(dotenvy::dotenv());

        let token = std::env::var("AA_API_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let response = csi.complete(&CompletionRequest::new(
            "llama-3.1-8b-instruct",
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>

Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant<|eot_id|><|start_header_id|>user<|end_header_id|>

What is the capital of France?<|eot_id|><|start_header_id|>assistant<|end_header_id|>",
            CompletionParams {
                stop: &["<|start_header_id|>".into()],
                ..Default::default()
            },
        ));
        assert_eq!(response.text.trim(), "The capital of France is Paris.");
    }

    #[test]
    fn can_make_multiple_requests() {
        drop(dotenvy::dotenv());

        let token = std::env::var("AA_API_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let params = CompletionParams {
            stop: &["<|start_header_id|>".into()],
            ..Default::default()
        };
        let completion_request = CompletionRequest::new(
            "llama-3.1-8b-instruct",
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>

Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant<|eot_id|><|start_header_id|>user<|end_header_id|>

What is the capital of France?<|eot_id|><|start_header_id|>assistant<|end_header_id|>",
            params,
        );

        let response = csi.complete_all(&vec![completion_request; 2]);
        assert!(response
            .into_iter()
            .all(|r| r.text.trim() == "The capital of France is Paris."));
    }

    #[test]
    fn chunk() {
        drop(dotenvy::dotenv());

        let token = std::env::var("AA_API_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let response = csi.chunk("123456", &ChunkParams::new("llama-3.1-8b-instruct", 1));

        assert_eq!(response, vec!["123", "456"]);
    }
}
