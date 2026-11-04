use std::{fs, ptr::null_mut};

use hip::{Dim, HIPError, hip_describe_err, hip_init, hip_load_module};

static KERNEL: &[u8] = include_bytes!("jaguar_amdgpu.elf");

pub fn main() -> Result<(), HIPError> {
    hip_init(0).map_err(hip_describe_err)?;
    
    let module = hip_load_module(KERNEL).map_err(hip_describe_err)?;
    println!("count: {}", module.function_count().map_err(hip_describe_err)?);
    let kernel = module.get_function("kernel").map_err(hip_describe_err)?;


    println!("max pot: {:#?}", kernel.hip_module_occupancy_max_potential_blocksize(4096).map_err(hip_describe_err)?);
    let mut args = Vec::new();

    let mut inputargs = Vec::new();
    for _ in 0..64 {
        
    }

    kernel.launch(Dim {x: 32, y: 1, z: 1}, Dim {x: 1024, y: 1, z: 1}, 4096, args.as_mut_ptr(), null_mut())?;



    // ...hipMalloc, hipMemcpy, hipModuleLaunchKernel, hipDeviceSynchronize, hipFree the same way

    // check(hip_init(0), "hipInit")?;
    // let mut module = std::ptr::null_mut();
    // check(module_load(&mut module, CODE_OBJECT.as_ptr() as *const c_void), "hipModuleLoadData")?;
    // let mut func = std::ptr::null_mut();
    // check(get_func(&mut func, module, c"add_one".as_ptr()), "hipModuleGetFunction")?;
    // launch exactly as before
    Ok(())
}