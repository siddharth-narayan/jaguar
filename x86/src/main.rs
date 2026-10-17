use std::collections::VecDeque;

use parser::{ParseTable, Transition, TransitionTable, json::{JSON, Token}};

pub fn kernel(table: &ParseTable, input: &[Token]) {
    let mut stack: VecDeque<usize> = VecDeque::with_capacity(1024);
    stack.push_back(ParseTable::FINISH);

    let mut state = ParseTable::START;
    let mut cursor = 0;

    loop {
        println!("state: {}\ncursor: {}", state, cursor);
        if state == ParseTable::FINISH {
            panic!("Yay!")
        }

        if state == ParseTable::FAIL {
            panic!("Aww")
        }

        // Avoid unwrap in real gpu code
        let transition_table = table.get(state).unwrap();
        
        let token = match input.get(cursor) {
            Some(t) => t,
            None => {
                state = ParseTable::FAIL;
                panic!("Aww");
                continue;
            }
        };

        println!("Token: {:#?}", token);
        match transition_table.get(*token as usize) {
            Some(Transition::None { next }) => {
                println!("None");
                state = *next;
                cursor += 1;
            },
            
            Some(Transition::Push { next }) => {
                println!("Push");
                // Skip for now
                state = *next;
                cursor += 1;
            },

            Some(Transition::Call { call }) => {
                println!("Call");
                stack.push_back(state);
                state = *call;
            },

            Some(Transition::Uncall) => {
                println!("Uncall");
                state = stack.pop_back().unwrap();
            },

            _ => {
                state = ParseTable::FAIL;
            }
        };
    }
}

pub fn main() {
    let input = [Token::LBRACE, Token::COMMA, Token::RBRACE];
    kernel(&JSON, &input)
}