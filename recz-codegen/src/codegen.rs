use crate::Config;
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use recz_adt::Set;
use recz_graph::{CaptureLabel, Edge, Graph, Node, NodeKind, Tag};

pub struct CodeGen {
    config: Config,
    num_labels: Vec<u32>,
    str_labels: Vec<Box<str>>,
    dfa: Graph,
}

impl CodeGen {
    pub fn build(config: Config, dfa: Graph) -> Self {
        let num_labels: Vec<_> = dfa
            .groups()
            .filter_map(|gr| {
                if let CaptureLabel::Num(num) = gr.label() {
                    Some(num)
                } else {
                    None
                }
            })
            .collect();
        let str_labels: Vec<_> = dfa
            .groups()
            .filter_map(|gr| {
                if let CaptureLabel::Str(str) = gr.label() {
                    Some(str)
                } else {
                    None
                }
            })
            .collect();

        Self {
            config,
            num_labels,
            str_labels,
            dfa,
        }
    }

    pub fn generate(&self) -> TokenStream {
        let vis = &self.config.visibility;
        let hay_ty = &self.config.haystack_ty;
        let pattern = &self.config.pattern;

        let as_str_fn = if self.config.haystack_ty.to_string() == "str" {
            quote! { as_str }
        } else {
            quote! { as_bytes }
        };

        let labels_len = self.num_labels.len() + self.str_labels.len();
        let labels = self.dfa.groups().map(|group| match group.label() {
            CaptureLabel::Num(n) => quote! { Label::Num(#n) },
            CaptureLabel::Str(s) => quote! { Label::Str(#s) },
        });

        let capture_by_num_body = if !self.num_labels.is_empty() {
            let match_arms = self.num_labels.iter().map(|num| {
                let group = self.dfa.group(CaptureLabel::Num(*num));
                let idx = group.index();
                quote!(#num => #idx)
            });
            quote! {
                let idx = match label {
                    #(#match_arms,)*
                    _ => return None,
                };
                self.range_to_capture(idx)
            }
        } else {
            quote! { None }
        };

        let capture_by_str_body = if !self.str_labels.is_empty() {
            let match_arms = self.str_labels.iter().map(|str| {
                let group = self.dfa.group(CaptureLabel::Str(str.clone()));
                let idx = group.index();
                quote!(#str => #idx)
            });
            quote! {
                let idx = match label {
                    #(#match_arms,)*
                    _ => return None,
                };
                self.range_to_capture(idx)
            }
        } else {
            quote! { None }
        };

        let mut mtch_gen = MatchGenerator::new(&self.dfa);
        mtch_gen.run();

        let mtch_states = mtch_gen.states();
        let start_node = self.dfa.start_node();
        let start_state_ident = format_ident!("{start_node}");
        let match_branches = mtch_gen.match_arms();

        quote!(
            use ::core::convert::AsRef;
            use ::core::option::Option;
            use ::core::range::Range;
            use ::recz::Label;

            //------------------------------------------------------------------
            // Capture
            //------------------------------------------------------------------

            #[derive(Debug, Clone, PartialEq, Eq)]
            #vis struct Capture<'h> {
                capture: &'h #hay_ty,
                start: usize,
            }

            impl<'h> Capture<'h> {
                #[inline]
                #vis fn #as_str_fn(&self) -> &'h #hay_ty {
                    self.capture
                }

                #[inline]
                #vis const fn start(&self) -> usize {
                    self.start
                }

                #[inline]
                #vis const fn end(&self) -> usize {
                    self.start + self.capture.len()
                }

                #[inline]
                #vis const fn range(&self) -> Range<usize> {
                    Range {
                        start: self.start(),
                        end: self.end(),
                    }
                }

                #[inline]
                #vis const fn len(&self) -> usize {
                    self.capture.len()
                }

                #[inline]
                #vis const fn is_empty(&self) -> bool {
                    self.capture.is_empty()
                }
            }

            //------------------------------------------------------------------
            // Match
            //------------------------------------------------------------------

            #[derive(Debug, Clone, PartialEq, Eq)]
            #vis struct Match<'h> {
                hay: &'h #hay_ty,
                ranges: [Range<usize>; #labels_len]
            }

            impl<'h> Match<'h> {
                #[inline]
                #vis fn #as_str_fn(&self) -> &'h #hay_ty {
                    &self.hay[self.range()]
                }

                #[inline]
                #vis const fn start(&self) -> usize {
                    self.ranges[0].start
                }

