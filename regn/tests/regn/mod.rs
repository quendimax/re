use pretty_assertions::assert_eq;
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use regn::Codgen;
use regr::{Arena, Graph};

fn flatten(tok_stream: TokenStream) -> Vec<String> {
    let mut vec = Vec::new();
    fn flatten_fn(tok_stream: TokenStream, vec: &mut Vec<String>) {
        for tok_tree in tok_stream {
            match tok_tree {
                TokenTree::Ident(ident) => vec.push(ident.to_string()),
                TokenTree::Punct(punct) => vec.push(punct.to_string()),
                TokenTree::Literal(literal) => vec.push(literal.to_string()),
                TokenTree::Group(group) => flatten_fn(group.stream(), vec),
            }
        }
    }
    flatten_fn(tok_stream, &mut vec);
    vec
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
        flatten(cd.produce()),
        flatten(quote! {
            struct Automaton {
                state: u32,
                tr_table: [[u8; u8::MAX as usize]; 0usize],
                accept_table: [bool; 0usize],
            }

            Automaton {
                state: 0u32,
                tr_table: [],
                accept_table: [false; 0usize],
            }
        })
    );
}
