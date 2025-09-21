use crate::codegen::CodeGen;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use regr::{Arena, Graph, Translator};
use resy::{Parser, enc::Utf8Encoder};
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

    let mut nfa_arena = Arena::new();
    let nfa = Graph::nfa_in(&mut nfa_arena);
    let start_node = nfa.start_node();
    let end_node = nfa.node();

    let parser = Parser::new(Utf8Encoder);
    let hir = parser
        .parse(&lit.value())
        .map_err(|err| syn::Error::new(lit.span(), err))?;

    let mut translator = Translator::new(&nfa);
    translator.translate(&hir, start_node, end_node);

    let mut dfa_arena = Arena::new();
    let dfa = nfa.determinize_in(&mut dfa_arena);

    let cogen = CodeGen::new(&dfa);
    let state_machine_code = cogen.gen_state_machine();
    let match_code = cogen.gen_match();
    let regex_code = cogen.gen_regex();

    Ok(quote!(
        {
            mod adhoc {
                #state_machine_code

                #match_code

                #regex_code
            }

            adhoc::Regex::new()
        }
    ))
}
