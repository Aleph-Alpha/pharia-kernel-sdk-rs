use pharia_skill::{prompt::llama3_instruct::Prompt, skill, CompletionParams, Csi};

// This can also return an `anyhow::Result<String>` if you need handle errors.
#[skill]
fn hello_world(csi: &impl Csi, name: &str) -> String {
    let prompt = Prompt::new(
        "Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant.",
    )
    .with_user_message(format!(
        "Provide a nice greeting for the person named: {name}"
    ));

    let result = csi.complete(
        "llama-3.1-8b-instruct",
        prompt,
        CompletionParams {
            stop: vec!["<|start_header_id|>".to_owned()],
            ..Default::default()
        },
    );

    result.text
}

#[cfg(test)]
mod tests {
    use pharia_skill_test::MockCsi;

    use super::*;

    #[test]
    fn basic_hello() {
        let name = "Homer";
        let expected = format!("Hello, {name}");
        // You can also use `TestCsi` if you want to test against the real inference.
        let csi = MockCsi::new(&expected);

        let response = hello_world(&csi, name);

        assert_eq!(response, expected);
    }
}
