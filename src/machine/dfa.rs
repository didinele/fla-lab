use miette::Diagnostic;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::parser::{ParserError, PartialMachineInfo};

#[derive(Error, Diagnostic, Debug)]
pub enum DFAError {
    #[error("Cannot have multiple transitions from state '{initial}' with symbol '{with_symbol}'")]
    MultipleTransitions {
        initial: &'static str,
        with_symbol: &'static str,
    },
    #[error(
        "DFA is incomplete: no transition defined for state '{initial}' with symbol '{with_symbol}'"
    )]
    IncompleteDFA {
        initial: &'static str,
        with_symbol: &'static str,
    },
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TransitionFrom {
    initial: &'static str,
    with_symbol: &'static str,
}

#[derive(Debug)]
pub struct TransitionTo(&'static str);

#[derive(Debug)]
pub struct Info {
    // We actually never need the full states hashset
    // pub states: HashSet<&'static str>,
    pub alphabet: HashSet<&'static str>,
    pub transitions: HashMap<TransitionFrom, TransitionTo>,
    pub start_state: &'static str,
    pub final_states: HashSet<&'static str>,
}

impl Info {
    pub fn new(machine: PartialMachineInfo, src: &'static str) -> miette::Result<Self> {
        let mut states = HashSet::new();
        let mut alphabet = HashSet::new();
        let mut transitions = HashMap::new();
        let mut final_states = HashSet::new();

        for state in machine.states {
            states.insert(state.src(src));
        }

        for symbol in machine.alphabet {
            alphabet.insert(symbol.src(src));
        }

        for final_state in machine.final_states {
            let state_str = final_state.src(src);

            if !states.contains(state_str) {
                return Err(ParserError::UnknownState {
                    at: final_state.span(),
                }
                .into());
            }

            final_states.insert(state_str);
        }

        let start_state_str = machine.start_state.src(src);
        if !states.contains(start_state_str) {
            return Err(ParserError::UnknownState {
                at: machine.start_state.span(),
            }
            .into());
        }

        for transition in machine.transitions {
            let from_state = transition.from.src(src);
            let symbol = transition.with.src(src);
            let to_state = transition.to.src(src);

            // Validate transition states and symbols
            if !states.contains(from_state) {
                return Err(ParserError::UnknownState {
                    at: transition.from.span(),
                }
                .into());
            }

            if !states.contains(to_state) {
                return Err(ParserError::UnknownState {
                    at: transition.to.span(),
                }
                .into());
            }

            if !alphabet.contains(symbol) {
                return Err(ParserError::UnknownAlphabetSymbol {
                    at: transition.with.span(),
                }
                .into());
            }

            let key = TransitionFrom {
                initial: from_state,
                with_symbol: symbol,
            };

            // Check if there's already a transition for this state and symbol (violates DFA property)
            if transitions.contains_key(&key) {
                return Err(DFAError::MultipleTransitions {
                    initial: from_state,
                    with_symbol: symbol,
                }
                .into());
            }

            transitions.insert(key, TransitionTo(to_state));
        }

        // Check if the DFA is complete (each state has a transition for each symbol in the alphabet)
        for state in &states {
            for symbol in &alphabet {
                let key = TransitionFrom {
                    initial: state,
                    with_symbol: symbol,
                };

                if !transitions.contains_key(&key) {
                    return Err(DFAError::IncompleteDFA {
                        initial: state,
                        with_symbol: symbol,
                    }
                    .into());
                }
            }
        }

        Ok(Info {
            alphabet,
            transitions,
            start_state: start_state_str,
            final_states,
        })
    }
}

#[derive(Debug)]
pub struct Machine {
    info: Info,
    current_state: &'static str,
}

impl Machine {
    pub fn new(info: Info) -> Self {
        Self {
            current_state: info.start_state,
            info,
        }
    }

    pub fn run(mut self, input: &str) -> bool {
        // Process each character in the input
        for c in input.chars() {
            // Find the matching symbol in the alphabet
            let symbol = self
                .info
                .alphabet
                .iter()
                .find(|&&s| s.len() == 1 && s.chars().next().unwrap() == c);

            if let Some(&symbol) = symbol {
                let key = TransitionFrom {
                    initial: self.current_state,
                    with_symbol: symbol,
                };

                if let Some(transition) = self.info.transitions.get(&key) {
                    self.current_state = transition.0;
                } else {
                    println!(
                        "No transition found for state '{}' with symbol '{}'. Counting as not accepted",
                        self.current_state, symbol
                    );
                    return false;
                }
            } else {
                // Symbol not in alphabet
                println!("Symbol '{}' not in alphabet. Counting as not accepted", c);
                return false;
            }
        }

        self.info.final_states.contains(self.current_state)
    }
}
