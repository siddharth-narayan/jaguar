#![feature(allocator_ext)]
#![feature(seek_stream_len)]

use std::{collections::HashSet, ffi::c_void, fs::{self, File}, io::{Read, Seek}, mem, ptr::null_mut, time::SystemTime};

use hip::{Dim, HIPDeviceAllocator, HIPError, hip_describe_err, hip_device_sync, hip_init, hip_load_module};
use parse::{brc::HashTable, json::{JSON, Token}, table::ParseTable};

static KERNEL: &[u8] = include_bytes!("jaguar_amdgpu.elf");

pub fn main() -> Result<(), HIPError> {
    let mut text = Vec::new_in(HIPDeviceAllocator());
    text.push('\n' as u8); // IMPORTANT! Without this, the first processor will continue backwards beyond the buffer

    let mut file = File::open("example-5M.txt").unwrap();
    let len = file.stream_len().unwrap() as usize;
    text.reserve(len);
    unsafe {
        file.read_exact(text.spare_capacity_mut().assume_init_mut()).unwrap();
        text.set_len(len + 1);
    }

    hip_init(0).map_err(hip_describe_err)?;
    let module = hip_load_module(KERNEL).map_err(hip_describe_err)?;
    let kernel = module.get_function("brc").map_err(hip_describe_err)?;

    let mut table = HashTable::new_in(HIPDeviceAllocator());
    let mut len = Box::new_in(text.len() as u32, HIPDeviceAllocator());
    let mut args = [
        &mut text.as_mut_ptr() as *mut *mut _ as *mut c_void,
        &mut len as *mut _ as *mut c_void,
        &mut Box::as_mut_ptr(&mut table) as *mut *mut _ as *mut c_void
    ];

    // println!("max pot: {:#?}", kernel.hip_module_occupancy_max_potential_blocksize(0).map_err(hip_describe_err)?);

    let time = SystemTime::now();
    kernel.launch(Dim {x: 32, y: 1, z: 1}, Dim {x: 1024, y: 1, z: 1}, 0, args.as_mut_ptr(), null_mut()).map_err(hip_describe_err)?;

    if let Err(e) = hip_device_sync().map_err(hip_describe_err) {
        println!("Failed to sync: {:#?}", e);
    }

    drop(text);
    drop(len);

    println!("Results: {:#?} ({}ms)", table, time.elapsed().unwrap().as_millis());

    Ok(())
}