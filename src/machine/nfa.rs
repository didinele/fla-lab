use std::collections::{HashMap, HashSet};

use miette::Diagnostic;
use thiserror::Error;

use crate::parser::{ParserError, PartialMachineInfo, StackTransition};

use super::{TransitionFrom, TransitionTo};

#[derive(Error, Diagnostic, Debug)]
pub enum NFAError {
    #[error("NFA cannot have stack operations")]
    StackOperationsNotAllowed,
    #[error("NFA cannot have tape operations")]
    TapeOperationsNotAllowed,
}

#[derive(Debug, Clone)]
pub struct Info {
    // We actually never need the full states hashset
    // pub states: HashSet<&'static str>,
    pub alphabet: HashSet<&'static str>,
    pub transitions: HashMap<TransitionFrom, Vec<TransitionTo>>,
    pub start_state: &'static str,
    pub final_states: HashSet<&'static str>,
}

impl Info {
    pub fn new(machine: PartialMachineInfo, src: &'static str) -> miette::Result<Self> {
        if machine.stack_alphabet.is_some() || machine.start_stack.is_some() {
            return Err(NFAError::StackOperationsNotAllowed.into());
        }

        if machine.tape_alphabet.is_some() || machine.blank_symbol.is_some() {
            return Err(NFAError::TapeOperationsNotAllowed.into());
        }

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

        alphabet.insert("ε");

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

        let start_state = machine.start_state.src(src);
        if !states.contains(start_state) {
            return Err(ParserError::UnknownState {
                at: machine.start_state.span(),
            }
            .into());
        }

        for transition in machine.transitions {
            if transition.from.with_stack_symbol.is_some() || transition.to.1.is_some() {
                return Err(NFAError::StackOperationsNotAllowed.into());
            }

            // Check for any stack transition operations
            if let Some(ref stack_trans) = transition.to.1 {
                match stack_trans {
                    StackTransition::Push(_, _)
                    | StackTransition::Pop(_)
                    | StackTransition::NoOp(_) => {
                        return Err(NFAError::StackOperationsNotAllowed.into());
                    }
                    StackTransition::Write(_, _) => {
                        return Err(NFAError::TapeOperationsNotAllowed.into());
                    }
                }
            }

            // Check for tape operations
            if transition.to.2.is_some() {
                return Err(NFAError::TapeOperationsNotAllowed.into());
            }

            let from_state = transition.from.initial.src(src);
            let symbol = transition.from.with_symbol.src(src);
            let to_state = transition.to.0.src(src);

            // Validate transition states and symbols
            if !states.contains(from_state) {
                return Err(ParserError::UnknownState {
                    at: transition.from.initial.span(),
                }
                .into());
            }

            if !states.contains(to_state) {
                return Err(ParserError::UnknownState {
                    at: transition.to.0.span(),
                }
                .into());
            }

            if !alphabet.contains(symbol) {
                return Err(ParserError::UnknownAlphabetSymbol {
                    at: transition.from.with_symbol.span(),
                }
                .into());
            }

            let key = TransitionFrom {
                initial: from_state,
                with_symbol: symbol,
            };

            transitions
                .entry(key)
                .or_insert_with(Vec::new)
                .push(TransitionTo(to_state));
        }

        Ok(Info {
            alphabet,
            transitions,
            start_state,
            final_states,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Machine {
    info: Info,
    current_states: HashSet<&'static str>,
}

impl Machine {
    pub fn new(info: Info) -> Self {
        // Start with the initial state and its epsilon closure
        let initial_states = HashSet::from([info.start_state]);
        let current_states = Self::compute_epsilon_closure(&info, initial_states);

        Self {
            info,
            current_states,
        }
    }

    pub fn run(mut self, input: &str) -> bool {
        // Process each character in the input
        for c in input.chars() {
            let symbol = self.find_symbol(c);

            if let Some(symbol) = symbol {
                self.current_states = self.get_next_states(symbol);
                if self.current_states.is_empty() {
                    println!(
                        "No valid transitions found for symbol '{}'. Counting as not accepted",
                        c
                    );

                    return false;
                }
            } else {
                // Symbol not in alphabet
                println!("Symbol '{}' not in alphabet. Counting as not accepted", c);
                return false;
            }
        }

        // check if any of the current states are final states
        self.current_states
            .iter()
            .any(|state| self.info.final_states.contains(state))
    }

    fn find_symbol(&self, c: char) -> Option<&'static str> {
        // Find the symbol in the alphabet that matches the character
        self.info
            .alphabet
            .iter()
            .find(|&&s| s.len() == 1 && s != "ε" && s.chars().next().unwrap() == c)
            .copied()
    }

    fn get_next_states(&self, symbol: &'static str) -> HashSet<&'static str> {
        let mut next_states = HashSet::new();

        // For each current state, find transitions with the given symbol
        for &current_state in &self.current_states {
            let key = TransitionFrom {
                initial: current_state,
                with_symbol: symbol,
            };

            if let Some(transitions) = self.info.transitions.get(&key) {
                for transition in transitions {
                    next_states.insert(transition.0);
                }
            }
        }

        Self::compute_epsilon_closure(&self.info, next_states)
    }

    fn compute_epsilon_closure(
        info: &Info,
        states: HashSet<&'static str>,
    ) -> HashSet<&'static str> {
        let mut closure = states.clone();
        let mut stack = states.into_iter().collect::<Vec<&'static str>>();

        while let Some(state) = stack.pop() {
            let key = TransitionFrom {
                initial: state,
                with_symbol: "ε",
            };

            if let Some(transitions) = info.transitions.get(&key) {
                for transition in transitions {
                    let target_state = transition.0;
                    if !closure.contains(target_state) {
                        closure.insert(target_state);
                        stack.push(target_state);
                    }
                }
            }
        }

        closure
    }
}
