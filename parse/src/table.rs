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
pub struct ParseTable<I: PartialEq + 'static, T: 'static> {
    states: &'static [TransitionTable<I, T>]
}

impl<I: PartialEq, T> ParseTable<I, T> {
    pub const fn new(start: usize, states: &'static [TransitionTable<I, T>]) -> ParseTable<I, T> {
        ParseTable {
            states
        }
    }

    pub fn get(&self, state: usize) -> Option<&TransitionTable<I, T>> {
        self.states.get(state)
    }

    pub unsafe fn get_unchecked(&self, state: usize) -> &TransitionTable<I, T> {
        unsafe { self.states.get_unchecked(state) }
    }
}