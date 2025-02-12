mod chunking;
mod document_index;
mod inference;
mod language;

use std::str::FromStr;

use exports::pharia::skill::skill_handler::Error;
use pharia::skill;
use serde::{Deserialize, Serialize};

use crate::{
    ChatRequest, ChatResponse, ChunkRequest, Completion, CompletionRequest, Document, DocumentPath,
    LanguageCode, SearchRequest, SearchResult, SelectLanguageRequest,
};

wit_bindgen::generate!({
    world: "skill",
    path: "./src/wit",
    pub_export_macro: true,
    default_bindings_module: "pharia_skill::bindings",
    additional_derives: [PartialEq],
});

/// CSI implementation for the WASI environment.
pub struct WitCsi;

impl super::Csi for WitCsi {
    fn chunk_concurrently(&self, requests: Vec<ChunkRequest<'_>>) -> Vec<Vec<String>> {
        skill::chunking::chunk(&requests.into_iter().map(Into::into).collect::<Vec<_>>())
    }

    fn search_concurrently(&self, requests: Vec<SearchRequest<'_>>) -> Vec<Vec<SearchResult<'_>>> {
        skill::document_index::search(&requests.into_iter().map(Into::into).collect::<Vec<_>>())
            .into_iter()
            .map(|results| results.into_iter().map(Into::into).collect())
            .collect()
    }

    fn documents<Metadata>(
        &self,
        paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Document<'_, Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a>,
    {
        skill::document_index::documents(&paths.into_iter().map(Into::into).collect::<Vec<_>>())
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    fn documents_metadata<Metadata>(
        &self,
        paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Option<Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a>,
    {
        skill::document_index::document_metadata(
            &paths.into_iter().map(Into::into).collect::<Vec<_>>(),
        )
        .into_iter()
        .map(|v| v.map(|v| Ok(serde_json::from_slice(&v)?)).transpose())
        .collect()
    }

    fn chat_concurrently(&self, requests: Vec<ChatRequest<'_>>) -> Vec<ChatResponse<'_>> {
        skill::inference::chat(&requests.into_iter().map(Into::into).collect::<Vec<_>>())
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn complete_concurrently(&self, requests: Vec<CompletionRequest<'_>>) -> Vec<Completion<'_>> {
        skill::inference::complete(&requests.into_iter().map(Into::into).collect::<Vec<_>>())
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn select_language_concurrently(
        &self,
        requests: Vec<SelectLanguageRequest<'_>>,
    ) -> Vec<Option<LanguageCode>> {
        skill::language::select_language(&requests.into_iter().map(Into::into).collect::<Vec<_>>())
            .into_iter()
            .map(|l| l.map(|l| LanguageCode::from_str(&l).expect("Unknown language code")))
            .collect()
    }
}

/// Newtype so we can create `From` trait implementations for `anyhow::Result` and `String`.
pub struct HandlerResult<T: Serialize>(Result<T, Error>);

impl<T: Serialize> From<T> for HandlerResult<T> {
    fn from(value: T) -> Self {
        Self(Ok(value))
    }
}

impl<T: Serialize> From<anyhow::Result<T>> for HandlerResult<T> {
    fn from(result: anyhow::Result<T>) -> Self {
        match result {
            Ok(value) => Self(Ok(value)),
            Err(error) => Self(Err(Error::Internal(error.to_string()))),
        }
    }
}

impl<T: Serialize> From<HandlerResult<T>> for Result<Vec<u8>, Error> {
    fn from(value: HandlerResult<T>) -> Self {
        value.0.and_then(|v| json::to_vec(&v))
    }
}

/// JSON serialization and deserialization helpers for the main skill macro.
pub mod json {
    pub use schemars::{schema_for, JsonSchema};
    use serde::{Deserialize, Serialize};

    use super::Error;

    /// Convert input from the parent `run` method into the expected Input for the skill handler.
    ///
    /// # Errors
    /// Will error if the input cannot be deserialized into the expected Input type.
    pub fn from_slice<'input, Input>(input: &'input [u8]) -> Result<Input, Error>
    where
        Input: Deserialize<'input>,
    {
        serde_json::from_slice(input)
            .map_err(|error| Error::InvalidInput(anyhow::Error::from(error).to_string()))
    }

    /// Convert output from the skill handler to the expected output for the parent `run` method.
    ///
    /// # Errors
    /// Will error if the output cannot be serialized into the expected output type.
    pub fn to_vec<Output>(output: &Output) -> Result<Vec<u8>, Error>
    where
        Output: Serialize,
    {
        serde_json::to_vec(output)
            .map_err(|error| Error::Internal(anyhow::Error::from(error).to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_result() {
        let result = HandlerResult::from("Hello, world!");
        let output = Result::<Vec<u8>, Error>::from(result).unwrap();
        assert_eq!(output, b"\"Hello, world!\"".to_vec());
    }

    #[test]
    fn dont_serialize_error() {
        let result = HandlerResult::<&str>(Err(Error::Internal("Hello, world!".to_owned())));
        let error = Result::<Vec<u8>, Error>::from(result).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Error::Internal(\"Hello, world!\")".to_owned()
        );
    }
}
