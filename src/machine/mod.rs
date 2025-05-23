pub mod dfa;
pub mod nfa;
pub mod pda;
pub mod tm;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TransitionFrom {
    pub initial: &'static str,
    pub with_symbol: &'static str,
}

#[derive(Debug, Clone)]
pub struct TransitionTo(pub &'static str);
