use std::collections::VecDeque;

use crate::table::{ParseTable, Transition, TransitionTable};
use crate::json::{JSON, JsonFragment, JsonFragmentType::{self, Incomplete}, Token};

pub fn kernel(table: &ParseTable, input: &[Token]) {
    let mut state_stack: VecDeque<(usize, usize)> = VecDeque::with_capacity(1024);
    state_stack.push_back((JsonFragmentType::Value as usize, 0));
    let mut tokenstack: VecDeque<JsonFragment> = VecDeque::with_capacity(1024);
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
        let transition_table = unsafe  { table.get(state).unwrap() };
        
        let token = match input.get(cursor) {
            Some(t) => t,
            None => {
                &Token::EOF
            }
        };

        println!("State: {}", state);
        match transition_table.get(*token) {
            Transition::None { next } => {
                println!("None from state {}", state);

                state = *next;
                cursor += 1;
            },
            
            Transition::Push { next } => {
                println!("Push from state {}", state);

                let back = unsafe { tokenstack.back_mut().unwrap() };
                back.tokens.push(token);

                state = *next;
                cursor += 1;
            },

            Transition::Call { call_state, return_state } => {
                println!("Call {} from state {}", call_state, state);

                state_stack.push_back((*call_state, *return_state));
                tokenstack.push_back(JsonFragment { fragment_type: Incomplete, children: Vec::new(), tokens: Vec::new() });
                state = *call_state;
            },

            Transition::Uncall => {
                println!("Uncall from state {}", state);
                let (caller, return_state) = unsafe { state_stack.pop_back().unwrap() };
                let caller =  unsafe { core::mem::transmute::<usize, JsonFragmentType>(caller) };
                
                let mut fragment = tokenstack.pop_back().unwrap();
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

    println!("{:#?}", result);
    println!("sucess: {}", success)
}