use core::sync::atomic::{Atomic, Ordering::{Relaxed, SeqCst}};
use alloc::fmt::Debug;

use alloc::{boxed::Box, vec::Vec};
use smallvec::SmallVec;

use crate::table::{FAIL, ParseTable, TransitionTable, UNCALL};

const HASH_BUCKET_COUNT: usize = 2usize.pow(21);

#[derive(Debug)]
pub struct HashEntry {
    // taken: Atomic<bool>,
    name: Atomic<u64>,
    min: Atomic<i16>,
    max: Atomic<i16>,
    total: Atomic<i32>,
    station_count: Atomic<u32>
}

impl HashEntry {
    pub fn update(&mut self, measurement: i16) {
        self.min.fetch_min(measurement, Relaxed);
        self.max.fetch_max(measurement, Relaxed);
        self.total.fetch_add(measurement as i32, Relaxed);
        self.station_count.fetch_add(1, Relaxed);
    }
}

pub struct HashTable {
    inner: Box<[HashEntry; HASH_BUCKET_COUNT]>
}

impl HashTable {
    pub fn new() -> HashTable {
        // Safe because zeroed HashTable is valid
        let mut table = unsafe { Box::<[HashEntry; HASH_BUCKET_COUNT]>::new_zeroed().assume_init() };
        for entry in &mut table {
            entry.min = 999.into();
            entry.max = (-999).into();
        }

        HashTable {
            inner: table
        }
    }

    pub fn update(&mut self, mut key: usize, name: u64, measurement: i16) {
        key = key % HASH_BUCKET_COUNT;

        while 
            let entry = unsafe { self.inner.get_unchecked_mut(key) } 
            && let update_status = entry.name.compare_exchange(0, name, SeqCst, SeqCst)
            && !update_status.is_err_and(|already_existing_name| already_existing_name == name) {

            key += 1;
            key = key % HASH_BUCKET_COUNT;
        }

        let entry = unsafe { self.inner.get_unchecked_mut(key as usize) };
        entry.update(measurement);
    }
}

impl Debug for HashTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.inner.iter().filter(|item| item.name.load(Relaxed) != 0)).finish()
    }
}

#[derive(Copy, Clone)]
pub enum BrcTransition {
    // Can be used as a none transition, maybe need to split them, but it's an optimization for needing less transition types
    Negate { next: usize },
    PushMult { next: usize },
    Hash { next: usize },
    Complete { next: usize }
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
        ('-' as u8, BrcTransition::Negate { next: 8 }),
        (';' as u8, BrcTransition::Negate { next: 8 })
    ], BrcTransition::PushMult { next: 7 }),

    // Possibly the negative after the second digit - State 7
    TransitionTable::new(&[
        ('-' as u8, BrcTransition::Negate { next: 8 })
    ], BrcTransition::PushMult { next: 9 }),

    // Semicolon - State 8
    TransitionTable::new(&[], BrcTransition::Negate { next: 9 }),

    // Full station name - State 9
    TransitionTable::new(&[
        ('\n' as u8, BrcTransition::Complete { next: 3 })
    ], BrcTransition::Hash { next: 9 }),

];

pub static BRC: ParseTable<u8, BrcTransition> = ParseTable::new(2, &STATES);
