#![feature(allocator_ext)]

use std::{alloc::System, ffi::c_void, fs::File, io::Read, mem, ptr::null_mut, thread, time::SystemTime};
use hip::{Dim, HIPError, hip_describe_err, hip_device_sync, hip_init, hip_load_module, mem::HIPAllocator};
use parse::{brc::{BrcTransition, HashTable, STATES}, table::ParseTable};
use x86::brc;

static KERNEL: &[u8] = include_bytes!("jaguar_amdgpu.elf");

pub fn main() {
    let mut text = Vec::new();
    text.push('\n' as u8); // IMPORTANT! Without this, the first processor will continue backwards beyond the buffer

    let mut file = File::open("example-100K.txt").unwrap();
    file.read_to_end(&mut text).expect("Failed to read file");
    
    println!("Finished reading file");

    let mut hash_table = HashTable::new_in(System);
    let parse_table = Box::new(ParseTable::new(STATES));

    gpu(&text, &parse_table, &mut hash_table, Dim { x: 1024, y: 1, z: 1 }, Dim { x: 1, y: 1, z: 1 }).unwrap();
}

fn gpu(text: &[u8], parse_table: &ParseTable<BrcTransition>, hash_table: &mut HashTable, dim: Dim, block_dim: Dim) -> Result<(), HIPError> {
    hip_init(0).map_err(hip_describe_err)?;
    let module = hip_load_module(KERNEL).map_err(hip_describe_err)?;
    let kernel = module.get_function("brc").map_err(hip_describe_err)?;

    // println!("max pot: {:#?}", kernel.hip_module_occupancy_max_potential_blocksize(0).map_err(hip_describe_err)?);

    // TEXT
    let mut gpu_text: Box<Vec<u8, HIPAllocator>, HIPAllocator> = Box::new_in(Vec::new_in(HIPAllocator), HIPAllocator);
    gpu_text.reserve(text.len());
    unsafe {
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

    // DIMENSIONS
    let dimensions = (dim, block_dim);
    let mut dimensions_gpu: Box<(Dim, Dim), HIPAllocator> = unsafe { Box::new_uninit_in(HIPAllocator).assume_init() };
    unsafe {
        hip::mem::ptr::copy_nonoverlapping(&dimensions, Box::as_mut_ptr(&mut dimensions_gpu), 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
    }

    let mut text_ptr  = gpu_text.as_ptr() as *mut c_void;
    let mut text_len  = gpu_text.len() as u64;
    let mut parse_ptr = Box::as_ptr(&parse_table_gpu);
    let mut hash_ptr  = Box::as_ptr(&hash_table_gpu);
    let mut dims_ptr  = Box::as_ptr(&dimensions_gpu);

    let mut args: [*mut c_void; 5] = [
        &mut text_ptr  as *mut _ as *mut c_void,
        &mut text_len  as *mut _ as *mut c_void,
        &mut parse_ptr as *mut _ as *mut c_void,
        &mut hash_ptr  as *mut _ as *mut c_void,
        &mut dims_ptr  as *mut _ as *mut c_void,
    ];

    let mut chunk_index = 0;
    let mut chunk_start = 0;
    while let chunk_end = get_chunk_end_location(text, chunk_index, dim.x as usize * block_dim.x as usize * 128) && chunk_end != text.len() {
        chunk_index += 1;

        unsafe {
            hip::mem::ptr::copy_nonoverlapping(text.as_ptr().offset(chunk_start as isize), gpu_text.as_mut_ptr().offset(chunk_start as isize), chunk_end - chunk_start, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
        }

        kernel.launch(dim, block_dim, 0, args.as_mut_ptr(), null_mut()).map_err(hip_describe_err)?;

    }

    if let Err(e) = hip_device_sync().map_err(hip_describe_err) {
        println!("Failed to sync: {:#?}", e);
    }

    unsafe {
        hip::mem::ptr::copy_nonoverlapping(Box::as_ptr(&hash_table_gpu), hash_table, 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
    }

    drop(hash_table_gpu);
    drop(dimensions_gpu);
    drop(gpu_text);

    println!("Results: {:#?} ({}ms)", hash_table, time.elapsed().unwrap().as_millis());

    return Ok(())
}

fn cpu(text: &Vec<u8>, parse_table: &ParseTable<BrcTransition>, hash_table: &HashTable) {
    let time: SystemTime = SystemTime::now();

    thread::scope(|s| {
        let count = 16;
        // let text = &text;
        // let parse_table = &parse_table;
        // let hash_table = &hash_table;
        for t in 0..count {
            s.spawn(move || {
                brc(text, parse_table, hash_table, t, count);
            });
        }
    });

    println!("Results: {:#?} ({}ms)", hash_table, time.elapsed().unwrap().as_millis());
}

fn get_chunk_end_location(input: &[u8], index: usize, chunk_size: usize) -> usize {
    let cursor = (index + 1) * chunk_size;
    loop {
        match input.get(cursor) {
            None => return input.len(),
            Some(n) => {
                if *n == '\n' as u8 {
                    return cursor
                }
            }
        }

    }
}