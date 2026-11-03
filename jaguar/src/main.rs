use libloading::{Library, Symbol};
use std::{ffi::{c_char, c_int, c_uint, c_void}, fs, ptr::null_mut, sync::Arc};

use crate::hip::HIP;

mod hip;

static KERNEL: &[u8] = include_bytes!(env!("CARGO_CDYLIB_FILE_JAGUAR_AMDGPU_jaguar_amdgpu"));

pub fn main() -> Result<(), ()>{
    let hip = HIP::load().unwrap();
    hip.init();
    
    let load_result = hip.load_module(KERNEL);
    fs::write("testkernelout", KERNEL);
    let module = match load_result {
        Ok(m) => m,
        Err(e) => {
            println!("{:#?}", hip.describe_err(e));
            return Err(());
        }
    };

    // ...hipMalloc, hipMemcpy, hipModuleLaunchKernel, hipDeviceSynchronize, hipFree the same way

    // check(hip_init(0), "hipInit")?;
    // let mut module = std::ptr::null_mut();
    // check(module_load(&mut module, CODE_OBJECT.as_ptr() as *const c_void), "hipModuleLoadData")?;
    // let mut func = std::ptr::null_mut();
    // check(get_func(&mut func, module, c"add_one".as_ptr()), "hipModuleGetFunction")?;
    // launch exactly as before
    Ok(())
}