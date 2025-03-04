use crate::{ChunkParams, ChunkRequest};

use super::pharia::skill::chunking;

impl From<ChunkParams> for chunking::ChunkParams {
    fn from(value: ChunkParams) -> Self {
        let ChunkParams {
            model,
            max_tokens,
            overlap,
        } = value;
        Self {
            model,
            max_tokens,
            overlap,
        }
    }
}

impl From<ChunkRequest> for chunking::ChunkRequest {
    fn from(value: ChunkRequest) -> Self {
        let ChunkRequest { text, params } = value;
        Self {
            text,
            params: params.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking() {
        let text = "This is a test string.";
        let model = "llama-3.1-8b-instruct";
        let max_tokens = 10;
        let overlap = 2;
        let params = ChunkParams {
            model: model.into(),
            max_tokens,
            overlap,
        };
        let request = ChunkRequest {
            text: text.into(),
            params,
        };
        let converted = chunking::ChunkRequest::from(request);

        assert_eq!(
            converted,
            chunking::ChunkRequest {
                text: text.into(),
                params: chunking::ChunkParams {
                    model: model.into(),
                    max_tokens,
                    overlap,
                },
            }
        );
    }
}
