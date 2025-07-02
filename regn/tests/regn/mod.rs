use pretty_assertions::assert_eq;
use proc_macro2::TokenStream;
use quote::quote;
use regn::Codgen;
use regr::{Arena, Graph};

fn pretty(tok_stream: TokenStream) -> String {
    prettyplease::unparse(&syn::parse2::<syn::File>(tok_stream).unwrap())
}

#[test]
fn test_codgen_ctor() {
    let mut ar = Arena::new();
    let gr = Graph::dfa_in(&mut ar);
    let _ = Codgen::new(&gr);
}

#[test]
#[should_panic(expected = "only DFA graphs are supported")]
fn test_codgen_ctor_panics() {
    let mut ar = Arena::new();
    let gr = Graph::nfa_in(&mut ar);
    let _ = Codgen::new(&gr);
}

#[test]
fn test_codgen_produce() {
    let mut ar = Arena::new();
    let gr = Graph::dfa_in(&mut ar);
    let cd = Codgen::new(&gr);

    assert_eq!(
        pretty(cd.produce()),
        pretty(quote! {
            struct Automaton {
                state: u32,
                tr_table: [[u8; u8::MAX as usize]; 0usize],
            }

            impl Automaton {
                fn new() -> Self {
                    Self {
                        state: 0u32,
                        tr_table: [],
                    }
                }
            }
        })
    );
}
