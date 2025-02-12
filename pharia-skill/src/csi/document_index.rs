use std::borrow::Cow;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Which documents you want to search in, and which type of index should be used
#[derive(Clone, Debug, Default, Serialize)]
pub struct IndexPath<'a> {
    /// The namespace the collection belongs to
    pub namespace: Cow<'a, str>,
    /// The collection you want to search in
    pub collection: Cow<'a, str>,
    /// The search index you want to use for the collection
    pub index: Cow<'a, str>,
}

impl<'a> IndexPath<'a> {
    pub fn new(
        namespace: impl Into<Cow<'a, str>>,
        collection: impl Into<Cow<'a, str>>,
        index: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            collection: collection.into(),
            index: index.into(),
        }
    }
}

/// Location of a document in the search engine
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DocumentPath<'a> {
    /// The namespace the collection belongs to
    pub namespace: Cow<'a, str>,
    /// The collection you want to search in
    pub collection: Cow<'a, str>,
    /// The name of the document
    pub name: Cow<'a, str>,
}

impl<'a> DocumentPath<'a> {
    pub fn new(
        namespace: impl Into<Cow<'a, str>>,
        collection: impl Into<Cow<'a, str>>,
        name: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            collection: collection.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SearchRequest<'a> {
    pub query: Cow<'a, str>,
    pub index_path: IndexPath<'a>,
    pub max_results: u32,
    pub min_score: Option<f64>,
    pub filters: Cow<'a, [SearchFilter<'a>]>,
}

impl<'a> SearchRequest<'a> {
    pub fn new(query: impl Into<Cow<'a, str>>, index_path: IndexPath<'a>) -> Self {
        Self {
            query: query.into(),
            index_path,
            max_results: 1,
            min_score: None,
            filters: Cow::Borrowed(&[]),
        }
    }

    #[must_use]
    pub fn with_filters(mut self, filters: impl Into<Cow<'a, [SearchFilter<'a>]>>) -> Self {
        self.filters = filters.into();
        self
    }

    #[must_use]
    pub fn with_max_results(mut self, max_results: u32) -> Self {
        self.max_results = max_results;
        self
    }

    #[must_use]
    pub fn with_min_score(mut self, min_score: Option<f64>) -> Self {
        self.min_score = min_score;
        self
    }
}

/// Result to a search query
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SearchResult<'a> {
    /// The path to the document that was found
    pub document_path: DocumentPath<'a>,
    /// The content of the document that was found
    pub content: Cow<'a, str>,
    /// How relevant the document is to the search query
    pub score: f64,
    pub start: TextCursor,
    pub end: TextCursor,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TextCursor {
    /// The index of the item in the document
    pub item: u32,
    /// The position of the cursor within the item
    pub position: u32,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SearchFilter<'a> {
    Without(Cow<'a, [FilterCondition<'a>]>),
    WithOneOf(Cow<'a, [FilterCondition<'a>]>),
    With(Cow<'a, [FilterCondition<'a>]>),
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FilterCondition<'a> {
    Metadata(MetadataFilter<'a>),
}

#[derive(Copy, Clone, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ModalityType {
    Text,
}

#[derive(Clone, Serialize, Debug)]
pub struct MetadataFilter<'a> {
    pub field: Cow<'a, str>,
    #[serde(flatten)]
    pub condition: MetadataFilterCondition<'a>,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MetadataFilterCondition<'a> {
    GreaterThan(f64),
    GreaterThanOrEqualTo(f64),
    LessThan(f64),
    LessThanOrEqualTo(f64),
    After(Timestamp),
    AtOrAfter(Timestamp),
    Before(Timestamp),
    AtOrBefore(Timestamp),
    EqualTo(MetadataFieldValue<'a>),
    IsNull(serde_bool::True),
}

#[derive(Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum MetadataFieldValue<'a> {
    String(Cow<'a, str>),
    Integer(i64),
    Boolean(bool),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "modality")]
pub enum Modality<'a> {
    Text { text: Cow<'a, str> },
    Image,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Document<'a, Metadata = Value> {
    pub path: DocumentPath<'a>,
    pub contents: Vec<Modality<'a>>,
    pub metadata: Option<Metadata>,
}
