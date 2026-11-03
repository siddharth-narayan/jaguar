use std::{ffi::{CStr, c_char, c_int, c_uint, c_void}, ptr::null_mut, sync::Arc};

use libloading::{Library, Symbol};

#[cfg(target_os = "windows")]
const HIP_NAMES: &[&str] = &["amdhip64_7.dll", "amdhip64_6.dll", "amdhip64.dll"];
#[cfg(target_os = "linux")]
const HIP_NAMES: &[&str] = &["libamdhip64.so.7", "libamdhip64.so.6", "libamdhip64.so"];

type HipErrCode = c_int; // 0 == hipSuccess

type HipInit = unsafe extern "C" fn(c_uint) -> HipErrCode;
type HipModule = *mut c_void;
type ModuleLoad = unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> HipErrCode;
type GetFunc = unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *const c_char) -> HipErrCode;
type GetErrorName = unsafe extern "C" fn(HipErrCode) -> *const c_char;
type GetErrorString = unsafe extern "C" fn(HipErrCode) -> *const c_char;

#[derive(Clone, Debug)]
pub struct HIPError {
    code: c_int,
    name: String,
    expl: String
}

pub struct HIP {
    library: Arc<Library>,
    hip_init: HipInit,

    get_err_name: GetErrorName,
    get_err_string: GetErrorString,

    module_load: ModuleLoad,
    get_func: GetFunc
}

impl HIP {
    fn find_lib() -> Result<Library, String> {
        for name in HIP_NAMES {
            if let Ok(lib) = unsafe { Library::new(*name) } {
                return Ok(lib)
            }
        }

        Err(format!("HIP runtime not found (tried {HIP_NAMES:?})"))
    }

    pub fn load() -> Result<HIP, String> {
        let library = Arc::new(HIP::find_lib()?);
        let libclone = library.clone();
        
        let hip_init: Symbol<HipInit> = unsafe { libclone.get(b"hipInit\0") }.map_err(|_| String::from("Unfoud"))?;
        let module_load: Symbol<ModuleLoad> = unsafe { libclone.get(b"hipModuleLoadData\0") }.map_err(|_| String::from("Unfoud"))?;
        let get_func: Symbol<GetFunc> = unsafe { libclone.get(b"hipModuleGetFunction\0") }.map_err(|_| String::from("Unfoud"))?;
        
        let get_err_name: Symbol<GetErrorName> = unsafe { libclone.get(b"hipGetErrorName\0") }.map_err(|_| String::from("Unfoud"))?;
        let get_err_string: Symbol<GetErrorString> = unsafe { libclone.get(b"hipGetErrorString\0") }.map_err(|_| String::from("Unfoud"))?;

        Ok(HIP { library, hip_init: *hip_init, module_load: *module_load, get_func: *get_func, get_err_name: *get_err_name, get_err_string: *get_err_string })
    }

    pub fn init(&self) {
        unsafe { (self.hip_init)(0) };
    }

    pub fn describe_err(&self, code: HipErrCode) -> HIPError {
        let name = unsafe { (self.get_err_name)(code) };
        let string = unsafe { (self.get_err_string)(code) };

        HIPError {
            code: code,
            name: unsafe { CStr::from_ptr(name) }.to_str().unwrap().to_string(),
            expl: unsafe { CStr::from_ptr(string) }.to_str().unwrap().to_string()
        }
    }

    pub fn load_module(&self, kernel: &[u8]) -> Result<HipModule, HipErrCode> {
        let mut module = null_mut();
        let error = unsafe { (self.module_load)(&mut module, kernel.as_ptr() as *const c_void) };
        if error != 0 {
            return Err(error)
        }

        return Ok(module);
    }
}

