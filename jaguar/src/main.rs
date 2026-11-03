use libloading::{Library, Symbol};
use std::{ffi::{c_char, c_int, c_uint, c_void}, sync::Arc};

static CODE_OBJECT: &[u8] = include_bytes!("../../target/amdgcn-amd-amdhsa/release/jaguar_amdgpu.elf");

#[cfg(target_os = "windows")]
const HIP_NAMES: &[&str] = &["amdhip64_7.dll", "amdhip64_6.dll", "amdhip64.dll"];
#[cfg(target_os = "linux")]
const HIP_NAMES: &[&str] = &["libamdhip64.so.7", "libamdhip64.so.6", "libamdhip64.so"];

type HipInit = unsafe extern "C" fn(c_uint) -> HipErr;
type ModuleLoad = unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> HipErr;
type GetFunc = unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *const c_char) -> HipErr;

struct HIPLibrary {
    library: Arc<Library>,
    hip_init: HipInit,
    module_load: ModuleLoad,
    get_func: GetFunc
}

impl HIPLibrary {
    fn find_lib() -> Result<Library, String> {
        for name in HIP_NAMES {
            if let Ok(lib) = unsafe { Library::new(*name) } {
                return Ok(lib)
            }
        }

        Err(format!("HIP runtime not found (tried {HIP_NAMES:?})"))
    }

    fn load() -> Result<HIPLibrary, String> {
        let library = Arc::new(HIPLibrary::find_lib()?);
        let libclone = library.clone();
        
        let hip_init: Symbol<unsafe extern "C" fn(c_uint) -> HipErr> = unsafe { libclone.get(b"hipInit\0") }.map_err(|_| String::from("Unfoud"))?;
        let module_load: Symbol<unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> HipErr> = unsafe { libclone.get(b"hipModuleLoadData\0") }.map_err(|_| String::from("Unfoud"))?;
        let get_func: Symbol<unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *const c_char) -> HipErr> = unsafe { libclone.get(b"hipModuleGetFunction\0") }.map_err(|_| String::from("Unfoud"))?;

        Ok(HIPLibrary { library, hip_init: *hip_init, module_load: *module_load, get_func: *get_func })
    }
}

type HipErr = c_int; // 0 == hipSuccess

unsafe fn run() -> Result<(), String> {
    let lib = HIPLibrary::load()?;

    
    // ...hipMalloc, hipMemcpy, hipModuleLaunchKernel, hipDeviceSynchronize, hipFree the same way

    // check(hip_init(0), "hipInit")?;
    // let mut module = std::ptr::null_mut();
    // check(module_load(&mut module, CODE_OBJECT.as_ptr() as *const c_void), "hipModuleLoadData")?;
    // let mut func = std::ptr::null_mut();
    // check(get_func(&mut func, module, c"add_one".as_ptr()), "hipModuleGetFunction")?;
    // launch exactly as before
    Ok(())
}

pub fn main() {

}