                #[inline]
                #vis const fn end(&self) -> usize {
                    self.ranges[0].end
                }

                #[inline]
                #vis const fn range(&self) -> Range<usize> {
                    self.ranges[0]
                }

                #[inline]
                #vis const fn len(&self) -> usize {
                    self.end() - self.start()
                }

                #[inline]
                #vis const fn is_empty(&self) -> bool {
                    self.end() == self.start()
                }
            }

            impl<'h> Match<'h> {
                #[inline]
                #vis const fn haystack(&self) -> &'h #hay_ty {
                    self.hay
                }

                #[inline]
                #vis fn capture<'a>(&self, label: impl Into<Label<'a>>) -> Option<Capture<'h>> {
                    match label.into() {
                        Label::Str("") => None,
                        Label::Str(s) => self.capture_by_str(s),
                        Label::Num(n) => self.capture_by_num(n),
                    }
                }

                #[inline]
                #vis fn capture_by_num(&self, label: u32) -> Option<Capture<'h>> {
                    #capture_by_num_body
                }

                #[inline]
                #vis fn capture_by_str(&self, label: &str) -> Option<Capture<'h>> {
                    #capture_by_str_body
                }
            }

            impl<'h> Match<'h> {
                #[inline]
                fn range_to_capture(&self, idx: usize) -> Option<Capture<'h>> {
                    let range = self.ranges[idx];
                    if range.end != usize::MAX {
                        Some(Capture {
                            capture: &self.hay[range],
                            start: range.start,
                        })
                    } else {
                        None
                    }
                }
            }

            //------------------------------------------------------------------
            // Regex
            //------------------------------------------------------------------

            #[derive(Debug, Clone, PartialEq, Eq)]
            #vis struct Regex;

            impl Regex {
                #[inline]
                #vis const fn pattern(&self) -> &'static str {
                    #pattern
                }

                #[inline]
                #vis const fn capture_labels(&self) -> &'static [Label<'static>] {
                    &[#(#labels),*]
                }

                #[inline]
                #vis fn mtch<'h>(&self, haystack: &'h #hay_ty) -> Option<Match<'h>> {
                    let mut m = None;
                    self.match_impl(haystack, &mut m);
                    m
                }

                #[inline]
                #vis fn find<'h>(&self, haystack: &'h #hay_ty) -> Option<Match<'h>> {
                    unimplemented!()
                }
            }

            impl Regex {
                fn match_impl<'h>(&self, haystack: &'h #hay_ty, m: &mut Option<Match<'h>>) {
                    const INVALID_RANGE: Range<usize> = Range {
                        start: usize::MAX,
                        end: usize::MAX,
                    };

                    #[allow(non_camel_case_types)]
                    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                    enum stt {
                        #(#mtch_states,)*
                    }

                    let source = AsRef::<[u8]>::as_ref(haystack);
                    let mut ranges = [INVALID_RANGE; #labels_len];
                    let mut pos = 0usize;
                    let mut curr_state = stt::#start_state_ident;

                    'main: loop {
                        match curr_state {
                            #(#match_branches)*
                        }
                        pos += 1;
                    }
                }
            }
        )
    }
}

struct MatchGenerator<'d> {
    dfa: &'d Graph,
    states: Set<syn::Ident>,
    state_arms: Vec<TokenStream>,
    visited: Set<Node<'d>>,
}

impl<'d> MatchGenerator<'d> {
    fn new(dfa: &'d Graph) -> Self {
        Self {
            dfa,
            states: Set::default(),
            state_arms: Vec::default(),
            visited: Set::default(),
        }
    }

    fn run(&mut self) {
        self.handle_node(self.dfa.start_node());
    }

    fn handle_node(&mut self, node: Node<'d>) {
        if self.visited.contains(&node) {
            return;
        }
        self.visited.insert(node);

        let curr_state_ident = format_ident!("{node}");
        self.states.insert(curr_state_ident.clone());

        let state_arm = match node.kind() {
            NodeKind::Epilogue => {
                assert_eq!(node.target_count(), 0);
                quote! {
                    stt::#curr_state_ident => {
                        *m = Some(Match { hay: haystack, ranges });
                        break 'main;
                    }
                }
            }
            NodeKind::Normal | NodeKind::Final => {
                let mut symbol_arms = Vec::<TokenStream>::with_capacity(node.target_count());
                let mut default_arm = quote! { _ => break 'main };

                for (edge, target) in node.targets() {
                    let symbol_arm = make_match_arm(node, edge, target);
                    if edge.is_epsilon() {
                        default_arm = symbol_arm;
                    } else {
                        symbol_arms.push(symbol_arm);
                    }
                    self.handle_node(target);
                }

                quote! {
                    stt::#curr_state_ident => {
                        match source.get(pos) {
                            #(#symbol_arms,)*
                            #default_arm,
                        }
                    }
                }
            }
        };
        self.state_arms.push(state_arm);
    }

    fn states(&self) -> impl Iterator<Item = &syn::Ident> {
        self.states.iter()
    }

    fn match_arms(&self) -> impl Iterator<Item = &TokenStream> {
        self.state_arms.iter()
    }
}

fn make_match_arm<'d>(source: Node<'d>, edge: Edge<'d>, target: Node<'d>) -> TokenStream {
    let pattern = make_match_arm_pat(edge);
    let expression = make_match_arm_expr(source, edge, target);
    quote! { #pattern => #expression }
}

fn make_match_arm_pat<'d>(edge: Edge<'d>) -> TokenStream {
    if edge.is_epsilon() {
        return quote!(_);
    }
    let ranges = edge.ranges().map(|r| {
        if r.width() == Some(1) {
            let byte = Literal::byte_character(r.start());
            quote!(#byte)
        } else {
            let start = Literal::byte_character(r.start());
            let last = Literal::byte_character(r.last());
            quote!(#start..=#last)
        }
    });
    quote! { Some(#(#ranges)|*) }
}

fn make_match_arm_expr<'d>(source: Node<'d>, edge: Edge<'d>, target: Node<'d>) -> TokenStream {
    let update_result_match = if source.is_final() && !target.is_final() && !target.is_epilogue() {
        quote! { *m = Some(Match { hay: haystack, ranges }); }
    } else {
        quote! {}
    };

    let tag_exprs = edge.tags().map(|tag| match tag {
        Tag::OpenGroup(index) => quote! { ranges[#index as usize].start = pos; },
        Tag::CloseGroup(index) => quote! { ranges[#index as usize].end = pos; },
        Tag::DeleteGroup(index) => quote! { ranges[#index as usize].end = usize::MAX; },
    });

    let target_state_ident = format_ident!("{target}");
    quote!({
        #(#tag_exprs)*
        curr_state = stt::#target_state_ident;
        #update_result_match
    })
}

// Possible optimizations:
//
// 1. If final state doesnt have output edges exept for epsilone edge to
// epilogue without tags, it can be removed
//
// 2. If node has only one outgoing edge with one symbol, it can be combined
// with the next node.
