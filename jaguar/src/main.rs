#![feature(allocator_ext)]
#![feature(seek_stream_len)]

use std::{alloc::{GlobalAlloc, GlobalAllocator, System}, collections::HashSet, ffi::c_void, fs::{self, File}, io::{Read, Seek}, mem, ptr::null_mut, thread::{self, sleep}, time::{Duration, SystemTime}};
use hip::{Dim, HIPError, bindings::{hipDeviceSynchronize, hipMemLocationType::hipMemLocationTypeDevice}, hip_describe_err, hip_device_sync, hip_init, hip_load_module, mem::{HIPAllocator, HIPManagedAllocator, HIPMemPrefetch, HIPMemoryLocation}};
use parse::{brc::{BrcTransition, HashTable, STATES}, table::ParseTable};
use x86::brc;

static KERNEL: &[u8] = include_bytes!("jaguar_amdgpu.elf");

pub fn main() -> Result<(), HIPError> {
    let mut text = Vec::new();
    text.push('\n' as u8); // IMPORTANT! Without this, the first processor will continue backwards beyond the buffer

    let mut file = File::open("example-1B.txt").unwrap();
    file.read_to_end(&mut text);
    
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
    let mut hash_table = HashTable::new_in(System);
    let mut hash_table_gpu: Box<HashTable, HIPAllocator> = unsafe { Box::new_uninit_in(HIPAllocator).assume_init() };
    unsafe {
        println!("src: {:#?}, dst: {:#?}, len: {}", Box::as_ptr(&hash_table), Box::as_mut_ptr(&mut hash_table_gpu), 1);
        hip::mem::ptr::copy_nonoverlapping(Box::as_ptr(&hash_table), Box::as_mut_ptr(&mut hash_table_gpu), 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
        gpu_text.set_len(text.len())
    };

    // PARSE TABLE
    let parse_table = Box::new(ParseTable::new(STATES));
    // hash_table.update(5, 42, 100);
    fs::write("aaahahahaha", format!("{:#?}", parse_table));
    // println!("next: {:#?}", parse_table.get(2, 89));
    // return Ok(());

    let mut parse_table_gpu: Box<ParseTable<_>, HIPAllocator> = unsafe { Box::new_uninit_in(HIPAllocator).assume_init() };
    unsafe {
        hip::mem::ptr::copy_nonoverlapping(Box::as_ptr(&parse_table), Box::as_mut_ptr(&mut parse_table_gpu), 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
    }


    // LEN
    let len = Box::new_in(gpu_text.len() as u32, System);
    let mut len_gpu: Box<u32, HIPAllocator> = unsafe { Box::new_uninit_in(HIPAllocator).assume_init() };
    unsafe {
        hip::mem::ptr::copy_nonoverlapping(Box::as_ptr(&len), Box::as_mut_ptr(&mut len_gpu), 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
    }

    let mut args = [
        &mut gpu_text.as_mut_ptr() as *mut *mut _ as *mut c_void,
        &mut len_gpu as *mut _ as *mut c_void,
        &mut Box::as_mut_ptr(&mut parse_table_gpu) as *mut *mut _ as *mut c_void,
        &mut Box::as_mut_ptr(&mut hash_table_gpu) as *mut *mut _ as *mut c_void
    ];

    // gpu_text.prefetch(HIPMemoryLocation { type_: hipMemLocationTypeDevice, id: 0 }).map_err(hip_describe_err)?;
    // len.prefetch( HIPMemoryLocation { type_: hipMemLocationTypeDevice, id: 0 }).map_err(hip_describe_err)?;
    // table.prefetch(HIPMemoryLocation { type_: hipMemLocationTypeDevice, id: 0 }).map_err(hip_describe_err)?;

    // unsafe { hipDeviceSynchronize() };
    
    // sleep(Duration::from_secs(3));
    // let sizes = [64, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536];
    // for size in sizes {
    //     println!("max pot size {}: {:#?}", size, kernel.hip_module_occupancy_max_potential_blocksize(size).map_err(hip_describe_err)?);
    // }

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

    // brc(&text, &parse_table, &hash_table);

    //     println!("Results: {:#?} ({}ms)", hash_table, time.elapsed().unwrap().as_millis());

    // return Ok(());

    // let time: SystemTime = SystemTime::now();
    // kernel.launch(Dim {x: 16, y: 1, z: 1}, Dim {x: 1024, y: 1, z: 1}, 32768, args.as_mut_ptr(), null_mut()).map_err(hip_describe_err)?;

    // if let Err(e) = hip_device_sync().map_err(hip_describe_err) {
    //     println!("Failed to sync: {:#?}", e);
    // }

    // unsafe {
    //     hip::mem::ptr::copy_nonoverlapping(Box::as_ptr(&hash_table_gpu), Box::as_mut_ptr(&mut hash_table), 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
    // }

    // drop(hash_table_gpu);
    // drop(gpu_text);
    // drop(len);

    println!("Results: {:#?} ({}ms)", hash_table, time.elapsed().unwrap().as_millis());

    Ok(())
}