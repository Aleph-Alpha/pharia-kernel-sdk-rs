pub mod prompt;

pub use bindings::{
    exports::pharia::skill::skill_handler::Error,
    pharia::skill::csi::{Completion, CompletionParams, FinishReason},
};
/// Macro to define a Skill. It wraps a function that takes a single argument and returns a single value.
pub use pharia_skill_macros::skill;

/// Cognitive System Interface
pub trait Csi {
    /// Generate a completion for a given prompt using a specific model.
    ///
    /// # Errors
    /// Will error if the completion fails due to invalid input.
    fn complete(
        &self,
        model: impl Into<String>,
        prompt: impl ToString,
        params: CompletionParams,
    ) -> Completion;
}

// Can't derive Default because it is a bindgen
#[allow(clippy::derivable_impls)]
impl Default for CompletionParams {
    fn default() -> Self {
        Self {
            max_tokens: None,
            temperature: None,
            top_k: None,
            top_p: None,
            stop: vec![],
        }
    }
}

/// Pub for macro to work. Internal use only.
pub mod bindings {
    use exports::pharia::skill::skill_handler::Error;
    use serde::Serialize;

    use crate::{Completion, CompletionParams};

    wit_bindgen::generate!({
        world: "skill",
        path: "./src/wit",
        pub_export_macro: true,
        default_bindings_module: "pharia_skill::bindings",
        additional_derives: [Clone],
    });

    /// CSI implementation for the WASI environment.
    pub struct WasiCsi;

    impl super::Csi for WasiCsi {
        fn complete(
            &self,
            model: impl Into<String>,
            prompt: impl ToString,
            params: CompletionParams,
        ) -> Completion {
            pharia::skill::csi::complete(&model.into(), &prompt.to_string(), &params)
        }
    }

    /// Newtype so we can create `From` trait implementations for `anyhow::Result` and `String`.
    pub struct HandlerResult<T: Serialize>(Result<T, Error>);

    impl<T: Serialize> From<T> for HandlerResult<T> {
        fn from(value: T) -> Self {
            Self(Ok(value))
        }
    }

    impl<T: Serialize> From<anyhow::Result<T>> for HandlerResult<T> {
        fn from(result: anyhow::Result<T>) -> Self {
            match result {
                Ok(value) => Self(Ok(value)),
                Err(error) => Self(Err(Error::Internal(error.to_string()))),
            }
        }
    }

    impl<T: Serialize> From<HandlerResult<T>> for Result<Vec<u8>, Error> {
        fn from(value: HandlerResult<T>) -> Self {
            value.0.and_then(|v| json::to_vec(&v))
        }
    }

    /// JSON serialization and deserialization helpers for the main skill macro.s
    pub mod json {
        use serde::{Deserialize, Serialize};

        use crate::Error;

        /// Convert input from the parent `run` method into the expected Input for the skill handler.
        ///
        /// # Errors
        /// Will error if the input cannot be deserialized into the expected Input type.
        pub fn from_slice<'input, Input>(input: &'input [u8]) -> Result<Input, Error>
        where
            Input: Deserialize<'input>,
        {
            serde_json::from_slice(input)
                .map_err(|error| Error::InvalidInput(anyhow::Error::from(error).to_string()))
        }

        /// Convert output from the skill handler to the expected output for the parent `run` method.
        ///
        /// # Errors
        /// Will error if the output cannot be serialized into the expected output type.
        pub fn to_vec<Output>(output: &Output) -> Result<Vec<u8>, Error>
        where
            Output: Serialize,
        {
            serde_json::to_vec(output)
                .map_err(|error| Error::Internal(anyhow::Error::from(error).to_string()))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn serialize_result() {
            let result = HandlerResult::from("Hello, world!");
            let output = Result::<Vec<u8>, Error>::from(result).unwrap();
            assert_eq!(output, b"\"Hello, world!\"".to_vec());
        }

        #[test]
        fn dont_serialize_error() {
            let result = HandlerResult::<&str>(Err(Error::Internal("Hello, world!".to_owned())));
            let error = Result::<Vec<u8>, Error>::from(result).unwrap_err();
            assert_eq!(
                error.to_string(),
                "Error::Internal(\"Hello, world!\")".to_owned()
            );
        }
    }
}
