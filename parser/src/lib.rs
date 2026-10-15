#![no_std]

enum Action {
    Push,
    Reduce
}

pub struct Transition {
    next_state: usize,
    action: Action
}

pub struct TransitionTable {
    states: &'static [Transition]
}

impl TransitionTable {
    pub const START: usize = 0;
    pub const FINISH: usize = usize::MAX;
    
    pub fn new(states: &'static [Transition]) -> TransitionTable {
        TransitionTable { states }
    }
}