#![no_std]

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
    states: &'static [Transition]
}

impl TransitionTable {
    pub const START: usize = 0;
    pub const FINISH: usize = usize::MAX;
    
    pub fn new(states: &'static [Transition]) -> TransitionTable {
        TransitionTable { states }
    }

    pub fn get(&self, transition: usize) -> Option<&Transition> {
        self.states.get(transition)
    }

    pub unsafe fn get_unchecked(&self, transition: usize) -> &Transition {
        unsafe { self.states.get_unchecked(transition) }
    }
}