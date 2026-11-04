use std::{ffi::{CStr, CString}, ptr::null_mut};

use hip_sys::hiprt::{hipError_t, hipFunction_t, hipGetErrorName, hipGetErrorString, hipInit, hipModule_t, hipModuleGetFunction};

pub use ::hip_sys::hiprt::hipError_t as ErrorCode;

#[derive(Clone, Debug)]
pub struct HIPError {
    code: ErrorCode,
    name: String,
    expl: String
}

#[derive(Clone, Copy)]
pub struct Function(hipFunction_t);

#[derive(Copy, Clone)]
pub struct Module(hipModule_t);

impl Module {
    pub fn get_function(&self, name: &str) -> Result<Function, ErrorCode> {
        let mut function = null_mut();
        
        let result = unsafe { hipModuleGetFunction(&mut function, self.0,  CString::new(name).map_err(|_| ErrorCode::hipErrorInvalidValue)?.as_ptr()) };
        if result == hipError_t::hipSuccess {
            Ok(Function(function))
        } else {
            Err(result)
        }
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
    let error = unsafe { ::hip_sys::hiprt::hipModuleLoad(&mut module, kernel.as_ptr() as *const i8) };
    if error != hipError_t::hipSuccess {
        return Err(error)
    }

    return Ok(Module(module));
}

