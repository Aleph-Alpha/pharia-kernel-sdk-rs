use pharia_skill::{CompletionRequest, Csi};
use pharia_skill_test::StubCsi;
use schemars::{schema::RootSchema, schema_for};
use ureq::json;

#[pharia_skill::skill]
fn can_compile(csi: &impl Csi, input: Vec<&str>) -> Vec<String> {
    csi.complete_concurrently(
        input
            .into_iter()
            .map(|i| CompletionRequest::new("hello", i))
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|c| c.text.into_owned())
    .collect()
}

#[test]
fn mock_csi() {
    let output = can_compile(&StubCsi, vec!["Hello,", " world!"]);
    assert_eq!(output.join(""), "Hello, world!");
}

#[test]
fn metadata() {
    use pharia_skill::bindings::exports::pharia::skill::skill_handler::Guest;
    let metadata = __pharia_skill::Skill::metadata();

    let input_schema =
        pharia_skill::bindings::json::from_slice::<RootSchema>(&metadata.input_schema).unwrap();
    let output_schema =
        pharia_skill::bindings::json::from_slice::<RootSchema>(&metadata.output_schema).unwrap();

    assert_eq!(input_schema, schema_for!(Vec<&str>));
    assert_eq!(output_schema, schema_for!(Vec<String>));
    assert_eq!(metadata.description, None);
    assert!(jsonschema::meta::is_valid(&json!(input_schema)));
    assert!(jsonschema::meta::is_valid(&json!(output_schema)));
}
