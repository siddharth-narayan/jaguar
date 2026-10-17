#![no_std]

pub mod json;

#[derive(Copy, Clone)]
pub enum Transition {
    None { next: usize },
    Push { next: usize },
    Call { call: usize },
    Uncall
}

pub struct TransitionTable {
    transitions: &'static [(usize, Transition)]
}

impl TransitionTable {
    pub const fn new(transitions: &'static [(usize, Transition)]) -> TransitionTable {
        TransitionTable { transitions }
    }

    pub fn get(&self, token: usize) -> Option<&Transition> {
        for (i, t) in self.transitions {
            if *i == token {
                return Some(t)
            }
        }
        
        None
    }
}

pub struct ParseTable {
    states: &'static [TransitionTable]
}

impl ParseTable {
    pub const START: usize = 0;
    pub const FINISH: usize = 1;
    pub const FAIL: usize = 2;
    pub const UNCALL: usize = 3; // A state that only uncalls

    pub const fn new(states: &'static [TransitionTable]) -> ParseTable {
        ParseTable {
            states
        }
    }

    pub fn get(&self, state: usize) -> Option<&TransitionTable> {
        self.states.get(state)
    }
}