#![cfg(target_arch = "amdgpu")]

#![no_std]
#![feature(abort_immediate)]
#![feature(abi_gpu_kernel)]

use embedded_alloc::LlffHeap;

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();

extern crate alloc;
use alloc::collections::VecDeque;
use alloc::vec::Vec;

use smallvec::SmallVec;

// use amdgpu_device_libs::prelude::*;

use parse::{table::{ParseTable, Transition, TransitionTable}, json::{JSON, JsonFragment, JsonFragmentType, Token}};

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "gpu-kernel" fn kernel(table: &ParseTable, input: &[Token]) {
    let mut state_stack: VecDeque<(usize, usize)> = VecDeque::with_capacity(1024);
    let mut tokenstack: VecDeque<JsonFragment> = VecDeque::with_capacity(1024);

    state_stack.push_back((JsonFragmentType::Value as usize, 0));
    tokenstack.push_back(JsonFragment::default());
    let mut result = JsonFragment::default();

    let mut state = JsonFragmentType::Value as usize;
    let mut cursor = 0;

    let success = loop {
        if state == ParseTable::FINISH {
            break true;
        }

        if state == ParseTable::FAIL {
            break false;
        }

        // Avoid unwrap in real gpu code
        let transition_table = unsafe  { table.get(state).unwrap_unchecked() };
        
        let token = match input.get(cursor) {
            Some(t) => t,
            None => {
                &Token::EOF
            }
        };

        // println!("State: {}", state);
        match transition_table.get(*token) {
            Transition::None { next } => {
                // println!("None from state {}", state);

                state = *next;
                cursor += 1;
            },
            
            Transition::Push { next } => {
                // println!("Push from state {}", state);

                let back = unsafe { tokenstack.back_mut().unwrap_unchecked() };
                back.tokens.push(token);

                state = *next;
                cursor += 1;
            },

            Transition::Call { call_state, return_state } => {
                // println!("Call {} from state {}", call_state, state);

                state_stack.push_back((*call_state, *return_state));
                tokenstack.push_back(JsonFragment { fragment_type: JsonFragmentType::Incomplete, children: Vec::new(), tokens: SmallVec::new() });
                state = *call_state;
            },

            Transition::Uncall => {
                // println!("Uncall from state {}", state);
                let (caller, return_state) = unsafe { state_stack.pop_back().unwrap_unchecked() };
                let caller =  unsafe { core::mem::transmute::<usize, JsonFragmentType>(caller) };
                
                let mut fragment = unsafe { tokenstack.pop_back().unwrap_unchecked() };
                fragment.fragment_type = caller;
                if let Some(s) = tokenstack.back_mut() {
                    s.children.push(fragment);
                } else {
                    result = fragment;
                }
                state = return_state;
            },
        };
    };
}