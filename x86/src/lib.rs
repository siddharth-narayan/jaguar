use core::slice;
use parse::brc::BrcTransition;
use parse::brc::HashTable;
use parse::table::ParseTable;
use std::collections::VecDeque;
use std::vec::Vec;

pub fn brc(
    input: &[u8],
    parse_table: &ParseTable<BrcTransition>,
    hash_table: &HashTable,
    absolute_id: usize,
    total_count: usize
) {
    let slice_size = input.len() / total_count;
    let start_pos = (input.len() - 1) - (absolute_id * slice_size);
    let end_pos = (start_pos + 1) - slice_size;
    
    let mut pos = start_pos;
    let mut state = 0;

    let mut key: usize = 0;
    let mut name: u64 = 0;
    let mut current_measurement: i32 = 0;
    loop {
        let current_byte = unsafe { *input.get(pos).unwrap_unchecked() };
        let action = parse_table.get(state, current_byte);
        // println!("state {}, input: {:?} Action: {:?}, current: {}", state, current_byte as char, action, current_measurement);
        if action.update_table() {
            // println!("INserting into table");
            hash_table.update(key, name, current_measurement as i16);
            key = 0;
            name = 0;
            current_measurement = 0;
        }

        state = action.next();
        current_measurement += (current_byte.wrapping_sub(b'0') as i32) * action.mult_input_by_to_add_current_measurement() as i32;
        current_measurement += -2 * current_measurement * (action.negate_current_measurement() as i32);
        
        key = key.overflowing_add((current_byte * action.mult_input_by_to_hash()) as usize).0;
        name = name.overflowing_add((name.wrapping_mul(254) + current_byte as u64) * action.mult_input_by_to_hash() as u64).0; // I pray this actually works

        // println!("(state {}, input: {:#?}) cur: {}, key: {}, name: {}", state, current_byte as char, current_measurement, key, name);
        // println!("action: {:?}", action);
        if pos <= end_pos {
            break;
        }

        pos -= 1;
    }
}
