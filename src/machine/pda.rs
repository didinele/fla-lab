use std::collections::{HashMap, HashSet, VecDeque};

use miette::Diagnostic;
use thiserror::Error;

use crate::parser::{ParserError, PartialMachineInfo, StackTransition};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PDATransitionFrom {
    initial: &'static str,
    with_symbol: &'static str,
    stack_top: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct PDATransitionTo {
    state: &'static str,
    stack_action: StackAction,
}

#[derive(Debug, Clone)]
pub enum StackAction {
    Push(&'static str),
    Pop,
    NoOp,
}

#[derive(Error, Diagnostic, Debug)]
pub enum PDAError {
    #[error(
        "PDA must have a stack alphabet and stack information in each transition"
    )]
    StackOperationsRequired,
}

#[derive(Debug, Clone)]
pub struct Info {
    // We actually never need the full states hashset
    // pub states: HashSet<&'static str>,
    pub alphabet: HashSet<&'static str>,
    pub stack_alphabet: HashSet<&'static str>,
    pub transitions: HashMap<PDATransitionFrom, Vec<PDATransitionTo>>,
    pub start_state: &'static str,
    pub final_states: HashSet<&'static str>,
    pub start_stack_symbol: Option<&'static str>,
}

impl Info {
    pub fn new(machine: PartialMachineInfo, src: &'static str) -> miette::Result<Self> {
        let mut states = HashSet::new();
        let mut alphabet = HashSet::new();
        let mut stack_alphabet = HashSet::new();
        let mut transitions = HashMap::new();
        let mut final_states = HashSet::new();
        let mut start_stack_symbol = None;

        for state in &machine.states {
            states.insert(state.src(src));
        }

        for symbol in &machine.alphabet {
            alphabet.insert(symbol.src(src));
        }

        alphabet.insert("ε");

        if let Some(stack_symbs) = &machine.stack_alphabet {
            for symbol in stack_symbs {
                stack_alphabet.insert(symbol.src(src));
            }
        } else {
            return Err(PDAError::StackOperationsRequired.into());
        }

        if let Some(start_stack) = &machine.start_stack {
            let symbol = start_stack.src(src);
            if !stack_alphabet.is_empty() && !stack_alphabet.contains(symbol) {
                return Err(ParserError::UnknownAlphabetSymbol {
                    at: start_stack.span(),
                }
                .into());
            }

            start_stack_symbol = Some(symbol);
        }

        for final_state in &machine.final_states {
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

        // Process transitions
        for transition in &machine.transitions {
            // Keep in mind transition.from.with_stack_symbol can be None, that is valid
            if transition.to.1.is_none() {
                return Err(PDAError::StackOperationsRequired.into());
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

            if !alphabet.contains(symbol) && symbol != "ε" {
                return Err(ParserError::UnknownAlphabetSymbol {
                    at: transition.from.with_symbol.span(),
                }
                .into());
            }

            // Handle stack symbols in the transition
            let stack_top = if let Some(stack_symbol) = &transition.from.with_stack_symbol {
                let stack_sym_str = stack_symbol.src(src);
                if !stack_alphabet.is_empty() && !stack_alphabet.contains(stack_sym_str) {
                    return Err(ParserError::UnknownAlphabetSymbol {
                        at: stack_symbol.span(),
                    }
                    .into());
                }

                Some(stack_sym_str)
            } else {
                // No specific stack symbol required
                None
            };

            // Extract stack action from transition
            let stack_action = match &transition.to.1 {
                Some(stack_trans) => match stack_trans {
                    StackTransition::Push(_, symbol) => {
                        let symbol_str = symbol.src(src);
                        if !stack_alphabet.is_empty() && !stack_alphabet.contains(symbol_str) {
                            return Err(
                                ParserError::UnknownAlphabetSymbol { at: symbol.span() }.into()
                            );
                        }
                        StackAction::Push(symbol_str)
                    }
                    StackTransition::Pop(_) => StackAction::Pop,
                    StackTransition::NoOp(_) => StackAction::NoOp,
                },
                None => unreachable!(
                    "Stack action should not be None, this should be handled above. This is a bug"
                ),
            };

            let key = PDATransitionFrom {
                initial: from_state,
                with_symbol: symbol,
                stack_top,
            };

            let value = PDATransitionTo {
                state: to_state,
                stack_action,
            };

            transitions.entry(key).or_insert_with(Vec::new).push(value);
        }

        Ok(Info {
            alphabet,
            stack_alphabet,
            transitions,
            start_state,
            final_states,
            start_stack_symbol,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Machine {
    info: Info,
    current_state: &'static str,
    stack: VecDeque<&'static str>,
}

impl Machine {
    pub fn new(info: Info) -> Self {
        let mut stack = VecDeque::new();

        if let Some(start_symbol) = info.start_stack_symbol {
            stack.push_back(start_symbol);
        }

        Self {
            current_state: info.start_state,
            info,
            stack,
        }
    }

    pub fn run(mut self, input: &str) -> bool {
        // Process each character in the input using a deterministic approach
        // (taking the first valid transition)
        let mut input_chars: Vec<char> = input.chars().collect();
        let mut position = 0;
        
        // First, try to process any epsilon transitions before consuming input
        while self.make_transition("ε", self.stack.back().copied()) {
            // Continue making epsilon transitions until none are left
        }

        // Process until we've consumed all input and can't make any more epsilon transitions
        while position < input_chars.len() || self.can_make_epsilon_transition() {
            let current_symbol = if position < input_chars.len() {
                self.find_symbol(input_chars[position])
            } else {
                // If we've consumed all input but still have states to process,
                // try epsilon transitions
                Some("ε")
            };

            if let Some(symbol) = current_symbol {
                // Get the stack top if available
                let stack_top = self.stack.back().copied();
                
                // Always try epsilon transitions first
                let mut made_transition = false;
                if symbol != "ε" {
                    while self.make_transition("ε", self.stack.back().copied()) {
                        made_transition = true;
                    }
                }

                // Then try to make a transition with the current input symbol
                if self.make_transition(symbol, self.stack.back().copied()) {
                    made_transition = true;
                    if position < input_chars.len() && symbol != "ε" {
                        // Move to next input character
                        position += 1;
                    }
                }
                
                if !made_transition {
                    // No valid transition found for the current state, symbol, and stack
                    if symbol == "ε" && position >= input_chars.len() {
                        // If we can't make any more epsilon transitions after consuming all input,
                        // we need to determine if we're in an accepting state
                        break;
                    } else {
                        println!(
                            "No valid transition found for state '{}', symbol '{}', stack top '{:?}'. Rejecting input.",
                            self.current_state, symbol, stack_top
                        );
                        return false;
                    }
                }
            } else {
                // Symbol not in alphabet
                println!(
                    "Symbol '{}' not in alphabet. Rejecting input.",
                    input_chars[position]
                );
                return false;
            }
        }

        // Accept if we're in a final state
        let is_accepted = self.info.final_states.contains(self.current_state);
        is_accepted
    }

    fn find_symbol(&self, c: char) -> Option<&'static str> {
        self.info
            .alphabet
            .iter()
            .find(|&&s| s.len() == 1 && s != "ε" && s.chars().next().unwrap() == c)
            .copied()
    }

    fn can_make_epsilon_transition(&self) -> bool {
        let stack_top = self.stack.back().copied();

        for (key, _) in &self.info.transitions {
            if key.initial == self.current_state && key.with_symbol == "ε" {
                if key.stack_top.is_none() || key.stack_top == stack_top {
                    return true;
                }
            }
        }

        false
    }

    fn make_transition(&mut self, symbol: &'static str, stack_top: Option<&'static str>) -> bool {
        let mut matching_transitions = Vec::new();

        // First, try to find transitions that match the specific stack top
        if let Some(top) = stack_top {
            for (key, transitions) in &self.info.transitions {
                if key.initial == self.current_state
                    && key.with_symbol == symbol
                    && key.stack_top == Some(top)
                {
                    matching_transitions.extend(transitions.iter().cloned());
                }
            }
        }

        // If no specific match, look for transitions with no stack top requirement
        if matching_transitions.is_empty() {
            for (key, transitions) in &self.info.transitions {
                if key.initial == self.current_state
                    && key.with_symbol == symbol
                    && key.stack_top.is_none()
                {
                    matching_transitions.extend(transitions.iter().cloned());
                }
            }
        }

        // If no transitions matched, return false
        if matching_transitions.is_empty() {
            return false;
        }

        // Use the first matching transition
        // In a full non-deterministic PDA implementation, we would explore all possible paths
        let transition = &matching_transitions[0];

        // Update stack based on the stack action
        match transition.stack_action {
            StackAction::Push(symbol) => {
                self.stack.push_back(symbol);
            }
            StackAction::Pop => {
                if !self.stack.is_empty() {
                    self.stack.pop_back();
                }
            }
            StackAction::NoOp => {
                // Do nothing
            }
        }

        // Update current state
        self.current_state = transition.state;
        true
    }
}
