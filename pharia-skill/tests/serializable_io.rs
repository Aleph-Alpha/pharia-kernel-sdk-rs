use pharia_skill::{Completion, CompletionParams, Csi, FinishReason};

#[pharia_skill::skill]
fn can_compile_with_result(_csi: &impl Csi, _input: &str) -> anyhow::Result<String> {
    Err(anyhow::anyhow!("Hello, world!"))
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
    let output = can_compile_with_result(&MockCsi, "Hello, world!");
    assert!(output.is_err());
}
