use pharia_skill::{skill, ChatRequest, Csi, Message};

// This can also return an `anyhow::Result<String>` if you need handle errors.
#[skill]
fn hello_world(csi: &impl Csi, name: &str) -> String {
    let system = Message::system(
        "Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant.",
    );

    let user = Message::user(format!(
        "Provide a nice greeting for the person named: {name}"
    ));
    let request = ChatRequest::new("llama-3.1-8b-instruct", system).and_message(user);

    let result = csi.chat(&request);
    result.message.content.to_string()
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
