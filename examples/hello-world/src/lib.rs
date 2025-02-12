use pharia_skill::{skill, ChatRequest, Csi, Message};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, JsonSchema, Serialize)]
struct Output {
    message: String,
}

/// This can also return an `anyhow::Result<Output>` if you need handle errors.
#[skill]
fn hello_world(csi: &impl Csi, name: &str) -> Output {
    let system = Message::system(
        "Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant.",
    );

    let user = Message::user(format!("Say hello to {name}",));
    let request = ChatRequest::new("llama-3.1-8b-instruct", system).and_message(user);

    let result = csi.chat(request);

    Output {
        message: result.message.content.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use pharia_skill_test::MockCsi;
    use schemars::{schema::RootSchema, schema_for};

    use super::*;

    #[test]
    fn basic_hello() {
        let name = "Homer";
        let expected = format!("Hello, {name}");
        // You can also use `DevtCsi` if you want to test against the real inference.
        let csi = MockCsi::new(&expected);

        let response = hello_world(&csi, name);

        assert_eq!(response.message, expected);
    }

    #[test]
    fn metadata() {
        use pharia_skill::bindings::exports::pharia::skill::skill_handler::Guest;
        let metadata = __pharia_skill::Skill::metadata();

        let input_schema =
            pharia_skill::bindings::json::from_slice::<RootSchema>(&metadata.input_schema).unwrap();
        let output_schema =
            pharia_skill::bindings::json::from_slice::<RootSchema>(&metadata.output_schema)
                .unwrap();

        assert_eq!(input_schema, schema_for!(String));
        assert_eq!(output_schema, schema_for!(Output));
        assert_eq!(
            metadata.description.unwrap(),
            "This can also return an `anyhow::Result<Output>` if you need handle errors."
        );
    }
}
