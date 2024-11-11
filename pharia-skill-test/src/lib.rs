use std::time::Duration;

use pharia_skill::{
    ChatRequest, ChatResponse, Completion, CompletionRequest, Csi, FinishReason, IndexPath,
    Language, Message, Role, SearchResult,
};
use serde::{de::DeserializeOwned, Serialize};
use ureq::{json, Agent, AgentBuilder};

pub struct StubCsi;

impl Csi for StubCsi {
    fn complete(&self, request: &CompletionRequest<'_>) -> Completion {
        Completion {
            text: request.prompt.clone().into_owned(),
            finish_reason: FinishReason::Stop,
        }
    }

    fn chunk(&self, text: &str, _params: &pharia_skill::ChunkParams<'_>) -> Vec<String> {
        vec![text.to_owned()]
    }

    fn select_language(&self, _text: &str, _languages: &[Language]) -> Option<Language> {
        None
    }

    fn search(
        &self,
        _index: &IndexPath<'_>,
        _query: &str,
        _max_results: u32,
        _min_score: Option<f64>,
    ) -> Vec<SearchResult> {
        vec![]
    }

    fn chat(&self, _request: &ChatRequest<'_>) -> ChatResponse<'_> {
        ChatResponse {
            message: Message::new(Role::User, ""),
            finish_reason: FinishReason::Stop,
        }
    }
}

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

    fn select_language(
        &self,
        _text: &str,
        _languages: &[pharia_skill::Language],
    ) -> Option<pharia_skill::Language> {
        None
    }

    fn search(
        &self,
        _index: &IndexPath<'_>,
        _query: &str,
        _max_results: u32,
        _min_score: Option<f64>,
    ) -> Vec<SearchResult> {
        vec![]
    }

    fn chat(&self, _request: &ChatRequest<'_>) -> ChatResponse<'_> {
        ChatResponse {
            message: Message::new(Role::User, self.response.clone()),
            finish_reason: FinishReason::Stop,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Function {
    Complete,
    CompleteAll,
    Chunk,
    SelectLanguage,
    Search,
    Chat,
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

    fn select_language(
        &self,
        text: &str,
        languages: &[pharia_skill::Language],
    ) -> Option<pharia_skill::Language> {
        self.csi_request(
            Function::SelectLanguage,
            json!({"text": text, "languages": languages}),
        )
    }

    fn search(
        &self,
        index: &IndexPath<'_>,
        query: &str,
        max_results: u32,
        min_score: Option<f64>,
    ) -> Vec<SearchResult> {
        self.csi_request(Function::Search, json!({"index_path": index, "query": query, "max_results": max_results, "min_score": min_score}))
    }

    fn chat(&self, request: &ChatRequest<'_>) -> ChatResponse<'_> {
        self.csi_request(Function::Chat, request)
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

    #[test]
    fn select_language() {
        drop(dotenvy::dotenv());

        let token = std::env::var("AA_API_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let response = csi.select_language("A rising tide lifts all boats", &Language::all());

        assert_eq!(response, Some(Language::Eng));
    }

    #[test]
    fn search() {
        drop(dotenvy::dotenv());

        let token = std::env::var("AA_API_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let response = csi.search(
            &IndexPath::new("aleph-alpha", "pharia-kernel-demo-collection", "asym-256"),
            "decoder",
            10,
            None,
        );

        assert!(!response.is_empty());
    }

    #[test]
    fn chat() {
        drop(dotenvy::dotenv());

        let token = std::env::var("AA_API_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let request = ChatRequest::new(
            "llama-3.1-8b-instruct",
            Message::user("Hello, how are you?"),
        );
        let response = csi.chat(&request);

        assert!(!response.message.content.is_empty());
    }
}
