use core::{alloc::Allocator, sync::atomic::{Atomic, Ordering::{Relaxed, SeqCst}}};
use alloc::fmt::Debug;

use alloc::{boxed::Box, vec::Vec};
use smallvec::SmallVec;

use crate::table::{FAIL, ParseTable, TransitionTable, UNCALL, ZeroedDefault};

const HASH_BUCKET_COUNT: usize = 2usize.pow(18);

#[derive(Debug)]
#[repr(C)]
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

#[repr(C)]
pub struct HashTable {
    inner: [HashEntry; HASH_BUCKET_COUNT]
}

impl HashTable {
    pub fn new_in<A: Allocator>(allocator: A) -> Box<HashTable, A> {
        // Safe because zeroed HashTable is valid
        let mut table = unsafe { Box::<HashTable, A>::new_zeroed_in(allocator).assume_init() };
        for entry in &mut table.inner {
            entry.min = 999.into();
            entry.max = (-999).into();
        }

        table
    }

    pub fn update(&mut self, mut key: usize, name: u64, measurement: i16) {
        // println!("update for key {}", key);
        key = key % HASH_BUCKET_COUNT;

        while 
            let entry = unsafe { self.inner.get_unchecked_mut(key) } 
            && let update_status = entry.name.compare_exchange(0, name, SeqCst, SeqCst)
            && update_status.is_err_and(|already_existing_name| already_existing_name != name) {

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

// #[derive(Copy, Clone, Debug)]
// #[repr(u16)]
// pub enum BrcTransition {
//     // Can be used as a none transition, maybe need to split them, but it's an optimization for needing less transition types
//     None { next: u8 } = 0,
//     Negate { next: u8 },
//     PushMult { next: u8 },
//     Hash { next: u8 },
//     Complete { next: u8 }
// }
// unsafe impl ZeroedDefault for BrcTransition {}

#[derive(Copy, Clone)]
pub struct BrcTransition {
    // next: u4,

    // -1 or 1 (or maybe 0?)
    // mult_existing_val_by: u2,
    
    // 0 or 1
    // mult_input_by_to_hash: u1,

    // 0 or 1
    // update_table: u1
    value: u8
}

impl Debug for BrcTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrcTransition")
            .field("next", &self.next())
            .field("mult_existing_val_by", &self.mult_existing_val_by())
            .field("mult_input_by_to_hash", &self.mult_input_by_to_hash())
            .field("update_table", &self.update_table())
            .finish()
    }
}

unsafe impl ZeroedDefault for BrcTransition {}

impl BrcTransition {
    pub const fn default() -> BrcTransition {
        BrcTransition::new(0, 0, 0, 0)
    }

    pub const fn new(next: u8, mult_existing_val_by: i8, mult_input_by_to_hash: u8, update_table: u8) -> BrcTransition {
        debug_assert!(next < 16);
        debug_assert!(mult_existing_val_by < 2 && mult_existing_val_by > -2);
        debug_assert!(mult_input_by_to_hash < 2);

        let mut value  = 0;
        value += next.deposit_bits(0b1111_0000);
        value += ((mult_existing_val_by + 1) as u8).deposit_bits(0b0000_1100);
        value += mult_input_by_to_hash.deposit_bits(0b0000_0010);
        value += update_table.deposit_bits(0b0000_0001);

        BrcTransition { value }
    }

    #[inline]
    pub fn next(self) -> u8 {
        return self.value.extract_bits(0b1111_0000);
    }

    #[inline]
    pub fn mult_existing_val_by(self) -> i8 {
        return (self.value.extract_bits(0b0000_1100) as i8) - 1;
    }

    #[inline]
    pub fn mult_input_by_to_hash(self) -> u8 {
        return self.value.extract_bits(0b0000_0010)
    }

    #[inline]
    pub fn update_table(self) -> bool {
        return unsafe { core::mem::transmute(self.value.extract_bits(0b0000_0001)) }
    }
}

#[rustfmt::skip]
pub const STATES: &'static [TransitionTable<u8, BrcTransition>] = &[
    // Fail - State 0
    TransitionTable::new(&[], BrcTransition::new(0, 1, 0, 0)),

    // Finish - State 1
    TransitionTable::new(&[], BrcTransition::new(1, 1, 0, 0)),
    
    // Search for newline - State 2
    TransitionTable::new(&[
        (b'\n', BrcTransition::new(3, 0, 0, 0))
    ], BrcTransition::new(2, 0, 0, 0)),

    // Single fractional digit - State 3
    TransitionTable::new(&[], BrcTransition::new(4, 1, 0, 0)),
    
    // Decimal point - State 4
    TransitionTable::new(&[], BrcTransition::new(5, 1, 0, 0)),

    // First digit (always present?) - State 5
    TransitionTable::new(&[], BrcTransition::new(6, 1, 0, 0)),

    // Second digit, or negative, or semicolon - State 6
    TransitionTable::new(&[
        (b'-', BrcTransition::new(9, -1, 0, 0)),
        (b';', BrcTransition::new(9, 1, 0, 0))
    ], BrcTransition::new(7, 1, 1, 0)),

    // Possibly the negative after the second digit - State 7
    TransitionTable::new(&[
        (b'-', BrcTransition::new(8, -1, 0, 0)),
        (b';', BrcTransition::new(9, 1, 0, 0))
    ], BrcTransition::default()),

    // The semi after the negative after the second digit >:(
    TransitionTable::new(&[
        (b';', BrcTransition::new(9, 1, 0, 0))
    ], BrcTransition::default()),

    // Full station name - State 9
    TransitionTable::new(&[
        // Complete
        (b'\n', BrcTransition::new(3, 0, 0, 1))
    ], BrcTransition::new(9, 1, 1, 0)),
];

// pub static BRC: ParseTable<BrcTransition> = ParseTable::new(STATES);
