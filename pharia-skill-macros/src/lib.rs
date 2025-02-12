use proc_macro::TokenStream;
use quote::quote;
use syn::{
    spanned::Spanned, AttrStyle, Expr, FnArg, GenericArgument, ItemFn, Lit, PathArguments,
    ReturnType, Type,
};

fn report_error(msg: &str, span: proc_macro2::Span) -> TokenStream {
    syn::Error::new(span, msg).to_compile_error().into()
}

const ARG_MSG: &str = "The skill function should take two arguments: first is `csi: &impl Csi`, second is `input` with a type that implements `serde::Deserialize` and `schemars::JsonSchema`.";
const RETURN_MSG: &str = "The skill function should return a value that implements `serde::Serialize` and `schemars::JsonSchema`.";

/// Macro to define a Skill. It wraps a function that takes a single argument and returns a single value.
///
/// The argument should implement `serde::Deserialize` to process an incoming JSON body, and the return value should implement `serde::Serialize` to return a JSON body.
/// Both also need to implement `schemars::JsonSchema` to generate a JSON schema for the input and output.
/// You can use the `#[derive(schemars::JsonSchema)]` attribute to automatically implement `JsonSchema` for your types.
///
/// Also, the doc comment can be used to provide a description of the skill.
#[proc_macro_attribute]
pub fn skill(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;
    let description = extract_doc_comment(&func);

    let Some(input_type) = func.sig.inputs.last() else {
        return report_error(ARG_MSG, func.span());
    };
    let input_type = match input_type {
        FnArg::Typed(pat_type) => &pat_type.ty,
        FnArg::Receiver(_) => return report_error(ARG_MSG, func.span()),
    };
    let output_type = extract_output_result(match &func.sig.output {
        ReturnType::Type(_, ty) => ty,
        ReturnType::Default => return report_error(RETURN_MSG, func.span()),
    });

    quote!(
        #func

        static __SKILL_METADATA: std::sync::LazyLock<::pharia_skill::bindings::exports::pharia::skill::skill_handler::SkillMetadata> = std::sync::LazyLock::new(|| {
            use ::pharia_skill::bindings::{exports::pharia::skill::skill_handler::SkillMetadata, json};
            let input_schema = json::schema_for!(#input_type);
            let output_schema = json::schema_for!(#output_type);
            SkillMetadata {
                description: (!#description.is_empty()).then_some(#description.to_string()),
                input_schema: json::to_vec(&input_schema).expect("Failed to serialize input schema"),
                output_schema: json::to_vec(&output_schema).expect("Failed to serialize output schema"),
            }
        });

        mod __pharia_skill {
            use ::pharia_skill::bindings::{
                export,
                exports::pharia::skill::skill_handler::{Error, Guest, SkillMetadata},
                json, HandlerResult, WitCsi,
            };

            pub struct Skill;

            impl Guest for Skill {
                fn run(input: Vec<u8>) -> Result<Vec<u8>, Error> {
                    let input = json::from_slice(&input)?;
                    let output = super::#func_name(&WitCsi, input);
                    HandlerResult::from(output).into()
                }

                fn metadata() -> SkillMetadata {
                    super::__SKILL_METADATA.clone()
                }
            }

            export!(Skill);
        }

    )
    .into()
}

// Pull out the type from a Result type if the user used one.
fn extract_output_result(output_type: &Type) -> &Type {
    match output_type {
        Type::Path(path) => path
            .path
            .segments
            .last()
            // Special case for Result types
            .and_then(|segment| (segment.ident == "Result").then_some(&segment.arguments))
            // We only want angle bracket generics
            .and_then(|args| match args {
                PathArguments::AngleBracketed(args) => Some(args),
                PathArguments::Parenthesized(_) | PathArguments::None => None,
            })
            // Get the first one that is a type
            .and_then(|args| {
                args.args.iter().find_map(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
            }),
        _ => None,
    }
    .unwrap_or(output_type)
}

fn extract_doc_comment(func: &ItemFn) -> String {
    func.attrs
        .iter()
        // Only grab attributes that are outer doc comments
        .filter(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident("doc"))
        // All doc comments should be NameValues
        .filter_map(|attr| attr.meta.require_name_value().ok())
        // Pull out the literal value of the line
        .filter_map(|meta_name_value| match &meta_name_value.value {
            Expr::Lit(lit) => Some(&lit.lit),
            _ => None,
        })
        // Get the string, and trim the extra whitespace
        .filter_map(|lit| {
            if let Lit::Str(s) = lit {
                Some(s.value().trim().to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
