use pharia_skill::{Completion, CompletionRequest, Csi, FinishReason};

#[pharia_skill::skill]
fn can_compile_with_result(_csi: &impl Csi, _input: &str) -> anyhow::Result<String> {
    Err(anyhow::anyhow!("Hello, world!"))
}

struct MockCsi;

impl pharia_skill::Csi for MockCsi {
    fn complete(&self, request: &CompletionRequest<'_>) -> Completion {
        Completion {
            text: request.prompt.clone().into_owned(),
            finish_reason: FinishReason::Stop,
        }
    }

    fn chunk(&self, text: &str, _params: &pharia_skill::ChunkParams<'_>) -> Vec<String> {
        vec![text.to_owned()]
    }
}

#[test]
fn mock_csi() {
    let output = can_compile_with_result(&MockCsi, "Hello, world!");
    assert!(output.is_err());
}
