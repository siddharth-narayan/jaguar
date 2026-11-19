
use std::collections::VecDeque;
use std::vec::Vec;
use parse::brc::BrcTransition;
use parse::brc::HashTable;
use core::slice;
use parse::table::{ParseTable};

const ASCII_ZERO_CODEPOINT: u8 = 48;
pub fn brc(input: &[u8], parse_table: &ParseTable<BrcTransition>, hash_table: &mut HashTable) {
    let total_count = 1;
    let absolute_id = 0;
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
                if state == 1 {
                    return;
                }
        let current_byte = unsafe { *input.get(pos).unwrap_unchecked() };

        let action = parse_table.get(state, current_byte);
        println!("action: {:#?}", action);
        match action {
            BrcTransition::Complete { next } => {
                println!("Complete: key: {}, name: {}, measurement: {}", key, name, current_measurement);
                message_id += 1;
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