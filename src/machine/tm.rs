use miette::{Diagnostic, SourceSpan};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};
use thiserror::Error;

use crate::parser::{Direction as ParserDirection, PartialMachineInfo, StackTransition};

#[derive(Error, Diagnostic, Debug)]
pub enum InfoError {
    #[error("Unknown state")]
    UnknownState {
        #[label("here")]
        at: SourceSpan,
    },
    #[error("Unknown tape symbol")]
    UnknownTapeSymbol {
        #[label("here")]
        at: SourceSpan,
    },
    #[error("Missing section")]
    #[diagnostic(help("expected to find {}", section))]
    MissingSection { section: &'static str },
    #[error("Missing tape operation")]
    MissingTapeOperation,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TMTransitionFrom {
    pub initial: &'static str,
    pub with_symbol: &'static str,
}

#[derive(Debug, Clone)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct TMTransitionTo {
    pub state: &'static str,
    pub write_symbol: &'static str,
    pub direction: Direction,
}

#[derive(Clone)]
pub struct Info {
    pub alphabet: HashSet<&'static str>,
    pub transitions: HashMap<TMTransitionFrom, TMTransitionTo>,
    pub start_state: &'static str,
    pub final_states: HashSet<&'static str>,
    pub blank_symbol: &'static str,
}

impl Info {
    pub fn new(machine_info: PartialMachineInfo, src: &'static str) -> miette::Result<Self> {
        let states: HashSet<&'static str> = machine_info
            .states
            .iter()
            .map(|state| state.src(src))
            .collect();

        let alphabet: HashSet<&'static str> = machine_info
            .alphabet
            .iter()
            .map(|symbol| symbol.src(src))
            .collect();

        let tape_alphabet = match machine_info.tape_alphabet {
            Some(ref symbols) => symbols
                .iter()
                .map(|symbol| symbol.src(src))
                .collect::<HashSet<_>>(),
            None => {
                return Err(InfoError::MissingSection {
                    section: "tape_alphabet",
                }
                .into());
            }
        };

        // Blank symbol is required for Turing Machine
        let blank_symbol = match machine_info.blank_symbol {
            Some(ref token) => token.src(src),
            None => {
                return Err(InfoError::MissingSection {
                    section: "blank_symbol",
                }
                .into());
            }
        };

        let final_states: HashSet<&'static str> = machine_info
            .final_states
            .iter()
            .map(|state| state.src(src))
            .collect();

        // Validate that all final states are in the set of states
        for final_state in &final_states {
            if !states.contains(final_state) {
                for token in &machine_info.final_states {
                    if token.src(src) == *final_state {
                        return Err(InfoError::UnknownState { at: token.span() }.into());
                    }
                }
            }
        }

        let start_state = machine_info.start_state.src(src);
        if !states.contains(start_state) {
            return Err(InfoError::UnknownState {
                at: machine_info.start_state.span(),
            }
            .into());
        }

        let mut transitions = HashMap::new();

        for transition in machine_info.transitions {
            let from_state = transition.from.initial.src(src);
            if !states.contains(from_state) {
                return Err(InfoError::UnknownState {
                    at: transition.from.initial.span(),
                }
                .into());
            }

            let with_symbol = transition.from.with_symbol.src(src);
            if with_symbol != "ε" && !tape_alphabet.contains(with_symbol) {
                return Err(InfoError::UnknownTapeSymbol {
                    at: transition.from.with_symbol.span(),
                }
                .into());
            }

            let to_state = transition.to.0.src(src);
            if !states.contains(to_state) {
                return Err(InfoError::UnknownState {
                    at: transition.to.0.span(),
                }
                .into());
            }

            // Parse tape operation data
            let write_symbol = match &transition.to.1 {
                Some(action) => match action {
                    StackTransition::Write(_, symbol_token) => {
                        let symbol = symbol_token.src(src);
                        if !tape_alphabet.contains(symbol) {
                            return Err(InfoError::UnknownTapeSymbol {
                                at: symbol_token.span(),
                            }
                            .into());
                        }
                        symbol
                    }
                    _ => {
                        return Err(InfoError::MissingTapeOperation.into());
                    }
                },
                None => {
                    return Err(InfoError::MissingTapeOperation.into());
                }
            };

            // Parse direction
            let direction = match &transition.to.2 {
                Some(dir_action) => match dir_action {
                    ParserDirection::Left(_) => Direction::Left,
                    ParserDirection::Right(_) => Direction::Right,
                },
                None => {
                    return Err(InfoError::MissingSection {
                        section: "direction",
                    }
                    .into());
                }
            };

            transitions.insert(
                TMTransitionFrom {
                    initial: from_state,
                    with_symbol,
                },
                TMTransitionTo {
                    state: to_state,
                    write_symbol,
                    direction,
                },
            );
        }

        Ok(Self {
            alphabet,
            transitions,
            start_state,
            final_states,
            blank_symbol,
        })
    }
}

struct Tape {
    tape: Vec<&'static str>,
    position: usize,
    blank_symbol: &'static str,
}

impl Tape {
    fn new(blank_symbol: &'static str) -> Self {
        Self {
            tape: vec![blank_symbol],
            position: 0,
            blank_symbol,
        }
    }

