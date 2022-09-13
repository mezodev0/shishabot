use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote, Block, Error, FnArg, Item, ItemFn, Lit, LitBool, LitStr, MetaNameValue, PatType,
    Result, ReturnType, Token,
};

use crate::util::IdentExt;

pub struct Function {
    block: Box<Block>,
    first_arg: PatType,
    second_arg: PatType,
    name: Ident,
}

impl Parse for Function {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed: Item = input.parse()?;

        let ItemFn { sig, block, .. } = match parsed {
            Item::Fn(m) => m,

            item => {
                return Err(syn::Error::new_spanned(
                    item,
                    "`msg_command` attribute can only be applied to functions",
                ))
            }
        };

        if sig.asyncness.is_none() {
            return Err(syn::Error::new_spanned(
                sig.asyncness,
                "`msg_command` function must be async",
            ));
        }

        if matches!(sig.output, ReturnType::Default) {
            return Err(syn::Error::new_spanned(
                sig,
                "`msg_command` function must return `Result<()>`",
            ));
        }

        let mut inputs = sig.inputs.into_iter();

        let first_arg = inputs
            .next()
            .and_then(|arg| match arg {
                FnArg::Typed(arg) => (arg.ty == parse_quote!(Arc<Context>)).then_some(arg),
                _ => None,
            })
            .ok_or_else(|| {
                Error::new(
                    Span::call_site(),
                    "first argument must be of type `Arc<Context>`",
                )
            })?;

        let second_arg = inputs
            .next()
            .and_then(|arg| match arg {
                FnArg::Typed(arg) => (arg.ty == parse_quote!(InteractionCommand)).then_some(arg),
                _ => None,
            })
            .ok_or_else(|| {
                Error::new(
                    Span::call_site(),
                    "second argument must be of type `InteractionCommand`",
                )
            })?;

        if let Some(arg) = inputs.next() {
            return Err(Error::new_spanned(&arg, "expected only two arguments"));
        }

        Ok(Self {
            block,
            first_arg,
            second_arg,
            name: sig.ident,
        })
    }
}

pub struct Attributes {
    name: LitStr,
    dm_permission: Option<LitBool>,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut dm_permission = None;

        while !input.is_empty() {
            let value = input.parse::<MetaNameValue>()?;

            if let Some(ident) = value.path.get_ident() {
                match ident.to_string().as_str() {
                    "name" => match value.lit {
                        Lit::Str(lit) => name = Some(lit),
                        _ => return Err(Error::new_spanned(value.lit, "expected string literal")),
                    },
                    "dm_permission" => match value.lit {
                        Lit::Bool(lit) => dm_permission = Some(lit),
                        _ => return Err(Error::new_spanned(value.lit, "expected boolean")),
                    },
                    _ => {
                        return Err(Error::new_spanned(
                            ident,
                            "expected `name`, or `dm_permission`",
                        ))
                    }
                }
            } else {
                return Err(Error::new_spanned(value.path, "expected single ident"));
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        let name =
            name.ok_or_else(|| Error::new(Span::call_site(), "`name = \"...\" is required`"))?;

        Ok(Self {
            name,
            dm_permission,
        })
    }
}

pub fn impl_(attrs: Attributes, fun: Function) -> Result<TokenStream> {
    let Function {
        block,
        first_arg,
        second_arg,
        name: fn_name,
    } = fun;

    let Attributes {
        name,
        dm_permission,
    } = attrs;

    let static_name = fn_name.to_uppercase();

    let dm_permission = OptionWrapper(dm_permission);

    let path = quote!(crate::core::commands::slash::MessageCommand);

    let tokens = quote! {
        pub static #static_name: #path = #path {
            create: create__,
            exec: exec__,
            name: #name,
        };

        fn create__() -> twilight_model::application::command::Command {
            twilight_model::application::command::Command {
                application_id: None,
                default_member_permissions: None,
                dm_permission: #dm_permission,
                description: String::new(),
                description_localizations: None,
                guild_id: None,
                id: None,
                kind: twilight_model::application::command::CommandType::Message,
                name: #name.to_owned(),
                name_localizations: None,
                options: Vec::new(),
                version: twilight_model::id::Id::new(1),
            }
        }

        fn exec__(#first_arg, #second_arg) -> crate::core::commands::slash::CommandResult {
            Box::pin(async move { #block })
        }
    };

    Ok(tokens)
}

struct OptionWrapper<T>(Option<T>);

impl<T: ToTokens> ToTokens for OptionWrapper<T> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.0 {
            Some(ref inner) => tokens.extend(quote!(Some(#inner))),
            None => tokens.extend(quote!(None)),
        }
    }
}
