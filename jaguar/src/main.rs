#![feature(allocator_ext)]

use std::{collections::HashSet, ffi::c_void, fs, mem, ptr::null_mut};

use hip::{Dim, HIPDeviceAllocator, HIPError, hip_describe_err, hip_device_sync, hip_init, hip_load_module};
use parse::{brc::HashTable, json::{JSON, Token}, table::ParseTable};

static KERNEL: &[u8] = include_bytes!("jaguar_amdgpu.elf");

pub fn main() -> Result<(), HIPError> {
    let mut text = fs::read("example.txt").unwrap().to_vec_in(HIPDeviceAllocator());
    hip_init(0).map_err(hip_describe_err)?;

    println!("niite");
    let module = hip_load_module(KERNEL).map_err(hip_describe_err)?;
    println!("count: {}", module.function_count().map_err(hip_describe_err)?);
    let kernel = module.get_function("brc").map_err(hip_describe_err)?;


    println!("max pot: {:#?}", kernel.hip_module_occupancy_max_potential_blocksize(4096).map_err(hip_describe_err)?);
    
    let mut inputs: Vec<Vec<Token>> = Vec::new();
    for _ in 0..32 * 1024 {
        inputs.push(Vec::new());
    }

    // let mut args = [ unsafe { mem::transmute(&raw const JSON) }, inputs.as_mut_ptr() as *mut c_void ];
    
    
    // let mut v: Box<HashSet<usize, HIPDeviceAllocator>, HIPDeviceAllocator> = Box::new_in(HashSet::new_in(allocator), allocator);
    // let mut b = Box::new_in(false, allocator);
    let mut table = HashTable::new_in(HIPDeviceAllocator());
    let mut len = Box::new_in(text.len() as u32, HIPDeviceAllocator());
    let mut args = [
        &mut text.as_mut_ptr() as *mut *mut _ as *mut c_void,
        &mut len as *mut _ as *mut c_void,
        &mut Box::as_mut_ptr(&mut table) as *mut *mut _ as *mut c_void
    ];

    println!("alloced and ddi everything");
    kernel.launch(Dim {x: 1, y: 1, z: 1}, Dim {x: 1, y: 1, z: 1}, 0, args.as_mut_ptr(), null_mut()).map_err(hip_describe_err)?;

    if let Err(e) = hip_device_sync().map_err(hip_describe_err) {
        println!("Failed to sync: {:#?}", e);
    }

    drop(text);
    drop(len);
    println!("after: {:#?}", table);

    // ...hipMalloc, hipMemcpy, hipModuleLaunchKernel, hipDeviceSynchronize, hipFree the same way

    // check(hip_init(0), "hipInit")?;
    // let mut module = std::ptr::null_mut();
    // check(module_load(&mut module, CODE_OBJECT.as_ptr() as *const c_void), "hipModuleLoadData")?;
    // let mut func = std::ptr::null_mut();
    // check(get_func(&mut func, module, c"add_one".as_ptr()), "hipModuleGetFunction")?;
    // launch exactly as before
    Ok(())
}