    fn get_current_symbol(&self) -> &'static str {
        if self.position < self.tape.len() {
            self.tape[self.position]
        } else {
            self.blank_symbol
        }
    }

    fn write_symbol(&mut self, symbol: &'static str) {
        if self.position >= self.tape.len() {
            self.tape.resize(self.position + 1, self.blank_symbol);
        }
        self.tape[self.position] = symbol;
    }

    fn move_left(&mut self) {
        if self.position > 0 {
            self.position -= 1;
        } else {
            // Add a blank symbol at the beginning of the tape
            self.tape.insert(0, self.blank_symbol);
        }
    }

    fn move_right(&mut self) {
        self.position += 1;
        if self.position >= self.tape.len() {
            self.tape.push(self.blank_symbol);
        }
    }
}

impl fmt::Debug for Tape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tape [")?;
        for (index, symbol) in self.tape.iter().enumerate() {
            if index == self.position {
                write!(f, "[{}]", symbol)?;
            } else {
                write!(f, " {} ", symbol)?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

pub struct Machine {
    info: Info,
}

impl Machine {
    pub fn new(info: Info) -> Self {
        Machine { info }
    }

    pub fn run(&self, input: &str) -> bool {
        let input_symbols: Vec<&'static str> = input
            .chars()
            .map(|c| {
                let s = String::from(c).leak();
                if !self.info.alphabet.contains(s) && s != "ε" {
                    println!("Warning: Symbol {} i§s not in the alphabet", s);
                }
                &*s
            })
            .collect();

        let mut tape = Tape::new(self.info.blank_symbol);

        // Initialize tape with input
        for symbol in input_symbols {
            tape.write_symbol(symbol);
            tape.move_right();
        }

        // Reset position to beginning
        while tape.position > 0 {
            tape.move_left();
        }

        let mut current_state = self.info.start_state;
        println!("Starting simulation with state: {}", current_state);

        // Run until we reach a final state or get stuck
        loop {
            let current_symbol = tape.get_current_symbol();
            let transition_key = TMTransitionFrom {
                initial: current_state,
                with_symbol: current_symbol,
            };

            if let Some(transition) = self.info.transitions.get(&transition_key) {
                println!(
                    "Transition: ({}, {}) -> ({}, {}, {:?})",
                    current_state,
                    current_symbol,
                    transition.state,
                    transition.write_symbol,
                    transition.direction
                );

                // Update the tape
                tape.write_symbol(transition.write_symbol);

                // Move the tape head
                match transition.direction {
                    Direction::Left => tape.move_left(),
                    Direction::Right => tape.move_right(),
                }

                // Update current state
                current_state = transition.state;
                println!(
                    "Current state: {}, {}",
                    current_state,
                    tape.get_current_symbol()
                );
                println!("{:?}", tape);

                // Check if we've reached a final state
                if self.info.final_states.contains(current_state) {
                    println!("Reached final state: {}", current_state);
                    return true;
                }
            } else {
                println!(
                    "No valid transition from state {} with symbol {}",
                    current_state, current_symbol
                );
                break;
            }
        }

        false
    }
}
