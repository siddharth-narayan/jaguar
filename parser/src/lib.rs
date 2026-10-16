#![no_std]

pub mod json;

#[derive(Copy, Clone)]
pub enum Action {
    Push,
    Reduce
}

#[derive(Copy, Clone)]
pub struct Transition {
    pub next_state: usize,
    pub action: Action
}

pub struct TransitionTable {
    transitions: &'static [(usize, Transition)]
}

impl TransitionTable {
    pub fn new(transitions: &'static [(usize, Transition)]) -> TransitionTable {
        TransitionTable { transitions }
    }

    pub fn get(&self, input: usize) -> Option<&Transition> {
        for (i, t) in self.transitions {
            if *i == input {
                return Some(t)
            }
        }
        
        None
    }
}