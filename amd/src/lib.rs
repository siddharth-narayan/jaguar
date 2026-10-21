#![no_std]

#![cfg(target_arch = "amdgpu")]
#![feature(abort_immediate)]
#![feature(abi_gpu_kernel)]

extern crate alloc;
use alloc::collections::VecDeque;

use amdgpu_device_libs::prelude::*;

use parser::{ParseTable, Transition, TransitionTable, json::{JSON, Token}};

#[no_mangle]
pub extern "gpu-kernel" fn kernel(table: &ParseTable, input: &[Token]) {
    let mut stack: VecDeque<usize> = VecDeque::with_capacity(1024);
    stack.push_back(ParseTable::FINISH);

    let mut state = ParseTable::START;
    let mut cursor = 0;

    loop {
        if state == ParseTable::FINISH {
            println!("Completed work item ({}, {}, {})", workitem_id_x(), workitem_id_y(), workitem_id_z());
            core::process::abort_immediate()
            // Report success?
            // dispatch_ptr().completion_signal.handle;
        }

        // Avoid unwrap in real gpu code
        let token = match input.get(cursor) {
            Some(t) => t,
            None => {
                &Token::EOF
            }
        };   
        
        let transition_table = unsafe  { table.get(state).unwrap() };
        match transition_table.get(*token) {
            Transition::None { next } => {
                state = *next;
                cursor += 1;
            },
            
            Transition::Push { next } => {
                // Skip for now
                state = *next;
                cursor += 1;
            },

            Transition::Call { call } => {
                stack.push_back(*call);
            },

            Transition::Uncall => {
                stack.pop_back();
            }
        };

        



    }
}