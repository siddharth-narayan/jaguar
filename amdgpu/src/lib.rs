#![allow(rust_analyzer::inactive_code)]
#![cfg(target_arch = "amdgpu")]
#![no_std]
#![feature(stdarch_amdgpu)]
#![feature(abort_immediate)]
#![feature(abi_gpu_kernel)]

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use parse::brc::BrcTransition;
use parse::brc::HashTable;
use core::arch::amdgpu::wavefrontsize;
use core::slice;
use smallvec::SmallVec;
use amdgpu_device_libs::print;
use amdgpu_device_libs::prelude::*;
use parse::table::{ParseTable, TransitionTable};

fn absolute_id() -> usize {
    workgroup_id_x() as usize * 32 + workitem_id_x() as usize
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn brc(input: *const u8, input_size: *const u32, parse_table: *const ParseTable<BrcTransition>, hash_table: *const HashTable, total_processors: *const u32) {
    let input = unsafe { core::slice::from_raw_parts(input, *input_size as usize) };
    let parse_table = unsafe { &*parse_table };
    let hash_table = unsafe { &*hash_table };

    let slice_size = input.len() / (unsafe { *total_processors } as usize);
    let start_pos = (input.len() - 1) - (absolute_id() * slice_size);
    let end_pos = (start_pos + 1) - slice_size;

    let mut pos = start_pos;
    let mut state = 0;

    let mut key: usize = 0;
    let mut name: u64 = 0;
    let mut current_measurement: i32 = 0;
    loop {
        let current_byte = unsafe { *input.get_unchecked(pos) };
        let action = unsafe { parse_table.get(state, current_byte) };
        if action.update_table() {
            hash_table.update(key, name, current_measurement as i16);
            key = 0;
            name = 0;
            current_measurement = 0;
        }

        state = action.next();
        current_measurement += (current_byte.wrapping_sub(b'0') as i32) * action.mult_input_by_to_add_current_measurement() as i32;
        current_measurement += -2 * current_measurement * (action.negate_current_measurement() as i32);
        
        key = key.overflowing_add((current_byte * action.mult_input_by_to_hash()) as usize).0;
        name = name.overflowing_add((name.wrapping_mul(254) + current_byte as u64) * action.mult_input_by_to_hash() as u64).0; // I pray this actually works

        if pos <= end_pos {
            break;
        }

        pos -= 1;
    }
}