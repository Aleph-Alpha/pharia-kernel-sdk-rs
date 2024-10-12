pub mod prompt;

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub use bindings::exports::pharia::skill::skill_handler::Error;
/// Macro to define a Skill. It wraps a function that takes a single argument and returns a single value.
pub use pharia_skill_macros::skill;

/// Cognitive System Interface
pub trait Csi {
    /// Generate a completion for a given prompt using a specific model.
    fn complete(&self, request: &CompletionRequest<'_>) -> Completion;

    /// Process multiple completion requests at once
    fn complete_all(&self, requests: &[CompletionRequest<'_>]) -> Vec<Completion> {
        requests
            .iter()
            .map(|request| self.complete(request))
            .collect()
    }

    /// Chunk the given text into smaller pieces that fit within the
    /// maximum token amount for a given model.
    fn chunk(&self, text: &str, params: &ChunkParams<'_>) -> Vec<String>;
}

/// The reason that the model stopped completing text
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// The model hit a natural stopping point or a provided stop sequence
    Stop,
    /// The maximum number of tokens specified in the request was reached
    Length,
    /// Content was omitted due to a flag from content filters
    ContentFilter,
}

/// The result of a completion, including the text generated as well as
/// why the model finished completing.
#[derive(Clone, Debug, Deserialize)]
pub struct Completion {
    /// The text generated by the model
    pub text: String,
    /// The reason the model finished generating
    pub finish_reason: FinishReason,
}

/// Completion request parameters
#[derive(Clone, Debug, Default, Serialize)]
pub struct CompletionParams<'a> {
    /// The maximum tokens that should be inferred.
    ///
    /// Note: the backing implementation may return less tokens due to
    /// other stop reasons.
    pub max_tokens: Option<u32>,
    /// The randomness with which the next token is selected.
    pub temperature: Option<f64>,
    /// The number of possible next tokens the model will choose from.
    pub top_k: Option<u32>,
    /// The probability total of next tokens the model will choose from.
    pub top_p: Option<f64>,
    /// A list of sequences that, if encountered, the API will stop generating further tokens.
    pub stop: &'a [Cow<'a, str>],
}

/// Parameters required to make a completion request.
#[derive(Clone, Debug, Serialize)]
pub struct CompletionRequest<'a> {
    /// The model to generate a completion from.
    pub model: Cow<'a, str>,
    /// The text to prompt the model with.
    pub prompt: Cow<'a, str>,
    /// Parameters to adjust the sampling behavior of the model.
    pub params: CompletionParams<'a>,
}

impl<'a> CompletionRequest<'a> {
    pub fn new(
        model: impl Into<Cow<'a, str>>,
        prompt: impl ToString,
        params: CompletionParams<'a>,
    ) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.to_string().into(),
            params,
        }
    }
}

/// Chunking parameters
#[derive(Clone, Debug, Serialize)]
pub struct ChunkParams<'a> {
    /// The name of the model the chunk is intended to be used for.
    /// This must be a known model.
    pub model: Cow<'a, str>,
    /// The maximum number of tokens that should be returned per chunk.
    pub max_tokens: u32,
}

impl<'a> ChunkParams<'a> {
    pub fn new(model: impl Into<Cow<'a, str>>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            max_tokens,
        }
    }
}

/// Pub for macro to work. Internal use only.
pub mod bindings {
    use exports::pharia::skill::skill_handler::Error;
    use serde::Serialize;

    use crate::{ChunkParams, Completion, CompletionParams, CompletionRequest, FinishReason};

    wit_bindgen::generate!({
        world: "skill",
        path: "./src/wit",
        pub_export_macro: true,
        default_bindings_module: "pharia_skill::bindings",
    });

    impl From<pharia::skill::csi::Completion> for Completion {
        fn from(value: pharia::skill::csi::Completion) -> Self {
            let pharia::skill::csi::Completion {
                text,
                finish_reason,
            } = value;
            Self {
                text,
                finish_reason: finish_reason.into(),
            }
        }
    }

    impl<'a> From<CompletionParams<'a>> for pharia::skill::csi::CompletionParams {
        fn from(value: CompletionParams<'a>) -> Self {
            let CompletionParams {
                max_tokens,
                temperature,
                top_k,
                top_p,
                stop,
            } = value;
            Self {
                max_tokens,
                temperature,
                top_k,
                top_p,
                stop: stop.iter().map(|s| s.clone().into_owned()).collect(),
            }
        }
    }

    impl<'a> From<&CompletionParams<'a>> for pharia::skill::csi::CompletionParams {
        fn from(value: &CompletionParams<'a>) -> Self {
            let CompletionParams {
                max_tokens,
                temperature,
                top_k,
                top_p,
                stop,
            } = value;
            Self {
                max_tokens: *max_tokens,
                temperature: *temperature,
                top_k: *top_k,
                top_p: *top_p,
                stop: stop.iter().map(|s| s.clone().into_owned()).collect(),
            }
        }
    }

    impl From<pharia::skill::csi::FinishReason> for FinishReason {
        fn from(value: pharia::skill::csi::FinishReason) -> Self {
            match value {
                pharia::skill::csi::FinishReason::Stop => Self::Stop,
                pharia::skill::csi::FinishReason::Length => Self::Length,
                pharia::skill::csi::FinishReason::ContentFilter => Self::ContentFilter,
            }
        }
    }

    impl<'a> From<&CompletionRequest<'a>> for pharia::skill::csi::CompletionRequest {
        fn from(value: &CompletionRequest<'a>) -> Self {
            let CompletionRequest {
                model,
                prompt,
                params,
            } = value;
            Self {
                model: model.clone().into_owned(),
                prompt: prompt.clone().into_owned(),
                params: params.into(),
            }
        }
    }

    impl<'a> From<&ChunkParams<'a>> for pharia::skill::csi::ChunkParams {
        fn from(value: &ChunkParams<'a>) -> Self {
            let ChunkParams { model, max_tokens } = value;
            Self {
                model: model.clone().into_owned(),
                max_tokens: *max_tokens,
            }
        }
    }

    /// CSI implementation for the WASI environment.
    pub struct WasiCsi;

    impl super::Csi for WasiCsi {
        fn complete(&self, request: &CompletionRequest<'_>) -> crate::Completion {
            pharia::skill::csi::complete(&request.model, &request.prompt, &(&request.params).into())
                .into()
        }

        fn complete_all(&self, requests: &[CompletionRequest<'_>]) -> Vec<Completion> {
            pharia::skill::csi::complete_all(&requests.iter().map(Into::into).collect::<Vec<_>>())
                .into_iter()
                .map(Into::into)
                .collect()
        }

        fn chunk(&self, text: &str, params: &ChunkParams<'_>) -> Vec<String> {
            pharia::skill::csi::chunk(text, &params.into())
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
        use serde::{Deserialize, Serialize};

        use crate::Error;

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
}
