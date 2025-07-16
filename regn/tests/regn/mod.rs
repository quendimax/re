use pretty_assertions::assert_eq;
use proc_macro2::TokenStream;
use quote::quote;
use regn::Codgen;
use regr::{Arena, Graph};

fn pretty(tok_stream: TokenStream) -> String {
    prettyplease::unparse(&syn::parse2::<syn::File>(tok_stream).unwrap())
}

#[test]
fn codegen_new() {
    let mut ar = Arena::new();
    let gr = Graph::dfa_in(&mut ar);
    let _ = gr.node();
    let _ = Codgen::new(&gr);
}

#[test]
#[should_panic(expected = "only DFA graphs are supported")]
fn codgen_new_panics() {
    let mut ar = Arena::new();
    let gr = Graph::nfa_in(&mut ar);
    let _ = Codgen::new(&gr);
}

#[test]
#[should_panic(expected = "can't generate code for an empty graph")]
fn codgen_produce_for_empty_graph() {
    let mut ar = Arena::new();
    let gr = Graph::dfa_in(&mut ar);
    let _ = Codgen::new(&gr);
}

#[test]
fn codgen_produce() {
    let mut ar = Arena::new();
    let gr = Graph::dfa_in(&mut ar);
    let _ = gr.node();
    let cd = Codgen::new(&gr);

    assert_eq!(
        pretty(cd.gen_state_machine()),
        pretty(quote! {
            #[derive(Debug)]
            struct StateMachine {
                state: usize,
            }

            impl StateMachine {
                const START_STATE: usize = 0usize;
                const INVALID_STATE: usize = 1usize;
                const MIN_ACCEPT_STATE: usize = 1usize;
                const STATES_NUM: usize = 1usize;

                const TRANSITION_TABLE: [[u8; 256usize]; Self::STATES_NUM] = [
                    [
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    ]
                ];

                #[inline]
                fn new() -> Self {
                    Self {
                        state: Self::START_STATE,
                    }
                }

                #[inline]
                fn reset(&mut self) {
                    self.state = Self::START_STATE;
                }

                #[inline]
                fn is_start(&self) -> bool {
                    self.state == Self::START_STATE
                }

                #[inline]
                fn is_acceptable(&self) -> bool {
                    self.state >= Self::MIN_ACCEPT_STATE
                }

                #[inline]
                fn is_invalid(&self) -> bool {
                    self.state == Self::INVALID_STATE
                }

                #[inline]
                fn next(&mut self, byte: u8) {
                    self.state = *unsafe {
                        Self::TRANSITION_TABLE
                            .get_unchecked(self.state)
                            .get_unchecked(byte as usize)
                    } as usize;

                    debug_assert!(
                        self.state < Self::STATES_NUM,
                        "invalid new state value {}",
                        self.state,
                    );
                }
            }
        })
    );
}
