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

const ASCII_ZERO_CODEPOINT: u8 = 48;
#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn brc(input: *const u8, input_size: *const u32, parse_table: *mut ParseTable<BrcTransition>, hash_table: *mut HashTable) {
    let input = unsafe { slice::from_raw_parts(input, *input_size as usize) };
    let parse_table = unsafe { &mut *parse_table };
    let hash_table = unsafe { &mut *hash_table };

    let total_count = 16384;
    let absolute_id = absolute_id();
    let slice_size = input.len() / total_count;
    
    let mut message_id = 0;
    // println!("let start_pos = ({} - 1) - ({} * {})", input.len(), absolute_id, slice_size);
    // println!("---");

    let start_pos = (input.len() - 1) - (absolute_id * slice_size);
    let end_pos = (start_pos + 1) - slice_size;
    let mut pos = start_pos;
    let mut state = 2;
    
    let mut key = 0;
    let mut name = 0;
    let mut current_measurement: i16 = 0;
    let mut current_multiplier = 1;
    loop {
        let current_byte = unsafe { *input.get(pos).unwrap_unchecked() };

        let action = parse_table.get(state, current_byte);
        match action {
            BrcTransition::Complete { next } => {
                // println!("worker {} ({}): Complete: key: {}, name: {}, measurement: {}", absolute_id, message_id, key, name, current_measurement);
                message_id += 1;
                key;
                name;
                current_measurement;
                hash_table.update(key, name, current_measurement);
                current_measurement = 0;
                current_multiplier = 1;
                key = 0;
                name = 0; 
                state = next;

                if pos <= end_pos {
                    break;
                }
            },
            
            BrcTransition::None { next } => {
                state = next
            },

            BrcTransition::Negate { next } => {
                current_measurement = -current_measurement;
                state = next
            },

            BrcTransition::PushMult { next } => {
                current_measurement = (((current_byte - ASCII_ZERO_CODEPOINT) as i16) * current_multiplier) + current_measurement;
                current_multiplier *= 10;
                state = next;
                // println!("worker {} ({}): PushMult measurement {}", absolute_id, message_id, current_measurement);
                message_id += 1;
            },

            BrcTransition::Hash { next } => {
                // println!("worker {} ({}): Hash", absolute_id, message_id);
                message_id += 1;
                name = (name.wrapping_mul(255)) + current_byte as u64;
                state = next
            }
        }

        if pos <= end_pos {
            break;
        }
        pos -= 1;
    }
}