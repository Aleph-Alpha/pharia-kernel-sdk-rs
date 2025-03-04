use crate::{
    ChatParams, ChatRequest, ChatResponse, Completion, CompletionParams, CompletionRequest,
    Distribution, FinishReason, Logprob, Logprobs, Message, TokenUsage,
};

use super::pharia::skill::inference;

impl From<inference::Logprob> for Logprob {
    fn from(value: inference::Logprob) -> Self {
        let inference::Logprob { token, logprob } = value;
        Self { token, logprob }
    }
}

impl From<inference::Distribution> for Distribution {
    fn from(value: inference::Distribution) -> Self {
        let inference::Distribution { sampled, top } = value;
        Self {
            sampled: sampled.into(),
            top: top.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<inference::TokenUsage> for TokenUsage {
    fn from(value: inference::TokenUsage) -> Self {
        let inference::TokenUsage { prompt, completion } = value;
        Self { prompt, completion }
    }
}

impl From<inference::FinishReason> for FinishReason {
    fn from(value: inference::FinishReason) -> Self {
        match value {
            inference::FinishReason::Stop => Self::Stop,
            inference::FinishReason::Length => Self::Length,
            inference::FinishReason::ContentFilter => Self::ContentFilter,
        }
    }
}

impl From<Logprobs> for inference::Logprobs {
    fn from(value: Logprobs) -> Self {
        match value {
            Logprobs::No => Self::No,
            Logprobs::Sampled => Self::Sampled,
            Logprobs::Top(n) => Self::Top(n),
        }
    }
}

impl From<CompletionParams> for inference::CompletionParams {
    fn from(value: CompletionParams) -> Self {
        let CompletionParams {
            max_tokens,
            temperature,
            top_k,
            top_p,
            stop,
            return_special_tokens,
            frequency_penalty,
            presence_penalty,
            logprobs,
        } = value;
        Self {
            max_tokens,
            temperature,
            top_k,
            top_p,
            stop,
            return_special_tokens,
            frequency_penalty,
            presence_penalty,
            logprobs: logprobs.into(),
        }
    }
}

impl From<CompletionRequest> for inference::CompletionRequest {
    fn from(value: CompletionRequest) -> Self {
        let CompletionRequest {
            model,
            prompt,
            params,
        } = value;
        Self {
            model,
            prompt,
            params: params.into(),
        }
    }
}

impl From<inference::Completion> for Completion {
    fn from(value: inference::Completion) -> Self {
        let inference::Completion {
            text,
            finish_reason,
            logprobs,
            usage,
        } = value;
        Self {
            text,
            finish_reason: finish_reason.into(),
            logprobs: logprobs.into_iter().map(Into::into).collect(),
            usage: usage.into(),
        }
    }
}

impl From<Message> for inference::Message {
    fn from(value: Message) -> Self {
        let Message { role, content } = value;
        Self { role, content }
    }
}

impl From<inference::Message> for Message {
    fn from(value: inference::Message) -> Self {
        let inference::Message { role, content } = value;
        Self { role, content }
    }
}

impl From<ChatParams> for inference::ChatParams {
    fn from(value: ChatParams) -> Self {
        let ChatParams {
            max_tokens,
            temperature,
            top_p,
            frequency_penalty,
            presence_penalty,
            logprobs,
        } = value;
        Self {
            max_tokens,
            temperature,
            top_p,
            frequency_penalty,
            presence_penalty,
            logprobs: logprobs.into(),
        }
    }
}

impl From<ChatRequest> for inference::ChatRequest {
    fn from(value: ChatRequest) -> Self {
        let ChatRequest {
            model,
            messages,
            params,
        } = value;
        Self {
            model,
            messages: messages.into_iter().map(Into::into).collect::<Vec<_>>(),
            params: params.into(),
        }
    }
}

impl From<inference::ChatResponse> for ChatResponse {
    fn from(value: inference::ChatResponse) -> Self {
        let inference::ChatResponse {
            message,
            finish_reason,
            logprobs,
            usage,
        } = value;
        Self {
            message: message.into(),
            finish_reason: finish_reason.into(),
            logprobs: logprobs.into_iter().map(Into::into).collect::<Vec<_>>(),
            usage: usage.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_request_conversion() {
        let model = "llama-2-7b-chat";
        let prompt = "Hello, world!";
        let max_tokens = Some(10);
        let temperature = Some(0.5);
        let top_p = Some(0.9);
        let frequency_penalty = Some(0.2);
        let presence_penalty = Some(0.1);
        let top_k = Some(5);
        let stop = &[".".into()];
        let return_special_tokens = true;
        let request = CompletionRequest {
            model: model.into(),
            prompt: prompt.into(),
            params: CompletionParams {
                max_tokens,
                temperature,
                top_p,
                frequency_penalty,
                presence_penalty,
                logprobs: Logprobs::No,
                top_k,
                stop: stop.into(),
                return_special_tokens,
            },
        };

        let converted = inference::CompletionRequest::from(request);

        assert_eq!(
            converted,
            inference::CompletionRequest {
                model: model.into(),
                prompt: prompt.into(),
                params: inference::CompletionParams {
                    max_tokens,
                    temperature,
                    top_p,
                    frequency_penalty,
                    presence_penalty,
                    logprobs: inference::Logprobs::No,
                    top_k,
                    stop: stop.into(),
                    return_special_tokens,
                },
            }
        );
    }

    #[test]
    fn test_completion_response_conversion() {
        let text = "Hello, world!";
        let token = vec![1, 2, 3];
        let logprob = -0.3;
        let prompt = 10;
        let completion = 5;
        let response = inference::Completion {
            text: text.into(),
            finish_reason: inference::FinishReason::Stop,
            logprobs: vec![inference::Distribution {
                sampled: inference::Logprob {
                    token: token.clone(),
                    logprob,
                },
                top: vec![],
            }],
            usage: inference::TokenUsage { prompt, completion },
        };

        let converted = Completion::from(response);

        assert_eq!(
            converted,
            Completion {
                text: text.into(),
                finish_reason: FinishReason::Stop,
                logprobs: (&[Distribution {
                    sampled: Logprob { token, logprob },
                    top: (&[]).into()
                }])
                    .into(),
                usage: TokenUsage { prompt, completion },
            }
        );
    }
}
