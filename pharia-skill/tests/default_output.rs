use pharia_skill::{Completion, CompletionParams, Csi, FinishReason};

#[pharia_skill::skill]
fn can_compile(csi: &impl Csi, input: Vec<&str>) -> Vec<String> {
    input
        .into_iter()
        .map(|input| {
            csi.complete("hello", input, CompletionParams::default())
                .text
        })
        .collect()
}

struct MockCsi;

impl pharia_skill::Csi for MockCsi {
    fn complete(
        &self,
        _model: impl Into<String>,
        prompt: impl ToString,
        _params: CompletionParams,
    ) -> Completion {
        Completion {
            text: prompt.to_string(),
            finish_reason: FinishReason::Stop,
        }
    }
}

#[test]
fn mock_csi() {
    let output = can_compile(&MockCsi, vec!["Hello,", " world!"]);
    assert_eq!(output.join(""), "Hello, world!");
}
