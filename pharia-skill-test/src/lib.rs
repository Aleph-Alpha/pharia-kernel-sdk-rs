use std::time::Duration;

use pharia_skill::{Completion, CompletionParams, Csi, FinishReason};
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

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Function {
    Complete,
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
    fn complete(
        &self,
        model: impl Into<String>,
        prompt: impl ToString,
        params: CompletionParams,
    ) -> Completion {
        self.csi_request(
            Function::Complete,
            json!({
                "model": model.into(),
                "prompt": prompt.to_string(),
                "params": params,
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_make_request() {
        drop(dotenvy::dotenv());

        let token = std::env::var("AA_API_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

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
