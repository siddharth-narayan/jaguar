use std::{alloc::{AllocError, Allocator}, ffi::{CStr, CString, c_void}, mem::{self, ManuallyDrop}, ptr::{NonNull, null, null_mut}};

#[allow(non_camel_case_types)]  
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
pub mod bindings;
use bindings::{hipError_t, hipError_t as ErrorCode, hipFunction_t, hipGetErrorName, hipModuleGetFunctionCount, hipGetErrorString, hipInit, hipModule_t, hipModuleGetFunction, hipModuleLaunchKernel, hipModuleLoad, hipModuleOccupancyMaxPotentialBlockSize, hipModuleUnload, hipStream_t};

use crate::bindings::{hipDeviceSynchronize, hipFree, hipMalloc, hipModuleLoadData};

#[derive(Clone, Debug)]
pub struct HIPError {
    code: ErrorCode,
    name: String,
    expl: String
}

#[derive(Clone, Copy, Debug)]
pub struct Dim {
    pub x: u32,
    pub y: u32,
    pub z: u32
}

pub struct Module (hipModule_t);
impl Module {
    pub fn function_count(&self) -> Result<u32, ErrorCode> {
        let mut count = 0;
        let result = unsafe { hipModuleGetFunctionCount(&mut count, self.0) };
        if result == hipError_t::hipSuccess {
            Ok(count)
        } else {
            Err(result)
        }
    }

    pub fn get_function(&self, name: &str) -> Result<Function, ErrorCode> {
        let mut function = null_mut();
        
        let result = unsafe { hipModuleGetFunction(&mut function, self.0,  CString::new(name).map_err(|_| ErrorCode::hipErrorInvalidValue)?.as_ptr()) };
        if result == hipError_t::hipSuccess {
            Ok(Function(function))
        } else {
            Err(result)
        }
    }

    fn unload(module: hipModule_t) -> Result<(), ErrorCode> {
        let result = unsafe { hipModuleUnload(module) };
        if result == hipError_t::hipSuccess {
            Ok(())
        } else {
            Err(result)
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        let _ = Module::unload(self.0);
    }
}

pub struct Function(hipFunction_t);
impl Function {
    pub fn launch(&self, dim: Dim, block_dim: Dim, shared_bytes: u32, kernel_params: *mut *mut ::libc::c_void, extra: *mut *mut ::libc::c_void) -> Result<(), ErrorCode> {
        let result = unsafe { 
            hipModuleLaunchKernel(
                self.0,
                dim.x,
                dim.y,
                dim.z,
                block_dim.x,
                block_dim.y,
                block_dim.z,
                shared_bytes,
                null_mut(),
                kernel_params,
                extra
            )
        };

        if result == ErrorCode::hipSuccess {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn hip_module_occupancy_max_potential_blocksize(&self, shared_mem: usize) -> Result<(i32, i32), ErrorCode> {
        let mut grid_size = 0;
        let mut block_size = 0;

        let error = unsafe { hipModuleOccupancyMaxPotentialBlockSize(&mut grid_size, &mut block_size, self.0, shared_mem, 0) };
        if error != hipError_t::hipSuccess {
            return Err(error)
        }

        return Ok((grid_size, block_size));
    }
}

pub fn hip_init(flags: u32) -> Result<(), ErrorCode> {
    let result = unsafe { hipInit(flags) };
    if result == hipError_t::hipSuccess {
        Ok(())
    } else {
        Err(result)
    }
}

pub fn hip_describe_err(code: ErrorCode) -> HIPError {
    let name = unsafe { hipGetErrorName(code) };
    let string = unsafe { hipGetErrorString(code) };

    HIPError {
        code: code,
        name: unsafe { CStr::from_ptr(name) }.to_str().unwrap().to_string(),
        expl: unsafe { CStr::from_ptr(string) }.to_str().unwrap().to_string()
    }
}

pub fn hip_load_module(kernel: &[u8]) -> Result<Module, ErrorCode> {
    let mut module = null_mut();
    let error = unsafe { hipModuleLoadData(&mut module, kernel.as_ptr() as *const c_void) };
    if error != hipError_t::hipSuccess {
        return Err(error)
    }

    return Ok(Module(module));
}

pub fn hip_device_sync() -> Result<(), ErrorCode> {
    let error = unsafe { hipDeviceSynchronize() };
    if error != hipError_t::hipSuccess {
        return Err(error)
    }

    return Ok(());
}

#[derive(Clone, Copy)]
pub struct HIPDeviceAllocator();
impl HIPDeviceAllocator {
    pub fn new() -> HIPDeviceAllocator {
        HIPDeviceAllocator()
    }
}

unsafe impl Allocator for HIPDeviceAllocator {
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let mut ptr: *mut u8 = null_mut();
        _ = unsafe { hipMalloc(&raw mut ptr as _, layout.size()) };

        if ptr.is_null() {
            return Err(AllocError)
        }

        let slice = unsafe { std::slice::from_raw_parts(ptr, layout.size()) };
        Ok(NonNull::from_ref(slice))
    }

    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        _ = unsafe { hipFree(ptr.as_ptr() as _) }
    }
}