use serde::{Deserialize, Serialize};

/// The reason that the model stopped completing text
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    /// The model hit a natural stopping point or a provided stop sequence
    Stop,
    /// The maximum number of tokens specified in the request was reached
    Length,
    /// Content was omitted due to a flag from content filters
    ContentFilter,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Logprobs {
    #[default]
    No,
    Sampled,
    Top(u8),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Logprob {
    pub token: Vec<u8>,
    pub logprob: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Distribution {
    pub sampled: Logprob,
    pub top: Vec<Logprob>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TokenUsage {
    pub prompt: u32,
    pub completion: u32,
}

/// Completion request parameters
#[derive(Clone, Debug, Serialize)]
pub struct CompletionParams {
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
    pub stop: Vec<String>,
    /// Whether to include special tokens like `<|eot_id|>` in the completion
    pub return_special_tokens: bool,
    /// When specified, this number will decrease (or increase) the probability of repeating
    /// tokens that were mentioned prior in the completion. The penalty is cumulative. The more
    /// a token is mentioned in the completion, the more its probability will decrease.
    /// A negative value will increase the likelihood of repeating tokens.
    pub frequency_penalty: Option<f64>,
    /// The presence penalty reduces the probability of generating tokens that are already
    /// present in the generated text respectively prompt. Presence penalty is independent of the
    /// number of occurrences. Increase the value to reduce the probability of repeating text.
    pub presence_penalty: Option<f64>,
    /// Use this to control the logarithmic probabilities you want to have returned. This is useful
    /// to figure out how likely it had been that this specific token had been sampled.
    pub logprobs: Logprobs,
}

impl Default for CompletionParams {
    fn default() -> Self {
        Self {
            return_special_tokens: true,
            max_tokens: None,
            temperature: None,
            top_k: None,
            top_p: None,
            stop: Vec::new(),
            frequency_penalty: None,
            presence_penalty: None,
            logprobs: Logprobs::default(),
        }
    }
}

/// Parameters required to make a completion request.
#[derive(Clone, Debug, Serialize)]
pub struct CompletionRequest {
    /// The model to generate a completion from.
    pub model: String,
    /// The text to prompt the model with.
    pub prompt: String,
    /// Parameters to adjust the sampling behavior of the model.
    pub params: CompletionParams,
}

impl CompletionRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            params: CompletionParams::default(),
        }
    }

    #[must_use]
    pub fn with_params(mut self, params: CompletionParams) -> Self {
        self.params = params;
        self
    }
}

/// The result of a completion, including the text generated as well as
/// why the model finished completing.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Completion {
    /// The text generated by the model
    pub text: String,
    /// The reason the model finished generating
    pub finish_reason: FinishReason,
    /// Contains the logprobs for the sampled and top n tokens, given that
    /// `completion-request.params.logprobs` has been set to `sampled` or `top`.
    pub logprobs: Vec<Distribution>,
    /// Usage statistics for the completion request.
    pub usage: TokenUsage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ChatParams {
    /// The maximum tokens that should be inferred.
    ///
    /// Note: the backing implementation may return less tokens due to
    /// other stop reasons.
    pub max_tokens: Option<u32>,
    /// The randomness with which the next token is selected.
    pub temperature: Option<f64>,
    /// The probability total of next tokens the model will choose from.
    pub top_p: Option<f64>,
    /// When specified, this number will decrease (or increase) the probability of repeating
    /// tokens that were mentioned prior in the completion. The penalty is cumulative. The more
    /// a token is mentioned in the completion, the more its probability will decrease.
    /// A negative value will increase the likelihood of repeating tokens.
    pub frequency_penalty: Option<f64>,
    /// The presence penalty reduces the probability of generating tokens that are already
    /// present in the generated text respectively prompt. Presence penalty is independent of the
    /// number of occurrences. Increase the value to reduce the probability of repeating text.
    pub presence_penalty: Option<f64>,
    /// Use this to control the logarithmic probabilities you want to have returned. This is useful
    /// to figure out how likely it had been that this specific token had been sampled.
    pub logprobs: Logprobs,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub params: ChatParams,
}

impl ChatRequest {
    pub fn new(model: impl Into<String>, message: Message) -> Self {
        Self {
            model: model.into(),
            messages: vec![message],
            params: ChatParams::default(),
        }
    }

    #[must_use]
    pub fn and_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    #[must_use]
    pub fn with_params(mut self, params: ChatParams) -> Self {
        self.params = params;
        self
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ChatResponse {
    /// The message generated by the model
    pub message: Message,
    /// The reason the model finished generating
    pub finish_reason: FinishReason,
    /// Contains the logprobs for the sampled and top n tokens, given that
    /// `completion-request.params.logprobs` has been set to `sampled` or `top`.
    pub logprobs: Vec<Distribution>,
    /// Usage statistics for the completion request.
    pub usage: TokenUsage,
}
