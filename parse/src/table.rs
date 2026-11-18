use core::mem::MaybeUninit;

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

pub const FINISH: usize = 0;
pub const FAIL: usize = 1;
pub const UNCALL: usize = 2; // A state that only uncalls
pub struct ParseTable<Action: Default> {
    table: [Action; 65536]
}

impl<A: Default + Copy> ParseTable<A> {
    pub const fn new<Input: PartialEq>(start: u8, states: &'static [TransitionTable<u8, A>]) -> ParseTable<A> {
        let table = MaybeUninit::uninit();
        for (state, transition_table) in states.iter().enumerate() {
            for transition in transition_table.transitions {
                let index = state << 8 + transition.0;
                
            }
        }
        
        return ParseTable { table: table.assume_init() };
    }

    #[inline]
    pub fn get(&self, state: u8, input: u8) -> A {
        let entry: u16 = (state as u16) << 8 + input;
        unsafe { *self.table.get(entry as usize).unwrap_unchecked() }
    }
}