use core::mem::MaybeUninit;
use alloc::fmt::Debug;

/// This struct filled with 0 bytes must be valid
pub unsafe trait ZeroedDefault {}

pub struct TransitionTable<I: 'static + PartialEq, T: 'static> {
    transitions: &'static [(I, T)],
    unmatched: T
}

impl<I: PartialEq, T> TransitionTable<I, T> {
    pub const fn new(transitions: &'static [(I, T)], unmatched: T) -> TransitionTable<I, T> {
        TransitionTable { transitions, unmatched}
    }

    pub fn get(&self, input: I) -> &T {
        for (i, t) in self.transitions {
            if *i == input {
                return t
            }
        }
        
        &self.unmatched
    }
}

pub const FINISH: u8 = 0;
pub const FAIL: u8 = 1;
pub const UNCALL: u8 = 2; // A state that only uncalls
pub struct ParseTable<Action> {
    table: [Action; 65536]
}

impl<A: ZeroedDefault + Copy + Debug> ParseTable<A> {
    pub const fn new(states: &'static [TransitionTable<u8, A>]) -> ParseTable<A> {
        let mut table: [A; 65536] = unsafe { MaybeUninit::zeroed().assume_init() };

        for state in 0..states.len() {
            let transition_table = &states[state];
            for transition_idx in 0..transition_table.transitions.len() {
                let (input, action) = transition_table.transitions.get(transition_idx).unwrap();
                for input in 0..u8::MAX {
                    table[Self::index(state as u8, input) as usize] = transition_table.unmatched
                }

                table[Self::index(state as u8, *input) as usize] = *action
            }
        }
        
        return ParseTable { table }
    }

    #[inline]
    const fn index(state: u8, input: u8) -> u16 {
        ((state as u16) << 8) + input as u16
    }

    #[inline]
    pub fn get(&self, state: u8, input: u8) -> A {
        let entry = Self::index(state, input);
        let result = unsafe { *self.table.get(entry as usize).unwrap_unchecked() };

        println!("state: {}, input: {}, action: {:#?}", state, input, result);
        result
    }
}