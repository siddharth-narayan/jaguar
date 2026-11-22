#![feature(allocator_ext)]

use std::{alloc::System, ffi::c_void, fs::File, io::Read, mem, ptr::null_mut, thread, time::SystemTime};
use hip::{Dim, HIPError, hip_describe_err, hip_device_sync, hip_init, hip_load_module, mem::HIPAllocator};
use parse::{brc::{BrcTransition, HashTable, STATES}, table::ParseTable};
use x86::brc;

static KERNEL: &[u8] = include_bytes!("jaguar_amdgpu.elf");

pub fn main() {
    let mut text = Vec::new();
    text.push('\n' as u8); // IMPORTANT! Without this, the first processor will continue backwards beyond the buffer

    let mut file = File::open("example-1B.txt").unwrap();
    file.read_to_end(&mut text).expect("Failed to read file");
    
    let mut hash_table = HashTable::new_in(System);
    let parse_table = Box::new(ParseTable::new(STATES));

    gpu(&text, &parse_table, &mut hash_table).unwrap();
}

fn gpu(text: &Vec<u8>, parse_table: &ParseTable<BrcTransition>, hash_table: &mut HashTable) -> Result<(), HIPError> {
    hip_init(0).map_err(hip_describe_err)?;
    let module = hip_load_module(KERNEL).map_err(hip_describe_err)?;
    let kernel = module.get_function("brc").map_err(hip_describe_err)?;

    // TEXT
    let mut gpu_text: Box<Vec<u8, HIPAllocator>, HIPAllocator> = Box::new_in(Vec::new_in(HIPAllocator), HIPAllocator);
    gpu_text.reserve(text.len());
    unsafe {
        hip::mem::ptr::copy_nonoverlapping(text.as_ptr(), gpu_text.as_mut_ptr(), text.len(), hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
        gpu_text.set_len(text.len())
    };

    // HASH TABLE
    let mut hash_table_gpu: Box<HashTable, HIPAllocator> = unsafe { Box::new_uninit_in(HIPAllocator).assume_init() };
    unsafe {
        hip::mem::ptr::copy_nonoverlapping(hash_table, Box::as_mut_ptr(&mut hash_table_gpu), 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
    };

    // PARSE TABLE
    let mut parse_table_gpu: Box<ParseTable<_>, HIPAllocator> = unsafe { Box::new_uninit_in(HIPAllocator).assume_init() };
    unsafe {
        hip::mem::ptr::copy_nonoverlapping(parse_table, Box::as_mut_ptr(&mut parse_table_gpu), 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
    }

    let args = [
        gpu_text.as_ptr() as *const _,
        gpu_text.len() as *const _,
        Box::as_ptr(&parse_table_gpu) as *mut c_void,
        Box::as_ptr(&hash_table_gpu) as *mut c_void
    ];

    let args = unsafe { mem::transmute::<[*const c_void; 4], [*mut c_void; 4]>(args) }.as_mut_ptr();

    let time: SystemTime = SystemTime::now();
    kernel.launch(Dim {x: 16, y: 1, z: 1}, Dim {x: 1024, y: 1, z: 1}, 32768, args, null_mut()).map_err(hip_describe_err)?;

    if let Err(e) = hip_device_sync().map_err(hip_describe_err) {
        println!("Failed to sync: {:#?}", e);
    }

    unsafe {
        hip::mem::ptr::copy_nonoverlapping(Box::as_ptr(&hash_table_gpu), hash_table, 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
    }

    drop(args);
    drop(hash_table_gpu);
    drop(gpu_text);

    println!("Results: {:#?} ({}ms)", hash_table, time.elapsed().unwrap().as_millis());

    return Ok(())
}

fn cpu(text: &Vec<u8>, parse_table: &ParseTable<BrcTransition>, hash_table: &HashTable) {
    let time: SystemTime = SystemTime::now();

    thread::scope(|s| {
        let count = 16;
        let text = &text;
        let parse_table = &parse_table;
        let hash_table = &hash_table;
        for t in 0..count {
            s.spawn(move || {
                brc(text, parse_table, hash_table, t, count);
            });
        }
    });

    println!("Results: {:#?} ({}ms)", hash_table, time.elapsed().unwrap().as_millis());
}