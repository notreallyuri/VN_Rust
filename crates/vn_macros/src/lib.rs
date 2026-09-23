use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Error, FnArg, ItemFn, LitStr, Pat, PatType, ReturnType, Type, parse_macro_input};

#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let alias = match attr.is_empty() {
        true => None,
        false => Some(parse_macro_input!(attr as LitStr)),
    };
    let function = parse_macro_input!(item as ItemFn);

    match expand(alias, function) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand(alias: Option<LitStr>, function: ItemFn) -> Result<proc_macro2::TokenStream, Error> {
    let name = match &alias {
        Some(alias) => alias.value(),
        None => function.sig.ident.to_string(),
    };
    let span = alias
        .as_ref()
        .map_or_else(|| function.sig.ident.span(), |alias| alias.span());
    check_name(&name, span)?;

    if let Some(asyncness) = function.sig.asyncness {
        return Err(Error::new(
            asyncness.span(),
            "a command runs inside a frame, so it can't be `async`",
        ));
    }
    if !function.sig.generics.params.is_empty() {
        return Err(Error::new(
            function.sig.generics.span(),
            "a command can't be generic: the engine calls it through one signature",
        ));
    }

    let mut inputs = function.sig.inputs.iter();
    let context = inputs.next().ok_or_else(|| {
        Error::new(
            function.sig.span(),
            "a command takes the context first: `fn name(ctx: &mut GameContext, ...)`",
        )
    })?;
    if let FnArg::Receiver(receiver) = context {
        return Err(Error::new(
            receiver.span(),
            "a command is a free function, not a method",
        ));
    }

    let arguments: Vec<&PatType> = inputs
        .map(|input| match input {
            FnArg::Typed(typed) => Ok(typed),
            FnArg::Receiver(receiver) => Err(Error::new(
                receiver.span(),
                "a command is a free function, not a method",
            )),
        })
        .collect::<Result<_, Error>>()?;

    for argument in &arguments {
        if matches!(*argument.ty, Type::Reference(_)) {
            return Err(Error::new(
                argument.ty.span(),
                "a command argument is parsed out of the story line, so it is owned: take `String`, not `&str`",
            ));
        }
    }

    let types: Vec<&Type> = arguments.iter().map(|argument| &*argument.ty).collect();
    let args = match types[..] {
        [] => quote!(()),
        [rest] if is_rest(rest) => quote!(#rest),
        _ => quote!((#(#types,)*)),
    };
    let bindings: Vec<syn::Ident> = (0..types.len())
        .map(|index| format_ident!("argument_{}", index))
        .collect();
    let taken = match types[..] {
        [] => quote!(let () = arguments;),
        [rest] if is_rest(rest) => {
            let binding = &bindings[0];
            quote!(let #binding = arguments;)
        }
        _ => quote!(let (#(#bindings,)*) = arguments;),
    };

    let marker = &function.sig.ident;
    let visibility = &function.vis;
    let attributes = &function.attrs;
    let documented = format!(
        "The `call {}` command. Register it with `.command({})`, or run it from Rust with `{}::call(...)`.",
        name, marker, marker
    );
    let output = match &function.sig.output {
        ReturnType::Default => {
            return Err(Error::new(
                function.sig.span(),
                "a command returns `Option<ScreenState>`: `None` stays in the story",
            ));
        }
        ReturnType::Type(_, ty) => ty,
    };

    let context_pattern = match context {
        FnArg::Typed(typed) => &typed.pat,
        FnArg::Receiver(_) => unreachable!("a receiver is refused above"),
    };
    let context_type = match context {
        FnArg::Typed(typed) => &typed.ty,
        FnArg::Receiver(_) => unreachable!("a receiver is refused above"),
    };
    let patterns: Vec<&Pat> = arguments.iter().map(|argument| &*argument.pat).collect();
    let body = &function.block;

    Ok(quote! {
        #(#attributes)*
        #[doc = #documented]
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug)]
        #visibility struct #marker;

        impl #marker {
            #visibility fn call(
                #context_pattern: #context_type,
                #(#patterns: #types,)*
            ) -> #output #body
        }

        impl ::vn_engine::game::commands::Command for #marker {
            type Args = #args;
            const NAME: &'static str = #name;

            fn run(context: &mut ::vn_engine::context::GameContext, arguments: Self::Args) -> #output {
                #taken
                Self::call(context, #(#bindings,)*)
            }
        }
    })
}

fn is_rest(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    let Some(last) = path.path.segments.last() else {
        return false;
    };
    if last.ident != "Vec" {
        return false;
    }

    let syn::PathArguments::AngleBracketed(arguments) = &last.arguments else {
        return false;
    };
    matches!(
        arguments.args.first(),
        Some(syn::GenericArgument::Type(Type::Path(inner)))
            if inner.path.segments.last().is_some_and(|last| last.ident == "String")
    )
}

fn check_name(name: &str, span: Span) -> Result<(), Error> {
    let mut characters = name.chars();
    let start = characters.next();
    let valid = matches!(start, Some(c) if c.is_ascii_lowercase() || c == '_')
        && characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');

    match valid {
        true => Ok(()),
        false => Err(Error::new(
            span,
            format!(
                "`{}` can't be a command name: a story writes `call <name>`, which is lowercase letters, digits and `_`, not starting with a digit",
                name
            ),
        )),
    }
}

#[proc_macro_derive(StoryWord, attributes(word))]
pub fn story_word(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);

    match expand_word(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_word(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let syn::Data::Enum(data) = &input.data else {
        return Err(Error::new(
            input.span(),
            "a story word is an enum: one variant per word the story may write",
        ));
    };
    if !input.generics.params.is_empty() {
        return Err(Error::new(
            input.generics.span(),
            "a story word can't be generic: the words are fixed when the game is built",
        ));
    }
    if data.variants.is_empty() {
        return Err(Error::new(
            input.span(),
            "a story word needs at least one variant",
        ));
    }

    let mut names = Vec::new();
    let mut words = Vec::new();
    for variant in &data.variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(Error::new(
                variant.fields.span(),
                "a story word's variants carry nothing: the story writes one word",
            ));
        }
        let word = match word_attribute(variant)? {
            Some(word) => word,
            None => snake_case(&variant.ident.to_string()),
        };
        check_name(&word, variant.ident.span())?;
        if words.contains(&word) {
            return Err(Error::new(
                variant.ident.span(),
                format!("`{}` is already the word of another variant", word),
            ));
        }
        names.push(&variant.ident);
        words.push(word);
    }

    let name = &input.ident;
    let visibility = &input.vis;
    let count = names.len();
    let first = names[0];
    let listed = words.join(", ");
    let unknown = format!("unknown {} `{{}}`; expected one of {}", name, listed);

    Ok(quote! {
        impl #name {
            #visibility const ALL: [#name; #count] = [#(#name::#names,)*];

            #visibility fn as_str(&self) -> &'static str {
                match self {
                    #(#name::#names => #words,)*
                }
            }

            #visibility fn from_word(word: &str) -> ::core::option::Option<Self> {
                match word {
                    #(#words => ::core::option::Option::Some(#name::#names),)*
                    _ => ::core::option::Option::None,
                }
            }

            #visibility fn words() -> ::std::vec::Vec<::std::string::String> {
                ::std::vec![#(::std::string::String::from(#words),)*]
            }
        }

        impl ::core::default::Default for #name {
            fn default() -> Self {
                #name::#first
            }
        }

        impl ::core::fmt::Display for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::vn_engine::game::commands::FromArg for #name {
            fn kind() -> ::vn_engine::script::ParamKind {
                ::vn_engine::script::ParamKind::Choice(Self::words())
            }

            fn from_arg(arg: &str) -> ::core::option::Option<Self> {
                Self::from_word(arg)
            }
        }

        impl ::vn_engine::game::commands::Arg for #name {
            const OPTIONAL: bool = false;

            fn kind() -> ::vn_engine::script::ParamKind {
                <Self as ::vn_engine::game::commands::FromArg>::kind()
            }

            fn parse(arg: ::core::option::Option<&str>) -> ::core::result::Result<Self, ::std::string::String> {
                let arg = arg.ok_or_else(|| ::std::string::String::from("missing argument"))?;
                Self::from_word(arg).ok_or_else(|| {
                    ::std::format!(
                        "`{}` is not {}",
                        arg,
                        <Self as ::vn_engine::game::commands::FromArg>::kind()
                    )
                })
            }
        }

        impl ::vn_engine::serde::Serialize for #name {
            fn serialize<S: ::vn_engine::serde::Serializer>(
                &self,
                serializer: S,
            ) -> ::core::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> ::vn_engine::serde::Deserialize<'de> for #name {
            fn deserialize<D: ::vn_engine::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> ::core::result::Result<Self, D::Error> {
                let word = <::std::string::String as ::vn_engine::serde::Deserialize>::deserialize(
                    deserializer,
                )?;
                Self::from_word(&word).ok_or_else(|| {
                    <D::Error as ::vn_engine::serde::de::Error>::custom(::std::format!(
                        #unknown,
                        word
                    ))
                })
            }
        }
    })
}

fn word_attribute(variant: &syn::Variant) -> Result<Option<String>, Error> {
    let Some(attribute) = variant.attrs.iter().find(|a| a.path().is_ident("word")) else {
        return Ok(None);
    };
    let word: LitStr = attribute.parse_args()?;
    Ok(Some(word.value()))
}

fn snake_case(name: &str) -> String {
    let mut out = String::new();
    let mut previous: Option<char> = None;

    for c in name.chars() {
        let boundary = match (previous, c) {
            (Some(p), c) if c.is_ascii_uppercase() && !p.is_ascii_uppercase() => true,
            (Some(p), c) if c.is_ascii_digit() && p.is_ascii_alphabetic() => true,
            _ => false,
        };
        if boundary {
            out.push('_');
        }
        out.extend(c.to_lowercase());
        previous = Some(c);
    }
    out
}
