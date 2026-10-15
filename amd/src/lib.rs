#![no_std]

#![feature(abort_immediate)]
#![feature(abi_gpu_kernel)]

use parser::TransitionTable;
use amdgpu_device_libs::prelude::*;

// use parser::

#[no_mangle]
pub extern "gpu-kernel" fn kernel(table: *const TransitionTable) {
    let table = unsafe { & *table };

    let state = TransitionTable::START;

    loop {
        if state == TransitionTable::FINISH {
            println!("Completed work item ({}, {}, {})", workitem_id_x(), workitem_id_y(), workitem_id_z());
            core::process::abort_immediate()
            // Report success?
            // dispatch_ptr().completion_signal.handle;
        }

        


    }
}