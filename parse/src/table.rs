#![no_std]

extern crate alloc;

use crate::json::{JsonFragmentType, Token::{self, *}};

#[derive(Copy, Clone)]
pub enum Transition {
    None { next: usize },
    Push { next: usize },
    // Uncall is for different return "addresses"
    Call { call_state: usize, return_state: usize },
    Uncall
}

pub struct TransitionTable {
    transitions: &'static [(Token, Transition)],
    unmatched: Transition
}

impl TransitionTable {
    pub const fn new(transitions: &'static [(Token, Transition)]) -> TransitionTable {
        TransitionTable { transitions, unmatched: Transition::None { next: ParseTable::FAIL } }
    }

    pub const fn new_ex(transitions: &'static [(Token, Transition)], unmatched: Transition) -> TransitionTable {
        TransitionTable { transitions, unmatched}
    }

    pub fn get(&self, token: Token) -> &Transition {
        for (i, t) in self.transitions {
            if *i == token {
                return t
            }
        }
        
        &self.unmatched
    }
}

pub struct ParseTable {
    states: &'static [TransitionTable]
}

impl ParseTable {
    pub const FINISH: usize = 0;
    pub const FAIL: usize = 1;
    pub const UNCALL: usize = 2; // A state that only uncalls

    pub const fn new(start: usize, states: &'static [TransitionTable]) -> ParseTable {
        ParseTable {
            states
        }
    }

    pub fn get(&self, state: usize) -> Option<&TransitionTable> {
        self.states.get(state)
    }

    pub unsafe fn get_unchecked(&self, state: usize) -> &TransitionTable {
        unsafe { self.states.get_unchecked(state) }
    }
}