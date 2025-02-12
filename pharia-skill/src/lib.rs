/// Pub for macro to work. Internal use only.
#[doc(hidden)]
pub mod bindings;
mod csi;

pub use csi::{
    chunking::{ChunkParams, ChunkRequest},
    document_index::{
        Document, DocumentPath, FilterCondition, IndexPath, MetadataFieldValue, MetadataFilter,
        MetadataFilterCondition, Modality, ModalityType, SearchFilter, SearchRequest, SearchResult,
        TextCursor,
    },
    inference::{
        ChatParams, ChatRequest, ChatResponse, Completion, CompletionParams, CompletionRequest,
        Distribution, FinishReason, Logprob, Logprobs, Message, TokenUsage,
    },
    language::{LanguageCode, SelectLanguageRequest},
    Csi,
};
/// Macro to define a Skill. It wraps a function that takes a single argument and returns a single value.
pub use pharia_skill_macros::skill;
