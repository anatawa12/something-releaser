use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::fs::read_to_string;
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Pairs;
use syn::Token;
use syn::*;

// unsupported: {} in separators
#[proc_macro]
pub fn load_format_file(item: TokenStream) -> TokenStream {
    let manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR").unwrap().into();
    let Input(path, pre, post) = parse_macro_input!(item as Input);

    let file = read_to_string(manifest_dir.join(path)).unwrap();

    let pat = file
        .replace("{", "{{")
        .replace("}", "}}")
        .replace(pre.as_str(), "{")
        .replace(post.as_str(), "}");

    let lit = LitStr::new(&pat, Span::call_site());
    (quote! {
        #lit
    })
    .into()
}

struct Input(String, String, String);

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed = punctuated::Punctuated::<LitStr, Token![,]>::parse_terminated(input)?;

        fn next(iter: &mut Pairs<LitStr, Token![,]>, span: Span, name: &str) -> Result<String> {
            Ok(iter
                .next()
                .ok_or_else(|| Error::new(span, name))?
                .value()
                .value())
        }
        let mut iter = parsed.pairs();
        let path = next(&mut iter, input.span(), "expects path")?;
        let prefix = next(&mut iter, input.span(), "expects prefix")?;
        let suffix = next(&mut iter, input.span(), "expects suffix")?;

        if let Some(some) = iter.next() {
            return Err(Error::new_spanned(some, "too many"));
        }

        Ok(Self(path, prefix, suffix))
    }
}
