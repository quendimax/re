// It is more a playground than a test collection.

use pretty_assertions::assert_eq;

#[test]
fn attempt_0() {
    let re = {
        mod adhoc {
            use recz::*;

            pub(crate) struct Match0<'h> {
                capture: &'h str,
                start: usize,
            }

            impl<'h> Match0<'h> {
                pub(crate) fn start(&self) -> usize {
                    self.start
                }

                pub(crate) fn end(&self) -> usize {
                    self.start + self.capture.len()
                }

                pub(crate) fn len(&self) -> usize {
                    self.capture.len()
                }

                pub(crate) fn is_empty(&self) -> bool {
                    self.capture.is_empty()
                }

                pub(crate) fn range(&self) -> std::ops::Range<usize> {
                    self.start..self.end()
                }

                pub(crate) fn as_str(&self) -> &'h str {
                    self.capture
                }

                pub(crate) fn as_bytes(&self) -> &'h [u8] {
                    self.as_str().as_bytes()
                }
            }

            impl<'h> MatchBytes<'h> for Match0<'h> {
                fn start(&self) -> usize {
                    self.start()
                }

                fn end(&self) -> usize {
                    self.end()
                }

                fn len(&self) -> usize {
                    self.len()
                }

                fn is_empty(&self) -> bool {
                    self.is_empty()
                }

                fn range(&self) -> std::ops::Range<usize> {
                    self.range()
                }

                fn as_bytes(&self) -> &'h [u8] {
                    self.as_bytes()
                }
            }

            impl<'h> MatchStr<'h> for Match0<'h> {
                fn as_str(&self) -> &'h str {
                    self.as_str()
                }
            }

            pub(crate) struct Regex;

            impl RegexStr for Regex {
                fn match_at<'h>(
                    &self,
                    haystack: &'h str,
                    start: usize,
                ) -> Option<impl MatchStr<'h>> {
                    self.match_at(haystack, start)
                }

                fn match_iter<'h>(
                    &self,
                    haystack: &'h str,
                ) -> impl Iterator<Item = impl MatchStr<'h>> {
                    [Match0 {
                        capture: haystack,
                        start: 0,
                    }]
                    .into_iter()
                }
            }

            impl Regex {
                pub(crate) fn match_at<'h>(
                    &self,
                    haystack: &'h str,
                    _start: usize,
                ) -> Option<Match0<'h>> {
                    Some(Match0 {
                        capture: &haystack[0..haystack.len()],
                        start: 0,
                    })
                }
            }
        }
        adhoc::Regex
    };

    let m = re.match_at("hello", 0).unwrap();
    assert_eq!(m.start(), 0);
    assert_eq!(m.end(), 5);
    assert_eq!(m.len(), 5);
    assert!(!m.is_empty());
    assert_eq!(m.range(), 0..5);
    assert_eq!(m.as_str(), "hello");
    assert_eq!(m.as_bytes(), &[104, 101, 108, 108, 111]);

    test_my_match(m);
}

fn test_my_match<'h>(m: impl recz::MatchStr<'h>) {
    _ = m.start();
    //
}

#[test]
fn attempt_1() {
    // re!("42");
    let re = {
        mod adhoc {
            #[derive(Debug)]
            struct StateMachine {
                state: usize,
            }

            impl StateMachine {
                const START_STATE: usize = 1usize;
                const INVALID_STATE: usize = 3usize;
                const FIRST_NON_FINAL_STATE: usize = 1usize;
                const STATES_NUM: usize = 3usize;
                const TRANSITION_TABLE: [[u8; 256usize]; Self::STATES_NUM] = [
                    [
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3,
                    ],
                    [
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 2u8, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3,
                    ],
                    [
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        0u8, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                        3, 3, 3, 3, 3, 3, 3,
                    ],
                ];

                fn new() -> Self {
                    Self {
                        state: Self::START_STATE,
                    }
                }

                fn is_final(&self) -> bool {
                    self.state < Self::FIRST_NON_FINAL_STATE
                }

                fn is_invalid(&self) -> bool {
                    self.state == Self::INVALID_STATE
                }

                fn next(&mut self, byte: u8) {
                    debug_assert!(
                        self.state < Self::STATES_NUM,
                        "transition from invalid state {0} is not allowed",
                        self.state,
                    );
                    self.state = *unsafe {
                        Self::TRANSITION_TABLE
                            .get_unchecked(self.state)
                            .get_unchecked(byte as usize)
                    } as usize;
                }
            }

            #[derive(Debug, PartialEq, Eq)]
            pub struct Match<'h> {
                capture: &'h str,
                start: usize,
            }

            impl<'h> Match<'h> {
                pub fn start(&self) -> usize {
                    self.start
                }

                pub fn end(&self) -> usize {
                    self.start + self.capture.len()
                }

                pub fn len(&self) -> usize {
                    self.capture.len()
                }

                pub fn is_empty(&self) -> bool {
                    self.capture.is_empty()
                }

                pub fn range(&self) -> ::core::ops::Range<usize> {
                    self.start..self.end()
                }

                pub fn as_str(&self) -> &'h str {
                    self.capture
                }

                pub fn as_bytes(&self) -> &'h [u8] {
                    self.as_str().as_bytes()
                }
            }

            #[derive(Debug)]
            pub struct Regex;

            impl Regex {
                pub fn new() -> Self {
                    Self
                }

                pub fn match_at<'h>(&self, haystack: &'h str, start: usize) -> Option<Match<'h>> {
                    let mut state_machine = StateMachine::new();
                    let mut final_index = None;
                    if state_machine.is_final() {
                        final_index = Some(0);
                    }
                    for (i, byte) in haystack.as_bytes()[start..].iter().enumerate() {
                        dbg!(&state_machine);
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
        adhoc::Regex::new()
    };

    let m = re.match_at("0421", 1);
    assert!(m.is_some());
    let m = m.unwrap();
    assert_eq!(m.as_str(), "42");
    assert_eq!(m.as_bytes(), b"42");
    assert_eq!(m.start(), 1);
    assert_eq!(m.end(), 3);
    assert_eq!(m.len(), 2);
    assert_eq!(m.range(), 1..3);
    assert_eq!(m.is_empty(), false);
}
