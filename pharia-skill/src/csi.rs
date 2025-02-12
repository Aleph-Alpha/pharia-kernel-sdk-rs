pub mod chunking;
pub mod document_index;
pub mod inference;
pub mod language;

use chunking::ChunkRequest;
use document_index::{Document, SearchResult};
use inference::{ChatRequest, ChatResponse, Completion, CompletionRequest};
use language::SelectLanguageRequest;
use serde::{Deserialize, Serialize};

use crate::{DocumentPath, LanguageCode, SearchRequest};

/// Cognitive System Interface
pub trait Csi {
    /// Chunk the given text into smaller pieces that fit within the
    /// maximum token amount for a given model.
    fn chunk(&self, request: ChunkRequest<'_>) -> Vec<String> {
        self.chunk_concurrently(vec![request]).remove(0)
    }

    /// Process multiple chunking requests at once
    fn chunk_concurrently(&self, requests: Vec<ChunkRequest<'_>>) -> Vec<Vec<String>>;

    /// Search for documents in a given index.
    fn search(&self, request: SearchRequest<'_>) -> Vec<SearchResult<'_>> {
        self.search_concurrently(vec![request]).remove(0)
    }

    /// Process multiple search requests at once
    fn search_concurrently(&self, requests: Vec<SearchRequest<'_>>) -> Vec<Vec<SearchResult<'_>>>;

    /// Retrieve a document from the Document Index by its path.
    ///
    /// # Errors
    /// Will return an error if document metadata cannot be deserialized.
    fn document<Metadata>(&self, path: DocumentPath<'_>) -> anyhow::Result<Document<'_, Metadata>>
    where
        Metadata: for<'a> Deserialize<'a> + Serialize,
    {
        Ok(self.documents(vec![path])?.remove(0))
    }

    /// Retrieve multiple documents from the Document Index by their paths.
    ///
    /// # Errors
    /// Will return an error if document metadata cannot be deserialized.
    fn documents<'m, Metadata>(
        &self,
        paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Document<'_, Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a> + Serialize;

    /// Retrieve a document's metadata from the Document Index by its path.
    ///
    /// # Errors
    /// Will return an error if metadata cannot be deserialized.
    fn document_metadata<Metadata>(
        &self,
        path: DocumentPath<'_>,
    ) -> anyhow::Result<Option<Metadata>>
    where
        Metadata: for<'a> Deserialize<'a> + Serialize,
    {
        Ok(self.documents_metadata(vec![path])?.remove(0))
    }

    /// Retrieve multiple documents' metadata from the Document Index by their paths.
    ///
    /// # Errors
    /// Will return an error if metadata cannot be deserialized.
    fn documents_metadata<Metadata>(
        &self,
        paths: Vec<DocumentPath<'_>>,
    ) -> anyhow::Result<Vec<Option<Metadata>>>
    where
        Metadata: for<'a> Deserialize<'a> + Serialize;

    /// Send messages with a particular role to a model and receive a response.
    /// Provides a higher level interface than completion for chat scenarios.
    fn chat(&self, request: ChatRequest<'_>) -> ChatResponse<'_> {
        self.chat_concurrently(vec![request]).remove(0)
    }

    /// Process multiple chat requests at once
    fn chat_concurrently(&self, requests: Vec<ChatRequest<'_>>) -> Vec<ChatResponse<'_>>;

    /// Generate a completion for a given prompt using a specific model.
    fn complete(&self, request: CompletionRequest<'_>) -> Completion<'_> {
        self.complete_concurrently(vec![request]).remove(0)
    }

    /// Process multiple completion requests at once
    fn complete_concurrently(&self, requests: Vec<CompletionRequest<'_>>) -> Vec<Completion<'_>>;

    /// Select the detected language for the provided input based on the list of possible languages.
    /// If no language matches, None is returned.
    ///
    /// text: Text input
    /// languages: All languages that should be considered during detection.
    fn select_language(&self, request: SelectLanguageRequest<'_>) -> Option<LanguageCode> {
        self.select_language_concurrently(vec![request]).remove(0)
    }

    /// Process multiple select language requests at once
    fn select_language_concurrently(
        &self,
        requests: Vec<SelectLanguageRequest<'_>>,
    ) -> Vec<Option<LanguageCode>>;
}
