#![allow(rust_analyzer::inactive_code)]
#![cfg(target_arch = "amdgpu")]
#![no_std]
#![feature(stdarch_amdgpu)]
#![feature(abort_immediate)]
#![feature(abi_gpu_kernel)]

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use parse::brc::BRC;
use parse::brc::HashTable;
use parse::brc::BrcTransition;
use core::arch::amdgpu::wavefrontsize;
use core::slice;
use smallvec::SmallVec;
use amdgpu_device_libs::print;
use amdgpu_device_libs::prelude::*;
use parse::{table::{ParseTable, TransitionTable}, json::{JSON, JsonFragment, JsonFragmentType, Token}};

fn absolute_id() -> usize {
    let wg_id = workgroup_id_x();
    let id = workitem_id_x();
    let dispatch = dispatch_ptr();
    wg_id as usize * wavefrontsize() as usize + id as usize
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn json() { // table: *const ParseTable, inputs: *mut Vec<Vec<Token>>) { 
    println!("Hello, World! -- From thread #{} #{} (absolute {})", workgroup_id_x(), workitem_id_x(), absolute_id());
    return;

    // let table = table.as_ref_unchecked();
    
    // let mut state_stack: VecDeque<(usize, usize)> = VecDeque::with_capacity(1024);
    // let mut tokenstack: VecDeque<JsonFragment> = VecDeque::with_capacity(1024);

    // state_stack.push_back((JsonFragmentType::Value as usize, 0));
    // tokenstack.push_back(JsonFragment::default());
    // let mut result = JsonFragment::default();

    // let mut state = JsonFragmentType::Value as usize;
    // let mut cursor = 0;

    // let success = loop {
    //     if state == ParseTable::FINISH {
    //         break true;
    //     }

    //     if state == ParseTable::FAIL {
    //         break false;
    //     }

    //     // Avoid unwrap in real gpu code
    //     let transition_table = unsafe  { table.get(state).unwrap_unchecked() };
        
    //     let token = match input.get(cursor) {
    //         Some(t) => t,
    //         None => {
    //             &Token::EOF
    //         }
    //     };

    //     // println!("State: {}", state);
    //     match transition_table.get(*token) {
    //         Transition::None { next } => {
    //             // println!("None from state {}", state);

    //             state = *next;
    //             cursor += 1;
    //         },
            
    //         Transition::Push { next } => {
    //             // println!("Push from state {}", state);

    //             let back = unsafe { tokenstack.back_mut().unwrap_unchecked() };
    //             back.tokens.push(token);

    //             state = *next;
    //             cursor += 1;
    //         },

    //         Transition::Call { call_state, return_state } => {
    //             // println!("Call {} from state {}", call_state, state);

    //             state_stack.push_back((*call_state, *return_state));
    //             tokenstack.push_back(JsonFragment { fragment_type: JsonFragmentType::Incomplete, children: Vec::new(), tokens: SmallVec::new() });
    //             state = *call_state;
    //         },

    //         Transition::Uncall => {
    //             // println!("Uncall from state {}", state);
    //             let (caller, return_state) = unsafe { state_stack.pop_back().unwrap_unchecked() };
    //             let caller =  unsafe { core::mem::transmute::<usize, JsonFragmentType>(caller) };
                
    //             let mut fragment = unsafe { tokenstack.pop_back().unwrap_unchecked() };
    //             fragment.fragment_type = caller;
    //             if let Some(s) = tokenstack.back_mut() {
    //                 s.children.push(fragment);
    //             } else {
    //                 result = fragment;
    //             }
    //             state = return_state;
    //         },
    //     };
    // };
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn brc(input: *const u8, input_size: *const u32, table: *mut HashTable) {
    println!("Hello, World! -- From thread #{} #{} (absolute {})", workgroup_id_x(), workitem_id_x(), absolute_id());
    println!("input addr: {:#?} equals {} size: {}", input, unsafe { *input }, unsafe { *input_size });
    let input = unsafe { slice::from_raw_parts(input, input_size as usize) };
    let table = unsafe { &mut *table };

    let total_count = 1;
    let absolute_id = absolute_id();
    let slice_size = input.len() / total_count;
    
    let start_pos = (input.len() - 1) - (absolute_id * slice_size);
    let end_pos = (start_pos + 1) - slice_size;
    let mut pos = start_pos;
    let mut state = 2;
    
    let mut key = 0;
    let mut name = 0;
    let mut current_measurement: i16 = 0;
    println!("Loop time!!");
    loop {
        if pos < end_pos {
            break;
        }

        let transition_table = unsafe  { BRC.get(state).unwrap() };

        let current_byte = unsafe { *input.get(pos).unwrap() };
        match transition_table.get(current_byte) {
            BrcTransition::Complete { next } => {
                println!("Complete: key: {}, name: {}, measurement: {}", key, name, current_measurement);
                table.update(key, name, current_measurement);
                state = *next
            }
            BrcTransition::Negate { next } => {
                println!("Negation");

                current_measurement = -current_measurement;
                state = *next
            },

            BrcTransition::PushMult { next } => {
                println!("PushMult");

                current_measurement = (current_measurement * 10) + (current_byte as i16);
                state = *next
            },

            BrcTransition::Hash { next } => {
                println!("Hash");
                name = (name * 255) + current_byte as u64;
                state = *next
            }
        }
    }
}