// #![no_std]

extern crate alloc;

use crate::json::{JsonFragmentType, Token::{self, *}};

pub mod json;

#[derive(Copy, Clone)]
pub enum Transition {
    None { next: usize },
    Push { next: usize },
    // Uncall is for different return "addresses"
    Call { call: usize, uncall: usize },
    Uncall
}

pub struct TransitionTable {
    transitions: &'static [(Token, Transition)]
}

impl TransitionTable {
    pub const fn new(transitions: &'static [(Token, Transition)]) -> TransitionTable {
        TransitionTable { transitions }
    }

    pub const fn special_case(special: (Token, Transition), default: Transition) -> [(Token, Transition); 13] {
        [
            special,
            (STRING, default),
            (NUMBER, default),
            (TRUE, default),
            (FALSE, default),
            (NULL, default),
            (COLON, default),
            (COMMA, default),
            (LBRACE, default),
            (RBRACE, default),
            (LBRACK, default),
            (RBRACK, default),
            (EOF, default)        
        ]
    }

    pub const fn new_all(transition: Transition) -> [(Token, Transition); 12] {
        [
            (STRING, transition),
            (NUMBER, transition),
            (TRUE, transition),
            (FALSE, transition),
            (NULL, transition),
            (COLON, transition),
            (COMMA, transition),
            (LBRACE, transition),
            (RBRACE, transition),
            (LBRACK, transition),
            (RBRACK, transition),
            (EOF, transition)        
        ]
    }

    pub fn get(&self, token: Token) -> &Transition {
        for (i, t) in self.transitions {
            if *i == token {
                return t
            }
        }
        
        println!("Failed to find transition {:#?} for table", token);

        &Transition::None { next: ParseTable::FAIL }
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