# pharia-kernel-sdk-rs

Rust SDK for Pharia Kernel Skills

## Getting Started

### Create a new crate

```sh
cargo new --lib hello-world
```

### Update `Cargo.toml`

Then update your `Cargo.toml` with some WASM and Kernel specific configuration.

```toml
[package]
name = "hello-world"
edition = "2021"
version = "0.1.0"

[lib]
# This specifies how it should be compiled, necessary for WASM components.
crate-type = ["cdylib"]

[dependencies]
# For capturing errors in your skill code.
anyhow = "1"
# The Skill SDK for building Kernel Skills
pharia-skill = { version = "0.6.0" }
# Used for autogenerating an OpenAPI spec for your skill.
schemars = "0.8"
# For deriving custom input and output structs
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
# Helpers for testing.
pharia-skill-test = { version = "0.6.0" }

[profile.release]
codegen-units = 1
opt-level = "s"
debug = false
strip = true
lto = true
```

### Start writing your skill

With this setup, you should be able to start writing a basic Skill component. Your input and output can be anything that implements `serde`'s `Deserialize` for your input and `Serialize` for your output.

```rust
use pharia_skill::{
    prompt::llama3_instruct::Prompt, skill, CompletionParams, CompletionRequest, Csi,
};

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

    let result = csi.complete(&CompletionRequest::new(
        "llama-3.1-8b-instruct",
        prompt,
        CompletionParams {
            stop: &["<|start_header_id|>".into()],
            ..Default::default()
        },
    ));

    result.text
}
```

### Testing

With the `pharia-skill-test` crate, you can run your skill code locally and test that it is working as expected.

A basic test for the above example would be:

```rust
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
```

Which you can then run like a normal Rust test:

```sh
cargo test
```

### Build

To deploy your skill, you will need to compile for WASM, specifically a WASM WASI target.

First you will need to make sure you have the `wasm32-wasip2` target installed, which is available as of Rust v1.82.0:

```sh
rustup target add wasm32-wasip2
```

Now you should be able to compile your skill:

```sh
cargo build --target wasm32-wasip2 --release
```

You can now find your compiled component in your `target` directory, like `target/wasm32-wasip2/release/hello_world.wasm`
