use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use regn::CodeGen;
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

    let mut nfa_arena = Arena::new();
    let nfa = Graph::nfa_in(&mut nfa_arena);

    let mut parser = Parser::new(&nfa, Utf8Encoder);
    let end_node = parser
        .parse(&lit.value(), nfa.start_node())
        .map_err(|err| syn::Error::new(lit.span(), err))?;
    end_node.finalize();

    let mut dfa_arena = Arena::new();
    let dfa = nfa.determine_in(&mut dfa_arena);

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
