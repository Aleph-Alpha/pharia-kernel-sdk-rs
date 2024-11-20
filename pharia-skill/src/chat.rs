#[cfg(test)]
mod tests {
    use std::{borrow::Cow, marker::PhantomData};

    use crate::{ChatParams, ChatRequest, Message};

    /// Marker for system message
    struct System;
    /// Marker for user message
    struct User;
    /// Marker for assistant message
    struct Assistant;

    /// Generate a new chat for use with a Llama 3 based model.
    /// Valid prompt chains based on <https://github.com/meta-llama/llama-models/blob/main/models/llama3_2/text_prompt_format.md>
    #[derive(Debug)]
    struct Llama3Chat<'a, LastMessage> {
        messages: Vec<Message<'a>>,
        last_message: PhantomData<LastMessage>,
    }

    impl<'a> Llama3Chat<'a, ()> {
        fn from_system(content: impl Into<Cow<'a, str>>) -> Llama3Chat<'a, System> {
            Llama3Chat {
                messages: vec![Message::system(content)],
                last_message: PhantomData,
            }
        }

        fn from_user(content: impl Into<Cow<'a, str>>) -> Llama3Chat<'a, User> {
            Llama3Chat {
                messages: vec![Message::user(content)],
                last_message: PhantomData,
            }
        }
    }

    impl<'a> Llama3Chat<'a, System> {
        fn with_user(mut self, content: impl Into<Cow<'a, str>>) -> Llama3Chat<'a, User> {
            self.messages.push(Message::user(content));

            Llama3Chat {
                messages: self.messages,
                last_message: PhantomData,
            }
        }
    }

    impl<'a> Llama3Chat<'a, User> {
        fn with_assistant(mut self, content: impl Into<Cow<'a, str>>) -> Llama3Chat<'a, Assistant> {
            self.messages.push(Message::assistant(content));

            Llama3Chat {
                messages: self.messages,
                last_message: PhantomData,
            }
        }
    }

    impl<'a> Llama3Chat<'a, Assistant> {
        fn with_user(mut self, content: impl Into<Cow<'a, str>>) -> Llama3Chat<'a, User> {
            self.messages.push(Message::user(content));

            Llama3Chat {
                messages: self.messages,
                last_message: PhantomData,
            }
        }
    }

    impl<'a> From<Llama3Chat<'a, User>> for Vec<Message<'a>> {
        fn from(value: Llama3Chat<'a, User>) -> Self {
            value.messages
        }
    }

    impl<'a> From<Llama3Chat<'a, System>> for Vec<Message<'a>> {
        fn from(value: Llama3Chat<'a, System>) -> Self {
            value.messages
        }
    }

    impl<'a> ChatRequest<'a> {
        fn new_messages(
            model: impl Into<Cow<'a, str>>,
            messages: impl Into<Vec<Message<'a>>>,
        ) -> Self {
            Self {
                model: model.into(),
                messages: messages.into(),
                params: ChatParams::default(),
            }
        }
    }

    #[test]
    fn build_up_llama_request_with_system() {
        let chat = Llama3Chat::from_system("You are a helpful assistant")
            .with_user("Who are you?")
            .with_assistant("I'm an AI assistant.")
            .with_user("Nice to meet you");

        ChatRequest::new_messages("llama-3.1-8b-instruct", chat);
    }

    #[test]
    fn build_up_zeroshot_llama_request() {
        let chat = Llama3Chat::from_user("What is the color of the sky?");

        ChatRequest::new_messages("llama-3.1-8b-instruct", chat);
    }

    #[test]
    fn build_up_zeroshot_system_llama_request() {
        let chat = Llama3Chat::from_system(
            "Environment: ipython

Cutting Knowledge Date: December 2023
Today Date: 24 September 2024
<|eot_id|><|start_header_id|>user<|end_header_id|>

Write code to check if number is prime. Use it to verify if number 7 is prime",
        );

        ChatRequest::new_messages("llama-3.1-8b-instruct", chat);
    }
}
