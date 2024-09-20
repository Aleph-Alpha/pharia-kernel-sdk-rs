pub mod llama3_instruct {
    use std::{borrow::Cow, fmt};

    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Role {
        System,
        User,
        IPython,
        Assistant,
    }

    impl fmt::Display for Role {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let role = match self {
                Role::System => "system",
                Role::User => "user",
                Role::IPython => "ipython",
                Role::Assistant => "assistant",
            };
            write!(f, "{role}")
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Message<'a> {
        role: Role,
        message: Cow<'a, str>,
    }

    impl<'a> Message<'a> {
        #[must_use]
        pub fn new(role: Role, message: impl Into<Cow<'a, str>>) -> Self {
            Self {
                role,
                message: message.into(),
            }
        }

        #[must_use]
        fn system(message: impl Into<Cow<'a, str>>) -> Self {
            Self::new(Role::System, message)
        }

        #[must_use]
        pub fn user(message: impl Into<Cow<'a, str>>) -> Self {
            Self::new(Role::User, message)
        }

        #[must_use]
        pub fn ipython(message: impl Into<Cow<'a, str>>) -> Self {
            Self::new(Role::IPython, message)
        }

        #[must_use]
        pub fn assistant(message: impl Into<Cow<'a, str>>) -> Self {
            Self::new(Role::Assistant, message)
        }

        #[must_use]
        pub fn role(&self) -> Role {
            self.role
        }

        #[must_use]
        pub fn message(&self) -> &str {
            &self.message
        }
    }

    impl<'a> fmt::Display for Message<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "<|start_header_id|>{}<|end_header_id|>", self.role)?;
            write!(f, "\n\n")?;
            write!(f, "{}", self.message)?;
            write!(f, "<|eot_id|>")
        }
    }

    pub struct Prompt<'a> {
        messages: Vec<Message<'a>>,
    }

    impl<'a> Prompt<'a> {
        #[must_use]
        pub fn new(system: impl Into<Cow<'a, str>>) -> Self {
            let system = Message::system(system);
            let messages = vec![system];
            Self { messages }
        }

        #[must_use]
        pub fn with_message(mut self, message: Message<'a>) -> Self {
            self.push(message);
            self
        }

        #[must_use]
        pub fn with_user_message(self, message: impl Into<Cow<'a, str>>) -> Self {
            let message = Message::user(message);
            self.with_message(message)
        }

        #[must_use]
        pub fn with_ipython_message(self, message: impl Into<Cow<'a, str>>) -> Self {
            let message = Message::ipython(message);
            self.with_message(message)
        }

        #[must_use]
        pub fn with_assistant_message(self, message: impl Into<Cow<'a, str>>) -> Self {
            let message = Message::assistant(message);
            self.with_message(message)
        }

        #[must_use]
        pub fn with_messages(mut self, messages: impl IntoIterator<Item = Message<'a>>) -> Self {
            self.extend(messages);
            self
        }

        pub fn push(&mut self, message: Message<'a>) {
            self.messages.push(message);
        }

        pub fn extend(&mut self, messages: impl IntoIterator<Item = Message<'a>>) {
            self.messages.extend(messages);
        }
    }

    impl<'a> fmt::Display for Prompt<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "<|begin_of_text|>")?;
            for message in &self.messages {
                write!(f, "{message}")?;
            }
            write!(f, "<|start_header_id|>assistant<|end_header_id|>")
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn llama3_message() {
            let message = Message::system(
                "Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant",
            );
            let expected = "<|start_header_id|>system<|end_header_id|>

Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant<|eot_id|>";
            assert_eq!(message.to_string(), expected);
        }

        #[test]
        fn llama3_instruct_prompt() {
            let llama3prompt = Prompt::new(
                "Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant",
            )
            .with_message(Message::user("What is the capital for France?"));
            let expected = "<|begin_of_text|><|start_header_id|>system<|end_header_id|>

Cutting Knowledge Date: December 2023
Today Date: 23 Jul 2024

You are a helpful assistant<|eot_id|><|start_header_id|>user<|end_header_id|>

What is the capital for France?<|eot_id|><|start_header_id|>assistant<|end_header_id|>";
            assert_eq!(llama3prompt.to_string(), expected);
        }
    }
}
