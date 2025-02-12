use std::borrow::Cow;

use serde::Serialize;

/// Chunking parameters
#[derive(Clone, Debug, Serialize)]
pub struct ChunkParams<'a> {
    /// The name of the model the chunk is intended to be used for.
    /// This must be a known model.
    pub model: Cow<'a, str>,
    /// The maximum number of tokens that should be returned per chunk.
    pub max_tokens: u32,
    /// The amount of allowed overlap between chunks.
    /// overlap must be less than max-tokens.
    pub overlap: u32,
}

impl<'a> ChunkParams<'a> {
    pub fn new(model: impl Into<Cow<'a, str>>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            max_tokens,
            overlap: 0,
        }
    }

    #[must_use]
    pub fn with_overlap(mut self, overlap: u32) -> Self {
        self.overlap = overlap;
        self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ChunkRequest<'a> {
    pub text: Cow<'a, str>,
    pub params: ChunkParams<'a>,
}

impl<'a> ChunkRequest<'a> {
    pub fn new(text: impl Into<Cow<'a, str>>, params: ChunkParams<'a>) -> Self {
        Self {
            text: text.into(),
            params,
        }
    }
}
