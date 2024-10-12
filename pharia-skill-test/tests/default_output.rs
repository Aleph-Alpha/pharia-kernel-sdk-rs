use pharia_skill::{CompletionParams, CompletionRequest, Csi};
use pharia_skill_test::StubCsi;

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

#[test]
fn mock_csi() {
    let output = can_compile(&StubCsi, vec!["Hello,", " world!"]);
    assert_eq!(output.join(""), "Hello, world!");
}
