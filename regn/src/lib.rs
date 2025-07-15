use proc_macro2::{Delimiter, Group, TokenStream};
use quote::{TokenStreamExt, quote};
use regr::Graph;
use std::collections::HashMap;
use std::str::FromStr;

type TransitionTable = Vec<[usize; 1 << u8::BITS]>;

pub struct Codgen {
    tr_table: TransitionTable,
    invalid_id: usize,
    start_id: usize,
    min_accept_id: usize,
}

impl<'a> Codgen {
    pub fn new(graph: &Graph<'a>) -> Self {
        assert!(graph.is_dfa(), "only DFA graphs are supported");
        assert!(!graph.is_empty(), "can't generate code for an empty graph");

        let (id_map, invalid_id, start_id, min_accept_id) = Self::build_id_map(graph);
        let tr_table = Self::build_tr_table(graph, invalid_id, &id_map);

        Codgen {
            tr_table,
            invalid_id,
            start_id,
            min_accept_id,
        }
    }

    /// Builds a map from node IDs to their respective indices in the transition
    /// table, rearranging them in the order that all acceptable nodes are
    /// placed before non-acceptable nodes.
    ///
    /// Returns a tuple containing the map and the total number of acceptable
    /// nodes.
    fn build_id_map(graph: &Graph<'a>) -> (HashMap<u64, usize>, usize, usize, usize) {
        let arena_len = graph.arena().nodes().len();
        let mut acc_nodes = Vec::with_capacity(arena_len);
        let mut non_acc_nodes = Vec::with_capacity(arena_len);
        graph.for_each_node(|node| {
            if node.is_acceptable() {
                acc_nodes.push(node);
            } else {
                non_acc_nodes.push(node);
            }
        });

        assert_eq!(acc_nodes.len() + non_acc_nodes.len(), arena_len);

        let mut id_map = HashMap::with_capacity(acc_nodes.len() + non_acc_nodes.len());
        let mut id = 0usize;
        for node in &non_acc_nodes {
            id_map.insert(node.uid(), id);
            id += 1;
        }
        // push acceptable states to the end of transition table
        for node in &acc_nodes {
            id_map.insert(node.uid(), id);
            id += 1;
        }

        let invalid_id = id_map.len();
        let start_id = id_map[&graph.start_node().uid()];
        let min_accept_id = non_acc_nodes.len();

        (id_map, invalid_id, start_id, min_accept_id)
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

    pub fn impl_state_machine(&self) -> TokenStream {
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
            if tr_table_len <= 1 << u8::BITS {
                quote! { u8 }
            } else if tr_table_len <= 1 << u16::BITS {
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
        let min_accept_state = self.min_accept_id;

        quote! {
            struct StateMachine {
                state: usize,
            }

            impl StateMachine {
                const START_STATE: usize = #start_state;
                const INVALID_STATE: usize = #invalid_state;
                const MIN_ACCEPT_STATE: usize = #min_accept_state;
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
                fn next(&mut self, byte: u8) {
                    self.state = unsafe {
                        Self::TRANSITION_TABLE
                            .get_unchecked(state)
                            .get_unchecked(byte as usize)
                    } as usize;

                    debug_assert!(self.state < Self::STATES_NUM, "invalid new state value {state}");
                }
            }
        }
    }

    pub fn impl_match(&self) -> TokenStream {
        quote! {
            struct Match0<'h> {
                capture: &'h str,
                start: usize,
            }

            impl<'h> Match0<'h> {
                #[inline]
                pub fn start(&self) -> usize {
                    self.start
                }

                #[inline]
                fn end(&self) -> usize {
                    self.start + self.capture.len()
                }

                #[inline]
                fn len(&self) -> usize {
                    self.capture.len()
                }

                #[inline]
                fn is_empty(&self) -> bool {
                    self.capture.is_empty()
                }

                #[inline]
                fn range(&self) -> std::ops::Range<usize> {
                    self.start..self.end()
                }

                #[inline]
                fn as_str(&self) -> &'h str {
                    self.capture
                }

                #[inline]
                pub fn as_bytes(&self) -> &'h [u8] {
                    self.as_str().as_bytes()
                }
            }

            impl<'h> MatchBytes<'h> for Match0<'h> {
                #[inline]
                fn start(&self) -> usize {
                    self.start()
                }

                #[inline]
                fn end(&self) -> usize {
                    self.end()
                }

                #[inline]
                fn len(&self) -> usize {
                    self.len()
                }

                #[inline]
                fn is_empty(&self) -> bool {
                    self.is_empty()
                }

                #[inline]
                fn range(&self) -> std::ops::Range<usize> {
                    self.range()
                }

                #[inline]
                fn as_bytes(&self) -> &'h [u8] {
                    self.as_bytes()
                }
            }

            impl<'h> MatchStr<'h> for Match0<'h> {
                #[inline]
                fn as_str(&self) -> &'h str {
                    self.as_str()
                }
            }
        }
    }

    pub fn impl_regex(&self) -> TokenStream {
        quote! {
            pub(crate) struct Regex {
                state_machine: StateMachine,
            }

            impl Regex {
                pub(crate) fn new() -> Self {
                    Self {
                        state_machine: StateMachine::new(),
                    }
                }

                pub fn match_at<'h>(&mut self, haystack: &'h str) -> Option<Match<'h>>{
                    for byte in haystack.as_bytes().iter() {
                        self.state_machine.next(byte);
                        if self.state_machine.is_acceptable() {
                            Match
                        }
                    }
                }
            }
        }
    }
}
