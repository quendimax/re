use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use regr::{Arena, Graph};
use renc::Utf8Encoder;
use resy::Parser;
use syn::{LitStr, Result};

pub(crate) fn re_impl(input: TokenStream2) -> Result<TokenStream2> {
    let lit = syn::parse2::<LitStr>(input)?;
    if !lit.suffix().is_empty() {
        let v = lit.token().to_string();
        let loc = v.len() - lit.suffix().len()..v.len();
        let span = lit.token().subspan(loc).unwrap_or_else(|| lit.span());
        return Err(syn::Error::new(
            span,
            "suffixes for string literals are not allowed",
        ));
    }

    let mut arena = Arena::new();
    let gr = Graph::nfa_in(&mut arena);

    let mut parser = Parser::new(&gr, Utf8Encoder);
    let end_node = parser
        .parse(&lit.value(), gr.start_node())
        .map_err(|err| syn::Error::new(lit.span(), err))?;
    end_node.acceptize();

    // TODO: generate code for the graph

    Ok(quote!())
}
