use serde::Deserialize;

use crate::{
    Document, DocumentPath, FilterCondition, IndexPath, MetadataFieldValue, MetadataFilter,
    MetadataFilterCondition, Modality, SearchFilter, SearchRequest, SearchResult, TextCursor,
};

use super::pharia::skill::document_index;

impl From<IndexPath> for document_index::IndexPath {
    fn from(value: IndexPath) -> Self {
        let IndexPath {
            namespace,
            collection,
            index,
        } = value;
        Self {
            namespace,
            collection,
            index,
        }
    }
}

impl From<document_index::DocumentPath> for DocumentPath {
    fn from(value: document_index::DocumentPath) -> Self {
        let document_index::DocumentPath {
            namespace,
            collection,
            name,
        } = value;
        Self {
            namespace,
            collection,
            name,
        }
    }
}

impl From<DocumentPath> for document_index::DocumentPath {
    fn from(value: DocumentPath) -> Self {
        let DocumentPath {
            namespace,
            collection,
            name,
        } = value;
        Self {
            namespace,
            collection,
            name,
        }
    }
}

impl From<SearchRequest> for document_index::SearchRequest {
    fn from(value: SearchRequest) -> Self {
        let SearchRequest {
            query,
            index_path,
            max_results,
            min_score,
            filters,
        } = value;
        Self {
            index_path: index_path.into(),
            query,
            max_results,
            min_score,
            filters: filters.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<document_index::SearchResult> for SearchResult {
    fn from(value: document_index::SearchResult) -> Self {
        let document_index::SearchResult {
            document_path,
            content,
            score,
            start,
            end,
        } = value;
        Self {
            document_path: document_path.into(),
            content,
            score,
            start: start.into(),
            end: end.into(),
        }
    }
}

impl From<document_index::TextCursor> for TextCursor {
    fn from(value: document_index::TextCursor) -> Self {
        let document_index::TextCursor { item, position } = value;
        Self { item, position }
    }
}

impl From<SearchFilter> for document_index::SearchFilter {
    fn from(value: SearchFilter) -> Self {
        match value {
            SearchFilter::Without(conditions) => {
                Self::Without(conditions.into_iter().map(Into::into).collect())
            }
            SearchFilter::WithOneOf(conditions) => {
                Self::WithOneOf(conditions.into_iter().map(Into::into).collect())
            }
            SearchFilter::With(conditions) => {
                Self::WithAll(conditions.into_iter().map(Into::into).collect())
            }
        }
    }
}

impl From<FilterCondition> for document_index::MetadataFilter {
    fn from(value: FilterCondition) -> Self {
        match value {
            FilterCondition::Metadata(metadata_filter) => metadata_filter.into(),
        }
    }
}

impl From<MetadataFilter> for document_index::MetadataFilter {
    fn from(value: MetadataFilter) -> Self {
        let MetadataFilter { field, condition } = value;
        Self {
            field,
            condition: condition.into(),
        }
    }
}

impl From<MetadataFilterCondition> for document_index::MetadataFilterCondition {
    fn from(value: MetadataFilterCondition) -> Self {
        match value {
            MetadataFilterCondition::GreaterThan(n) => Self::GreaterThan(n),
            MetadataFilterCondition::GreaterThanOrEqualTo(n) => Self::GreaterThanOrEqualTo(n),
            MetadataFilterCondition::LessThan(n) => Self::LessThan(n),
            MetadataFilterCondition::LessThanOrEqualTo(n) => Self::LessThanOrEqualTo(n),
            MetadataFilterCondition::After(s) => Self::After(s.to_string()),
            MetadataFilterCondition::AtOrAfter(s) => Self::AtOrAfter(s.to_string()),
            MetadataFilterCondition::Before(s) => Self::Before(s.to_string()),
            MetadataFilterCondition::AtOrBefore(s) => Self::AtOrBefore(s.to_string()),
            MetadataFilterCondition::EqualTo(metadata_field_value) => {
                Self::EqualTo(metadata_field_value.into())
            }
            MetadataFilterCondition::IsNull(_) => Self::IsNull,
        }
    }
}

impl From<MetadataFieldValue> for document_index::MetadataFieldValue {
    fn from(value: MetadataFieldValue) -> Self {
        match value {
            MetadataFieldValue::String(s) => Self::StringType(s),
            MetadataFieldValue::Integer(n) => Self::IntegerType(n),
            MetadataFieldValue::Boolean(b) => Self::BooleanType(b),
        }
    }
}

impl From<document_index::Modality> for Modality {
    fn from(value: document_index::Modality) -> Self {
        match value {
            document_index::Modality::Text(text) => Self::Text { text },
            document_index::Modality::Image => Self::Image,
        }
    }
}

impl<Metadata> TryFrom<document_index::Document> for Document<Metadata>
where
    Metadata: for<'a> Deserialize<'a>,
{
    type Error = anyhow::Error;

    fn try_from(value: document_index::Document) -> Result<Self, Self::Error> {
        let document_index::Document {
            path,
            contents,
            metadata,
        } = value;
        Ok(Self {
            path: path.into(),
            contents: contents.into_iter().map(Into::into).collect(),
            metadata: metadata.map(|m| serde_json::from_slice(&m)).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_conversion() {
        let query = "example query";
        let namespace = "example_namespace";
        let collection = "example_collection";
        let index = "example_index";
        let min_score = Some(0.5);
        let max_results = 10;
        let field = "example_field";
        let timestamp = "2005-08-07T23:19:49.123Z";
        let filter = &[
            FilterCondition::Metadata(MetadataFilter {
                field: field.into(),
                condition: MetadataFilterCondition::LessThan(10.),
            }),
            FilterCondition::Metadata(MetadataFilter {
                field: field.into(),
                condition: MetadataFilterCondition::Before(timestamp.parse().unwrap()),
            }),
        ];
        let filters = &[SearchFilter::With(filter.into())];
        let index_path = IndexPath {
            namespace: namespace.into(),
            collection: collection.into(),
            index: index.into(),
        };
        let request = SearchRequest {
            query: query.into(),
            index_path,
            min_score,
            max_results,
            filters: filters.into(),
        };
        let converted = document_index::SearchRequest::from(request);

        assert_eq!(
            converted,
            document_index::SearchRequest {
                query: query.into(),
                index_path: document_index::IndexPath {
                    namespace: namespace.into(),
                    collection: collection.into(),
                    index: index.into(),
                },
                min_score,
                max_results,
                filters: vec![document_index::SearchFilter::WithAll(vec![
                    document_index::MetadataFilter {
                        field: field.into(),
                        condition: document_index::MetadataFilterCondition::LessThan(10.),
                    },
                    document_index::MetadataFilter {
                        field: field.into(),
                        condition: document_index::MetadataFilterCondition::Before(
                            timestamp.to_owned()
                        ),
                    }
                ])],
            }
        );
    }

    #[test]
    fn test_response_conversion() {
        let namespace = "test_namespace";
        let collection = "test_collection";
        let name = "test_name";
        let content = "test_content";
        let score = 10.0;
        let item = 1;
        let position = 1;
        let response = document_index::SearchResult {
            document_path: document_index::DocumentPath {
                namespace: namespace.into(),
                collection: collection.into(),
                name: name.into(),
            },
            content: content.into(),
            score,
            start: document_index::TextCursor { item, position },
            end: document_index::TextCursor { item, position },
        };
        let converted = SearchResult::from(response);

        assert_eq!(
            converted,
            SearchResult {
                document_path: DocumentPath {
                    namespace: namespace.into(),
                    collection: collection.into(),
                    name: name.into()
                },
                content: content.into(),
                score,
                start: TextCursor { item, position },
                end: TextCursor { item, position }
            }
        );
    }
}
