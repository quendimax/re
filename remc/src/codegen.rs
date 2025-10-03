use proc_macro2::{Delimiter, Group, TokenStream};
use quote::{TokenStreamExt, quote};
use regr::Graph;
use std::collections::HashMap;
use std::str::FromStr;

type TransitionTable = Vec<[usize; 1 << u8::BITS]>;

pub struct CodeGen {
    tr_table: TransitionTable,
    invalid_id: usize,
    start_id: usize,
    first_non_final_id: usize,
}

impl<'a> CodeGen {
    pub fn new(graph: &Graph<'a>) -> Self {
        assert!(!graph.is_empty(), "can't generate code for an empty graph");

        let (id_map, invalid_id, start_id, first_non_final_id) = Self::build_id_map(graph);
        let tr_table = Self::build_tr_table(graph, invalid_id, &id_map);

        CodeGen {
            tr_table,
            invalid_id,
            start_id,
            first_non_final_id,
        }
    }

    /// Builds a map from node IDs to their respective indices in the transition
    /// table, rearranging them in the order that all final nodes are
    /// placed before non-final nodes.
    ///
    /// Returns a tuple containing the map and the total number of final nodes.
    fn build_id_map(graph: &Graph<'a>) -> (HashMap<u64, usize>, usize, usize, usize) {
        let arena_len = graph.arena().nodes().len();
        let mut final_nodes = Vec::with_capacity(arena_len);
        let mut non_final_nodes = Vec::with_capacity(arena_len);
        graph.for_each_node(|node| {
            if node.is_final() {
                final_nodes.push(node);
            } else {
                non_final_nodes.push(node);
            }
        });

        assert_eq!(final_nodes.len() + non_final_nodes.len(), arena_len);

        let mut id_map = HashMap::with_capacity(final_nodes.len() + non_final_nodes.len());
        let mut id = 0usize;
        // push final states to the beginning of the transition table
        for node in &final_nodes {
            id_map.insert(node.uid(), id);
            id += 1;
        }
        for node in &non_final_nodes {
            id_map.insert(node.uid(), id);
            id += 1;
        }

        let invalid_id = id_map.len();
        let start_id = id_map[&graph.start_node().uid()];
        let first_non_final_id = final_nodes.len();

        (id_map, invalid_id, start_id, first_non_final_id)
    }

    fn build_tr_table(
        graph: &Graph<'a>,
        invalid_id: usize,
        id_map: &HashMap<u64, usize>,
    ) -> TransitionTable {
        let mut tr_table = vec![[invalid_id; 1 << u8::BITS]; id_map.len()];
        graph.for_each_node(|node| {
            let node_id = id_map[&node.uid()];
            for (target, tr) in node.targets().iter() {
                let target_id = id_map[&target.uid()];
                for sym in tr.symbols() {
                    tr_table[node_id][sym as usize] = target_id;
                }
            }
        });
        tr_table
    }

    pub fn gen_state_machine(&self) -> TokenStream {
        let tr_table_len: usize = self.tr_table.len();
        let mut tr_table_lines = Vec::new();
        for line in &self.tr_table {
            let mut token_line = TokenStream::new();
            for num in line {
                // to remove suffix
                let stream = TokenStream::from_str(&format!("{num},")).unwrap();
                token_line.append_all(stream);
            }
            tr_table_lines.push(Group::new(Delimiter::Bracket, token_line))
        }

        let state_type = {
            // we need to leave one value for the invalid state
            if tr_table_len < 1 << u8::BITS {
                quote! { u8 }
            } else if tr_table_len < 1 << u16::BITS {
                quote! { u16 }
            } else {
                panic!("number of states {tr_table_len} is too big for calculation");
            }
        };

        // Number of all possible bytes, i.e. transitions.
        let bytes_num: usize = 1 << u8::BITS;

        // Number of states in the automaton.
        let states_num: usize = tr_table_len;

        let start_state = self.start_id;
        let invalid_state = self.invalid_id;
        let first_non_final_state = self.first_non_final_id;

        quote! {
            #[derive(Debug)]
            struct StateMachine {
                state: usize,
            }

            impl StateMachine {
                const START_STATE: usize = #start_state;
                const INVALID_STATE: usize = #invalid_state;
                const FIRST_NON_FINAL_STATE: usize = #first_non_final_state;
                const STATES_NUM: usize = #states_num;

                const TRANSITION_TABLE: [[#state_type; #bytes_num]; Self::STATES_NUM] = [
                    #(#tr_table_lines),*
                ];

                #[inline]
                fn new() -> Self {
                    Self {
                        state: Self::START_STATE,
                    }
                }

                #[inline]
                fn is_final(&self) -> bool {
                    self.state < Self::FIRST_NON_FINAL_STATE
                }

                #[inline]
                fn is_invalid(&self) -> bool {
                    self.state == Self::INVALID_STATE
                }

                #[inline]
                fn next(&mut self, byte: u8) {
                    ::core::debug_assert!(
                        self.state < Self::STATES_NUM,
                        "transition from invalid state {} is not allowed",
                        self.state,
                    );

                    self.state = *unsafe {
                        Self::TRANSITION_TABLE
                            .get_unchecked(self.state)
                            .get_unchecked(byte as usize)
                    } as usize;
                }
            }
        }
    }

    pub fn gen_match(&self) -> TokenStream {
        let vis = quote!(pub);
        quote! {
            #[derive(Debug, PartialEq, Eq)]
            #vis struct Match<'h> {
                capture: &'h str,
                start: usize,
            }

            impl<'h> Match<'h> {
                #[inline]
                #vis fn start(&self) -> usize {
                    self.start
                }

                #[inline]
                #vis fn end(&self) -> usize {
                    self.start + self.capture.len()
                }

                #[inline]
                #vis fn len(&self) -> usize {
                    self.capture.len()
                }

                #[inline]
                #vis fn is_empty(&self) -> bool {
                    self.capture.is_empty()
                }

                #[inline]
                #vis fn range(&self) -> ::core::ops::Range<usize> {
                    self.start..self.end()
                }

                #[inline]
                #vis fn as_str(&self) -> &'h str {
                    self.capture
                }

                #[inline]
                #vis fn as_bytes(&self) -> &'h [u8] {
                    self.as_str().as_bytes()
                }
            }
        }
    }

    pub fn gen_regex(&self) -> TokenStream {
        let vis = quote!(pub);
        quote! {
            #[derive(Debug)]
            pub struct Regex;

            impl Regex {
                #[inline]
                #vis fn new() -> Self {
                    Self
                }

                #vis fn match_at<'h>(&mut self, haystack: &'h str, start: usize) -> Option<Match<'h>>{
                    let mut state_machine = StateMachine::new();
                    let mut final_index = None;
                    if state_machine.is_final() {
                        final_index = Some(0);
                    }
                    for (i, byte) in haystack[start..].as_bytes().iter().enumerate() {
                        state_machine.next(*byte);
                        if state_machine.is_final() {
                            final_index = Some(i + 1);
                        }
                        if state_machine.is_invalid() {
                            break;
                        }
                    }
                    final_index.map(|index| Match {
                        capture: &haystack[start..start + index],
                        start,
                    })
                }
            }
        }
    }
}
