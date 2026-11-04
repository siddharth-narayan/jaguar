use hip::{HIPError, hip_describe_err, hip_load_module, hip_init};

static KERNEL: &[u8] = include_bytes!("jaguar_amdgpu.elf");

pub fn main() -> Result<(), HIPError> {
    hip_init(0);
    
    let module = hip_load_module(KERNEL).map_err(hip_describe_err)?;
    let kernel = module.get_function("kernel").map_err(hip_describe_err)?;



    // ...hipMalloc, hipMemcpy, hipModuleLaunchKernel, hipDeviceSynchronize, hipFree the same way

    // check(hip_init(0), "hipInit")?;
    // let mut module = std::ptr::null_mut();
    // check(module_load(&mut module, CODE_OBJECT.as_ptr() as *const c_void), "hipModuleLoadData")?;
    // let mut func = std::ptr::null_mut();
    // check(get_func(&mut func, module, c"add_one".as_ptr()), "hipModuleGetFunction")?;
    // launch exactly as before
    Ok(())
}