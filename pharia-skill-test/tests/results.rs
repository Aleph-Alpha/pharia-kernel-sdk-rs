use pharia_skill::Csi;
use pharia_skill_test::StubCsi;

#[pharia_skill::skill]
fn can_compile_with_result(_csi: &impl Csi, _input: &str) -> anyhow::Result<String> {
    Err(anyhow::anyhow!("Hello, world!"))
}

#[test]
fn mock_csi() {
    let output = can_compile_with_result(&StubCsi, "Hello, world!");
    assert!(output.is_err());
}
