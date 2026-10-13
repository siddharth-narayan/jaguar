#![feature(abi_gpu_kernel)]
#![no_std]

// use amdgpu_device_libs::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "gpu-kernel" fn kernel(/* Arguments */) {
    // println!("Hello, World!");
    // Code
}