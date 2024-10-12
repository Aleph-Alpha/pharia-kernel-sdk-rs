use pharia_skill::{Completion, CompletionParams, CompletionRequest, Csi, FinishReason};

#[pharia_skill::skill]
fn can_compile(csi: &impl Csi, input: Vec<&str>) -> Vec<String> {
    csi.complete_all(
        &input
            .into_iter()
            .map(|i| CompletionRequest::new("hello", i, CompletionParams::default()))
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|c| c.text)
    .collect()
}

struct MockCsi;

impl pharia_skill::Csi for MockCsi {
    fn complete(&self, request: &CompletionRequest<'_>) -> Completion {
        Completion {
            text: request.prompt.clone().into_owned(),
            finish_reason: FinishReason::Stop,
        }
    }
}

#[test]
fn mock_csi() {
    let output = can_compile(&MockCsi, vec!["Hello,", " world!"]);
    assert_eq!(output.join(""), "Hello, world!");
}
