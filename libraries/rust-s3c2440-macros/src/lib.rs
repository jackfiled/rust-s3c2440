use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{ItemFn, Lit, Meta, Token, parse_macro_input};

struct EntryParameters {
    call_init: bool,
}

impl Parse for EntryParameters {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut call_init = true;

        if input.is_empty() {
            return Ok(EntryParameters { call_init });
        }

        let meta_list = input.parse_terminated(Meta::parse, Token![,])?;

        for meta in meta_list {
            match meta {
                Meta::NameValue(value) => {
                    let ident = value.path.get_ident().ok_or_else(|| {
                        syn::Error::new_spanned(&value.path, "Expected identifier.")
                    })?;

                    if ident != "call_init" {
                        return Err(syn::Error::new_spanned(
                            &value.path,
                            format!("Unknown identifier {}", ident),
                        ));
                    }

                    match &value.value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: Lit::Bool(b), ..
                        }) => call_init = b.value,
                        _ => {
                            return Err(syn::Error::new_spanned(
                                &value.path,
                                "Expected boolean value for call_init parameter.",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta.path(),
                        "Expected `key = value` pair.",
                    ));
                }
            }
        }

        Ok(EntryParameters { call_init })
    }
}

/// Attribute macro used to tag the entry point function for S3C2440 application.
///
/// # Arguments
/// Provided a parameter called `call_init` to control weather call the `rust_s3c2440_library::init_board`
/// hook function to initialize board.
/// This parameter is `true` default. If the application developer needs low-level control of the board, he
/// can trune off the behaviour.
#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let arguments = parse_macro_input!(attr as EntryParameters);
    let input = parse_macro_input!(item as ItemFn);
    let ItemFn {
        attrs: _attrs,
        vis: _vis,
        sig,
        block,
    } = input;

    let check_return_type = match sig.output {
        syn::ReturnType::Type(_, ref ty) => {
            if let syn::Type::Path(path) = ty.as_ref() {
                if path.path.segments.len() == 1 && path.path.segments[0].ident == "!" {
                    None
                } else {
                    Some(
                        syn::Error::new(sig.span(), "Entry function should not return.")
                            .to_compile_error(),
                    )
                }
            } else {
                None
            }
        }
        syn::ReturnType::Default => Some(
            syn::Error::new(sig.span(), "Entry function should not return.").to_compile_error(),
        ),
    };

    if let Some(err) = check_return_type {
        return err.into();
    }

    let expanded = if arguments.call_init {
        quote! {
            fn __user_main() -> ! #block

            #[unsafe(no_mangle)]
            pub fn rust_main() -> ! {
                rust_s3c2440_std::init_board();
                __user_main()
            }
        }
    } else {
        quote! {
            fn __user_main() -> ! #block

            #[unsafe(no_mangle)]
            pub fn rust_main() -> ! {
                __user_main()
            }
        }
    };

    TokenStream::from(expanded)
}
