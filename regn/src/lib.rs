use proc_macro2::TokenStream;
use quote::quote;
use regr::{Graph, Node};
use std::collections::{HashMap, HashSet};

pub struct Codgen<'a, 'g> {
    graph: &'g Graph<'a>,
    tr_table: Vec<[usize; u8::MAX as usize + 1]>,
    id_map: HashMap<Node<'a>, usize>,
}

impl<'a, 'g> Codgen<'a, 'g> {
    pub fn new(graph: &'g Graph<'a>) -> Self {
        assert!(graph.is_dfa(), "only DFA graphs are supported");
        let tr_table = Vec::new();
        Codgen {
            graph,
            tr_table,
            id_map: HashMap::new(),
        }
    }

    /// Evaluates the given DFA graph.
    pub fn fill_id_map(&mut self) {
        fn eval<'a, 'g>(node: Node<'a>, codgen: &mut Codgen<'a, 'g>) {
            if codgen.id_map.contains_key(&node) {
                return;
            }
            let id = codgen.id_map.len();
            codgen.id_map.insert(node, id);
            for target in node.targets().keys() {
                eval(*target, codgen);
            }
        }
        eval(self.graph.start_node(), self);
    }

    fn get_id(&self, node: Node<'a>) -> usize {
        *self
            .id_map
            .get(&node)
            .expect("run fill_id_map before using get_id")
    }

    pub fn fill_tables(&mut self) {
        self.tr_table
            .resize(self.id_map.len(), [0; u8::MAX as usize + 1]);

        let mut visited = HashSet::new();
        fn fill<'a, 'g>(node: Node<'a>, codgen: &mut Codgen<'a, 'g>, visited: &mut HashSet<u64>) {
            if visited.contains(&node.uid()) {
                return;
            }
            visited.insert(node.uid());
            let id = codgen.get_id(node);
            for (target, tr) in node.targets().iter() {
                for sym in tr.symbols() {
                    codgen.tr_table[id][sym as usize] = codgen.get_id(*target);
                }
                fill(*target, codgen, visited);
            }
        }
        fill(self.graph.start_node(), self, &mut visited);
    }

    pub fn produce(&self) -> TokenStream {
        let tr_table_len: usize = self.tr_table.len();
        let mut tr_table_quoted = Vec::new();
        for line in &self.tr_table {
            tr_table_quoted.push(quote! { [ #(#line),*; u8::MAX as usize]});
        }

        const MAX_LEN_FOR_U8: usize = u8::MAX as usize + 1;
        const MAX_LEN_FOR_U16: usize = u16::MAX as usize + 1;
        const MAX_LEN_FOR_U32: usize = u32::MAX as usize + 1;

        let id_type = {
            if tr_table_len <= MAX_LEN_FOR_U8 {
                quote! { u8 }
            } else if tr_table_len <= MAX_LEN_FOR_U16 {
                quote! { u16 }
            } else if tr_table_len <= MAX_LEN_FOR_U32 {
                quote! { u32 }
            } else {
                quote! { u64 }
            }
        };

        quote! {
            struct Automaton {
                state: u32,
                tr_table: [[#id_type; u8::MAX as usize]; #tr_table_len],
            }

            impl Automaton {
                fn new() -> Self {
                    Self {
                        state: 0u32,
                        tr_table: [#(#tr_table_quoted),*],
                    }
                }
            }
        }
    }
}
