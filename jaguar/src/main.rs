use libloading::{Library, Symbol};
use std::ffi::{c_char, c_int, c_uint, c_void};

static CODE_OBJECT: &[u8] = include_bytes!("../../target/amdgcn-amd-amdhsa/release/jaguar_amdgpu.elf");

#[cfg(target_os = "windows")]
const HIP_NAMES: &[&str] = &["amdhip64_7.dll", "amdhip64_6.dll", "amdhip64.dll"];
#[cfg(target_os = "linux")]
const HIP_NAMES: &[&str] = &["libamdhip64.so.7", "libamdhip64.so.6", "libamdhip64.so"];

struct HIPLibrary<'a> {
    library: Library,
    hip_init: Symbol<'a, unsafe extern "C" fn(c_uint) -> HipErr>,
    module_load: Symbol<'a, unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> HipErr>,
    get_func: Symbol<'a, unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *const c_char) -> HipErr>
}

impl<'a> HIPLibrary<'a> {
    fn find_lib() -> Result<Library, String> {
        for name in HIP_NAMES {
            if let Ok(lib) = unsafe { Library::new(*name) } {
                return Ok(lib)
            }
        }

        Err(format!("HIP runtime not found (tried {HIP_NAMES:?})"))
    }

    fn new<'b>() -> Result<HIPLibrary<'b>, String> {
        let library = HIPLibrary::find_lib()?;
        let hip_init: Symbol<unsafe extern "C" fn(c_uint) -> HipErr> = unsafe { library.get(b"hipInit\0") }.map_err(|_| String::from("Unfoud"))?;
        let module_load: Symbol<unsafe extern "C" fn(*mut *mut c_void, *const c_void) -> HipErr> = unsafe { library.get(b"hipModuleLoadData\0") }.map_err(|_| String::from("Unfoud"))?;
        let get_func: Symbol<unsafe extern "C" fn(*mut *mut c_void, *mut c_void, *const c_char) -> HipErr> = unsafe { library.get(b"hipModuleGetFunction\0") }.map_err(|_| String::from("Unfoud"))?;

        Ok(HIPLibrary { library, hip_init, module_load, get_func })
    }
}


fn symbols() {
  
}

type HipErr = c_int; // 0 == hipSuccess

unsafe fn run() -> Result<(), String> {
    let lib = load_hip()?;

    
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