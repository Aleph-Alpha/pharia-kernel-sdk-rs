use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Which documents you want to search in, and which type of index should be used
#[derive(Clone, Debug, Default, Serialize)]
pub struct IndexPath {
    /// The namespace the collection belongs to
    pub namespace: String,
    /// The collection you want to search in
    pub collection: String,
    /// The search index you want to use for the collection
    pub index: String,
}

impl IndexPath {
    pub fn new(
        namespace: impl Into<String>,
        collection: impl Into<String>,
        index: impl Into<String>,
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
pub struct DocumentPath {
    /// The namespace the collection belongs to
    pub namespace: String,
    /// The collection you want to search in
    pub collection: String,
    /// The name of the document
    pub name: String,
}

impl DocumentPath {
    pub fn new(
        namespace: impl Into<String>,
        collection: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            collection: collection.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub index_path: IndexPath,
    pub max_results: u32,
    pub min_score: Option<f64>,
    pub filters: Vec<SearchFilter>,
}

impl SearchRequest {
    pub fn new(query: impl Into<String>, index_path: IndexPath) -> Self {
        Self {
            query: query.into(),
            index_path,
            max_results: 1,
            min_score: None,
            filters: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_filters(mut self, filters: impl Into<Vec<SearchFilter>>) -> Self {
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
pub struct SearchResult {
    /// The path to the document that was found
    pub document_path: DocumentPath,
    /// The content of the document that was found
    pub content: String,
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
pub enum SearchFilter {
    Without(Vec<FilterCondition>),
    WithOneOf(Vec<FilterCondition>),
    With(Vec<FilterCondition>),
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FilterCondition {
    Metadata(MetadataFilter),
}

#[derive(Copy, Clone, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ModalityType {
    Text,
}

#[derive(Clone, Serialize, Debug)]
pub struct MetadataFilter {
    pub field: String,
    #[serde(flatten)]
    pub condition: MetadataFilterCondition,
}

#[derive(Clone, Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MetadataFilterCondition {
    GreaterThan(f64),
    GreaterThanOrEqualTo(f64),
    LessThan(f64),
    LessThanOrEqualTo(f64),
    After(Timestamp),
    AtOrAfter(Timestamp),
    Before(Timestamp),
    AtOrBefore(Timestamp),
    EqualTo(MetadataFieldValue),
    IsNull(serde_bool::True),
}

#[derive(Clone, Serialize, Debug)]
#[serde(untagged)]
pub enum MetadataFieldValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "modality")]
pub enum Modality {
    Text { text: String },
    Image,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Document<Metadata = Value> {
    pub path: DocumentPath,
    pub contents: Vec<Modality>,
    pub metadata: Option<Metadata>,
}
