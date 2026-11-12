#![no_std]

extern crate alloc;

// use crate::json::{JsonFragmentType, Token::{self, *}};



pub struct TransitionTable<T: 'static> {
    transitions: &'static [(usize, T)],
    unmatched: T
}

impl<T> TransitionTable<T> {
    pub const fn new(transitions: &'static [(usize, T)], unmatched: T) -> TransitionTable<T> {
        TransitionTable { transitions, unmatched}
    }

    pub fn get(&self, input: usize) -> &T {
        for (i, t) in self.transitions {
            if *i == input {
                return t
            }
        }
        
        &self.unmatched
    }
}

pub const FINISH: usize = 0;
pub const FAIL: usize = 1;
pub const UNCALL: usize = 2; // A state that only uncalls
pub struct ParseTable<T: 'static> {
    states: &'static [TransitionTable<T>]
}

impl<T> ParseTable<T> {
    pub const fn new(start: usize, states: &'static [TransitionTable<T>]) -> ParseTable<T> {
        ParseTable {
            states
        }
    }

    pub fn get(&self, state: usize) -> Option<&TransitionTable<T>> {
        self.states.get(state)
    }

    pub unsafe fn get_unchecked(&self, state: usize) -> &TransitionTable<T> {
        unsafe { self.states.get_unchecked(state) }
    }
}