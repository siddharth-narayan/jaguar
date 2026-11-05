#![cfg(target_arch = "amdgpu")]

#![no_std]
#![feature(abort_immediate)]
#![feature(abi_gpu_kernel)]

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use smallvec::SmallVec;

use amdgpu_device_libs::prelude::*;

use parse::{table::{ParseTable, Transition, TransitionTable}, json::{JSON, JsonFragment, JsonFragmentType, Token}};

fn absolute_id() -> usize {
    let wg_id = workgroup_id_x();
    let id = workitem_id_x();
    let dispatch = dispatch_ptr();
    wg_id as usize * dispatch.workgroup_size_x as usize + id as usize
}

#[unsafe(no_mangle)]
pub extern "gpu-kernel" fn json(table: *const ParseTable, inputs: *const Vec<Vec<Token>>) {
    println!("Hello from thread {}", absolute_id());
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


struct Cs;
critical_section::set_impl!(Cs);
unsafe impl critical_section::Impl for Cs {
    unsafe fn acquire() -> critical_section::RawRestoreState { }
    unsafe fn release(_: critical_section::RawRestoreState) { }
}

// #[no_mangle]
// pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
//     let mut i = 0;
//     while i < n {
//         let x = *a.add(i);
//         let y = *b.add(i);
//         if x != y {
//             return x as i32 - y as i32;
//         }
//         i += 1;
//     }
//     0
// }