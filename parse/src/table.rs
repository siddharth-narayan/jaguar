use core::mem::MaybeUninit;
use alloc::fmt::Debug;

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

impl<A: Default + Copy + Debug> ParseTable<A> {
    pub fn new(states: &'static [TransitionTable<u8, A>]) -> ParseTable<A> {
        let mut table: [A; 65536] = unsafe { MaybeUninit::uninit().assume_init() };
        for element in &mut table {
            *element = A::default();
        }

        for (state, transition_table) in states.iter().enumerate() {
            for (input, action) in transition_table.transitions {
                for input in 0..u8::MAX {
                    table[Self::index(state as u8, input) as usize] = transition_table.unmatched
                }

                table[Self::index(state as u8, *input) as usize] = *action
            }
        }
        
        return ParseTable { table }
    }

    #[inline]
    fn index(state: u8, input: u8) -> u16 {
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