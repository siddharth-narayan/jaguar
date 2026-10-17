#![no_std]
#![feature(abort_immediate)]
#![feature(abi_gpu_kernel)]

extern crate alloc;
use alloc::collections::VecDeque;

use amdgpu_device_libs::prelude::*;

use parser::{Action, ParseTable, TransitionTable, json};

// use parser::

#[no_mangle]
pub extern "gpu-kernel" fn kernel(table: &TransitionTable, input: &[char]) {
    let stack: VecDeque<usize> = VecDeque::with_capacity(1024);
    let mut state = ParseTable::START;

    loop {
        if state == ParseTable::FINISH {
            println!("Completed work item ({}, {}, {})", workitem_id_x(), workitem_id_y(), workitem_id_z());
            core::process::abort_immediate()
            // Report success?
            // dispatch_ptr().completion_signal.handle;
        }

        let next = table.get(state).unwrap();
        match next.action {
            Action::None => {

            },
            
            Action::Push => {

            },

            Action::Reduce => {

            }
        }

        state = next.next_state;



    }
}