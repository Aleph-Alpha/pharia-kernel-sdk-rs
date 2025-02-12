use std::{borrow::Cow, time::Duration};

use pharia_skill::{
    ChatRequest, ChatResponse, ChunkRequest, Completion, CompletionRequest, Csi, Document,
    DocumentPath, FinishReason, LanguageCode, Message, SearchRequest, SearchResult,
    SelectLanguageRequest, TokenUsage,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use ureq::{json, serde_json::Value, Agent, AgentBuilder};

pub struct StubCsi;

impl Csi for StubCsi {
    fn chat_concurrently(&self, requests: Vec<ChatRequest<'_>>) -> Vec<ChatResponse<'_>> {
        requests
            .iter()
            .map(|_| ChatResponse {
                message: Message::new("user", ""),
                finish_reason: FinishReason::Stop,
            })
            .collect()
    }

    fn complete_concurrently(&self, requests: Vec<CompletionRequest<'_>>) -> Vec<Completion<'_>> {
        requests
            .into_iter()
            .map(|request| Completion {
                text: request.prompt.into_owned().into(),
                finish_reason: FinishReason::Stop,
                logprobs: Cow::Borrowed(&[]),
                usage: TokenUsage {
                    prompt: 0,
                    completion: 0,
                },
            })
            .collect()
    }

    fn chunk_concurrently(&self, requests: Vec<ChunkRequest<'_>>) -> Vec<Vec<String>> {
        requests
            .into_iter()
            .map(|request| vec![request.text.into_owned()])
            .collect()
    }

    fn select_language_concurrently(
        &self,
        requests: Vec<SelectLanguageRequest<'_>>,
    ) -> Vec<Option<LanguageCode>> {
        requests.iter().map(|_| None).collect()
    }

    fn search_concurrently(&self, _requests: Vec<SearchRequest<'_>>) -> Vec<Vec<SearchResult<'_>>> {
        vec![]
    }

    fn documents<Metadata>(
        &self,
        _paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Document<'_, Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a>,
    {
        Ok(vec![])
    }

    fn documents_metadata<Metadata>(
        &self,
        _paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Option<Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a>,
    {
        Ok(vec![])
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
    fn chat_concurrently(&self, requests: Vec<ChatRequest<'_>>) -> Vec<ChatResponse<'_>> {
        requests
            .iter()
            .map(|_| ChatResponse {
                message: Message::new("user", self.response.clone()),
                finish_reason: FinishReason::Stop,
            })
            .collect()
    }

    fn complete_concurrently(&self, requests: Vec<CompletionRequest<'_>>) -> Vec<Completion<'_>> {
        requests
            .iter()
            .map(|_| Completion {
                text: Cow::Borrowed(&self.response),
                finish_reason: FinishReason::Stop,
                logprobs: Cow::Borrowed(&[]),
                usage: TokenUsage {
                    prompt: 0,
                    completion: 0,
                },
            })
            .collect()
    }

    fn chunk_concurrently(&self, requests: Vec<ChunkRequest<'_>>) -> Vec<Vec<String>> {
        requests
            .into_iter()
            .map(|request| vec![request.text.into_owned()])
            .collect()
    }

    fn select_language_concurrently(
        &self,
        requests: Vec<SelectLanguageRequest<'_>>,
    ) -> Vec<Option<LanguageCode>> {
        requests.iter().map(|_| None).collect()
    }

    fn search_concurrently(&self, _requests: Vec<SearchRequest<'_>>) -> Vec<Vec<SearchResult<'_>>> {
        vec![]
    }

    fn documents<Metadata>(
        &self,
        _paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Document<'_, Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a>,
    {
        Ok(vec![])
    }

    fn documents_metadata<Metadata>(
        &self,
        _paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Option<Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a>,
    {
        Ok(vec![])
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum Function {
    Complete,
    Chunk,
    SelectLanguage,
    Search,
    Chat,
    Documents,
    DocumentMetadata,
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
    const VERSION: &str = "0.3";

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
        Self::new("https://pharia-kernel.product.pharia.com", token)
    }

    fn csi_request<R: DeserializeOwned>(
        &self,
        function: Function,
        payload: impl Serialize,
    ) -> anyhow::Result<R> {
        let json = CsiRequest {
            version: Self::VERSION,
            function,
            payload,
        };
        let response = self
            .agent
            .post(&format!("{}/csi", &self.address))
            .set("Authorization", &format!("Bearer {}", self.token))
            .send_json(json);

        match response {
            Ok(response) => Ok(response.into_json::<R>()?),
            Err(ureq::Error::Status(status, response)) => {
                panic!(
                    "Failed Request: Status {status} {}",
                    response.into_json::<Value>().unwrap_or_default()
                );
            }
            Err(e) => {
                panic!("{e}")
            }
        }
    }
}

impl Csi for DevCsi {
    fn chat_concurrently(&self, requests: Vec<ChatRequest<'_>>) -> Vec<ChatResponse<'_>> {
        self.csi_request(Function::Chat, json!({"requests": requests}))
            .unwrap()
    }

    fn complete_concurrently(&self, requests: Vec<CompletionRequest<'_>>) -> Vec<Completion<'_>> {
        self.csi_request(Function::Complete, json!({"requests": requests}))
            .unwrap()
    }

    fn chunk_concurrently(&self, requests: Vec<ChunkRequest<'_>>) -> Vec<Vec<String>> {
        self.csi_request(Function::Chunk, json!({"requests": requests}))
            .unwrap()
    }

    fn select_language_concurrently(
        &self,
        requests: Vec<SelectLanguageRequest<'_>>,
    ) -> Vec<Option<LanguageCode>> {
        self.csi_request(Function::SelectLanguage, json!({"requests": requests}))
            .unwrap()
    }

    fn search_concurrently(&self, requests: Vec<SearchRequest<'_>>) -> Vec<Vec<SearchResult<'_>>> {
        self.csi_request(Function::Search, json!({"requests": requests}))
            .unwrap()
    }

    fn documents<Metadata>(
        &self,
        paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Document<'_, Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a> + Serialize,
    {
        Ok(self
            .csi_request::<Vec<Document<'_, Metadata>>>(
                Function::Documents,
                json!({"requests": paths}),
            )?
            .into_iter()
            .collect())
    }

    fn documents_metadata<Metadata>(
        &self,
        paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Option<Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a> + Serialize,
    {
        Ok(self
            .csi_request::<Vec<Option<Metadata>>>(
                Function::DocumentMetadata,
                json!({"requests": paths}),
            )?
            .into_iter()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;
    use pharia_skill::{
        ChatParams, ChunkParams, ChunkRequest, CompletionParams, IndexPath, Modality,
    };

    use super::*;

    #[test]
    fn can_make_request() {
        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let response = csi.complete(
            CompletionRequest::new(
                "llama-3.1-8b-instruct",
                "<|begin_of_text|><|start_header_id|>system<|end_header_id|>

Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant<|eot_id|><|start_header_id|>user<|end_header_id|>

What is the capital of France?<|eot_id|><|start_header_id|>assistant<|end_header_id|>",
            )
            .with_params(CompletionParams {
                stop: vec!["<|start_header_id|>".into()].into(),
                max_tokens: Some(10),
                ..Default::default()
            }),
        );
        assert_eq!(
            response.text.trim(),
            "The capital of France is Paris.<|eot_id|>"
        );
    }

    #[test]
    fn can_make_multiple_requests() {
        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let params = CompletionParams {
            stop: vec!["<|start_header_id|>".into()].into(),
            max_tokens: Some(10),
            ..Default::default()
        };
        let completion_request = CompletionRequest::new(
            "llama-3.1-8b-instruct",
            "<|begin_of_text|><|start_header_id|>system<|end_header_id|>

Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant<|eot_id|><|start_header_id|>user<|end_header_id|>

What is the capital of France?<|eot_id|><|start_header_id|>assistant<|end_header_id|>",
        )
        .with_params(params);

        let response = csi.complete_concurrently(vec![completion_request; 2]);
        assert!(response
            .into_iter()
            .all(|r| r.text.trim() == "The capital of France is Paris.<|eot_id|>"));
    }

    #[test]
    fn chunk() {
        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let response = csi.chunk(ChunkRequest::new(
            "123456",
            ChunkParams::new("llama-3.1-8b-instruct", 1),
        ));

        assert_eq!(response, vec!["123", "456"]);
    }

    #[test]
    fn select_language() {
        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let response = csi.select_language(SelectLanguageRequest::new(
            "A rising tide lifts all boats",
            &[LanguageCode::Eng, LanguageCode::Deu, LanguageCode::Fra],
        ));

        assert_eq!(response, Some(LanguageCode::Eng));
    }

    #[test]
    fn search() {
        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let response = csi.search(
            SearchRequest::new("decoder", IndexPath::new("Kernel", "test", "asym-64"))
                .with_max_results(10),
        );

        assert!(!response.is_empty());
    }

    #[test]
    fn chat() {
        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let request = ChatRequest::new(
            "llama-3.1-8b-instruct",
            Message::user("Hello, how are you?"),
        )
        .with_params(ChatParams {
            max_tokens: Some(1),
            ..Default::default()
        });
        let response = csi.chat(request);

        assert!(!response.message.content.is_empty());
    }

    #[test]
    fn documents() {
        #[derive(Debug, Deserialize, Serialize)]
        struct Metadata {
            created: Timestamp,
            url: String,
        }

        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let path = DocumentPath::new("Kernel", "test", "kernel-docs");
        let response = csi.document::<Metadata>(path.clone()).unwrap();

        assert_eq!(response.path, path);
        assert_eq!(response.contents.len(), 1);
        assert!(
            matches!(&response.contents[0], Modality::Text { text } if text.contains("Kernel"))
        );
    }

    #[test]
    fn document_metadata() {
        #[derive(Debug, Deserialize, Serialize)]
        struct Metadata {
            created: Timestamp,
            url: String,
        }

        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let path = DocumentPath::new("Kernel", "test", "kernel-docs");
        let response = csi.document_metadata::<Metadata>(path.clone()).unwrap();

        assert!(response.is_some());
    }

    #[test]
    fn invalid_metadata() {
        drop(dotenvy::dotenv());

        let token = std::env::var("PHARIA_AI_TOKEN").unwrap();
        let csi = DevCsi::aleph_alpha(token);

        let path = DocumentPath::new("Kernel", "test", "kernel-docs");
        let response = csi.document_metadata::<String>(path.clone());

        assert!(response.is_err());
    }
}
