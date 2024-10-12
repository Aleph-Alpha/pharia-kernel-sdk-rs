use pharia_skill::{Completion, CompletionRequest, Csi, FinishReason};

#[pharia_skill::skill]
fn can_compile_with_result(_csi: &impl Csi, _input: &str) -> anyhow::Result<String> {
    Err(anyhow::anyhow!("Hello, world!"))
}

struct MockCsi;

impl pharia_skill::Csi for MockCsi {
    fn complete(&self, request: &CompletionRequest<'_>) -> Completion {
        Completion {
            text: request.prompt.clone(),
            finish_reason: FinishReason::Stop,
        }
    }
}

#[test]
fn mock_csi() {
    let output = can_compile_with_result(&MockCsi, "Hello, world!");
    assert!(output.is_err());
}
