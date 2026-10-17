#![no_std]

pub mod json;

#[derive(Copy, Clone)]
pub enum Action {
    None,
    Push,
    Reduce
}

#[derive(Copy, Clone)]
pub struct Transition {
    pub next_state: usize,
    pub action: Action
}

impl Transition {
    pub const fn new(next: usize, action: Action) -> Transition {
        Transition { next_state: next, action }
    }
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
    pub const FAIL: usize = usize::MAX - 1;
    pub const FINISH: usize = usize::MAX;

    pub const fn new(states: &'static [TransitionTable]) -> ParseTable {
        ParseTable {
            states
        }
    }
}