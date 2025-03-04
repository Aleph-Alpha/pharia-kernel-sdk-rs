use crate::SelectLanguageRequest;

use super::pharia::skill::language;

impl From<SelectLanguageRequest> for language::SelectLanguageRequest {
    fn from(value: SelectLanguageRequest) -> Self {
        let SelectLanguageRequest { text, languages } = value;
        Self {
            text,
            languages: languages.iter().copied().map(|l| l.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_select_language_request() {
        let text = "Hello, world!";
        let languages = vec!["eng".to_owned(), "fra".to_owned()];
        let request = SelectLanguageRequest {
            text: text.into(),
            languages: languages.iter().map(|l| l.parse().unwrap()).collect(),
        };
        let converted = language::SelectLanguageRequest::from(request);

        assert_eq!(
            converted,
            language::SelectLanguageRequest {
                text: text.into(),
                languages
            }
        );
    }
}
