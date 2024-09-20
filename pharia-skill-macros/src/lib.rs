use proc_macro::TokenStream;
use quote::quote;

/// Macro to define a Skill. It wraps a function that takes a single argument and returns a single value.
///
/// The argument should implement `Deserialize` to process an incoming JSON body, and the return value should implement `Serialize` to return a JSON body.
#[proc_macro_attribute]
pub fn skill(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;

    quote!(
        #func
        mod __pharia_skill {
            pub struct Skill;

            impl ::pharia_skill::bindings::exports::pharia::skill::skill_handler::Guest for Skill {
                fn run(input: Vec<u8>) -> Result<Vec<u8>, ::pharia_skill::Error> {
                    let input = ::pharia_skill::macro_helpers::json::from_slice(&input)?;
                    let output = super::#func_name(&::pharia_skill::bindings::WasiCsi, input);
                    ::pharia_skill::macro_helpers::HandlerResult::from(output).into()
                }
            }

            ::pharia_skill::bindings::export!(Skill);
        }

    )
    .into()
}
