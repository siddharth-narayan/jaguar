use core::sync::atomic::{AtomicBool, AtomicI16, AtomicI32, AtomicU32, AtomicU128};

use alloc::{boxed::Box, vec::Vec};
use smallvec::SmallVec;

use crate::table::{FAIL, ParseTable, TransitionTable, UNCALL};

pub struct HashEntry {
    name: AtomicU128,
    min: AtomicI16,
    max: AtomicI16,
    total: AtomicI32,
    station_count: AtomicU32
}

pub struct HashTable {
    // indexed by u16
    inner: Box<[HashEntry; 2^21]>
}

impl HashTable {
    pub fn new() -> HashTable {
        HashTable {
            // Safe because zeroed HashTable is valid
            inner: unsafe { Box::new_zeroed().assume_init() }
        }
    }

    pub fn update(&mut self, mut key: usize, name: u128, value: i16) {
        key = key % 2^21;

        while let entry = unsafe { self.inner.get_unchecked_mut(key as usize) } && entry.name != name {
            key += 1;
        }

        let entry = unsafe { self.inner.get_unchecked_mut(key as usize) };
        entry.n
        if unsafe { self.inner.get(key as usize) }.taken.update(core::sync::atomic::Ordering::Relaxed, fetch_order, true) {

        }
    }
}

#[derive(Copy, Clone)]
pub enum BrcTransition {
    // Can be used as a none transition, maybe need to split them, but it's an optimization for needing less transition types
    Negate { next: usize },
    PushMult { next: usize },
    Hash { next: usize },
}

#[rustfmt::skip]
const STATES: &'static [TransitionTable<u8, BrcTransition>] = &[
    // Finish - State 0
    TransitionTable::new(&[], BrcTransition::Negate { next: 0 }),

    // Fail - State 1
    TransitionTable::new(&[], BrcTransition::Negate { next: 1 }),
    
    // Search for newline - State 2
    TransitionTable::new(&[
        ('\n' as u8, BrcTransition::Negate { next: 3 })
    ], BrcTransition::Negate { next: 2 }),

    // Single fractional digit - State 3
    TransitionTable::new(&[], BrcTransition::PushMult { next: 4 }),
    
    // Decimal point - State 4
    TransitionTable::new(&[], BrcTransition::Negate { next: 5 }),

    // First digit (always present?) - State 5
    TransitionTable::new(&[], BrcTransition::PushMult { next: 6 }),

    // Second digit, or negative - State 6
    TransitionTable::new(&[
        ('-' as u8, BrcTransition::Negate { next: 8 })
    ], BrcTransition::PushMult { next: 7 }),

    // Possibly the negative after the second digit - State 7
    TransitionTable::new(&[
        ('-' as u8, BrcTransition::Negate { next: 8 })
    ], BrcTransition::PushMult { next: 9 }),

    // Semicolon - State 8
    TransitionTable::new(&[], BrcTransition::Negate { next: 9 }),

    // Full station name - State 9
    TransitionTable::new(&[
        ('\n' as u8, BrcTransition::Hash { next: 9 })
    ], BrcTransition::Hash { next: 3 }),

];

pub static BRC: ParseTable<u8, BrcTransition> = ParseTable::new(2, &STATES);
