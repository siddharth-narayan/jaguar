#![feature(allocator_ext)]
#![feature(seek_stream_len)]

use std::{alloc::{GlobalAlloc, GlobalAllocator, System}, collections::HashSet, ffi::c_void, fs::{self, File}, io::{Read, Seek}, mem, ptr::null_mut, thread::sleep, time::{Duration, SystemTime}};
use hip::{Dim, HIPError, bindings::{hipDeviceSynchronize, hipMemLocationType::hipMemLocationTypeDevice}, hip_describe_err, hip_device_sync, hip_init, hip_load_module, mem::{HIPAllocator, HIPManagedAllocator, HIPMemPrefetch, HIPMemoryLocation}};
use parse::{brc::HashTable, json::{JSON, Token}, table::ParseTable};

static KERNEL: &[u8] = include_bytes!("jaguar_amdgpu.elf");

pub fn main() -> Result<(), HIPError> {
    let mut text = fs::read("example-100K.txt").unwrap();
    text.push('\n' as u8); // IMPORTANT! Without this, the first processor will continue backwards beyond the buffer
    
    hip_init(0).map_err(hip_describe_err)?;
    let module = hip_load_module(KERNEL).map_err(hip_describe_err)?;
    let kernel = module.get_function("brc").map_err(hip_describe_err)?;

    let mut gpu_text: Box<Vec<u8, HIPAllocator>, HIPAllocator> = Box::new_in(Vec::new_in(HIPAllocator), HIPAllocator);
    gpu_text.reserve(text.len());
   
    unsafe {
        hip::mem::ptr::copy_nonoverlapping(text.as_ptr(), gpu_text.as_mut_ptr(), text.len(), hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
        gpu_text.set_len(text.len())
    };

    let mut table = HashTable::new_in(System);
    let mut table_gpu: Box<HashTable, HIPAllocator> = unsafe { Box::new_uninit_in(HIPAllocator).assume_init() };
    unsafe {
        println!("src: {:#?}, dst: {:#?}, len: {}", Box::as_ptr(&table), Box::as_mut_ptr(&mut table_gpu), 1);
        hip::mem::ptr::copy_nonoverlapping(Box::as_ptr(&table), Box::as_mut_ptr(&mut table_gpu), 1, hip::bindings::hipMemcpyKind::hipMemcpyHostToDevice).map_err(hip_describe_err)?;
        gpu_text.set_len(text.len())
    };

    let mut len = Box::new_in(gpu_text.len() as u32, HIPAllocator);
    let mut args = [
        &mut gpu_text.as_mut_ptr() as *mut *mut _ as *mut c_void,
        &mut len as *mut _ as *mut c_void,
        &mut Box::as_mut_ptr(&mut table) as *mut *mut _ as *mut c_void
    ];

    // gpu_text.prefetch(HIPMemoryLocation { type_: hipMemLocationTypeDevice, id: 0 }).map_err(hip_describe_err)?;
    // len.prefetch( HIPMemoryLocation { type_: hipMemLocationTypeDevice, id: 0 }).map_err(hip_describe_err)?;
    // table.prefetch(HIPMemoryLocation { type_: hipMemLocationTypeDevice, id: 0 }).map_err(hip_describe_err)?;

    // unsafe { hipDeviceSynchronize() };
    
    // sleep(Duration::from_secs(3));
    // println!("max pot: {:#?}", kernel.hip_module_occupancy_max_potential_blocksize(0).map_err(hip_describe_err)?);

    let time = SystemTime::now();
    kernel.launch(Dim {x: 32, y: 1, z: 1}, Dim {x: 128, y: 1, z: 1}, 0, args.as_mut_ptr(), null_mut()).map_err(hip_describe_err)?;

    if let Err(e) = hip_device_sync().map_err(hip_describe_err) {
        println!("Failed to sync: {:#?}", e);
    }

    drop(gpu_text);
    drop(len);

    println!("Results: {:#?} ({}ms)", table, time.elapsed().unwrap().as_millis());

    Ok(())
}