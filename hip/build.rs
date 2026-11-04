#![feature(exit_status_error)]

fn main() {
    let mut hipconfig = std::process::Command::new("hipconfig");
    hipconfig.arg("-p");

    let hip_path: String = hipconfig.output()
        .expect("hipconfig -p failed to run").exit_ok()
        .expect("hipconfig -p returned with a nonzero status code").stdout.try_into()
        .expect("hipconfig -p returned invalid UTF-8");

    println!("cargo:rustc-link-search=native={}", format!("{}/lib", hip_path));
    println!("cargo:rustc-link-lib=dylib=amdhip64");
    println!("cargo:rerun-if-changed=src/wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("src/wrapper.h")
        .clang_arg(format!("-I{}/include", hip_path))
        .clang_arg(format!("-I{}/include/hipify", hip_path))
        .clang_arg("-D__HIP_PLATFORM_AMD__") // needed for rocm>6)
        .rustified_non_exhaustive_enum("hip.*")
        .generate_block(false)
        .size_t_is_usize(true)
        .ctypes_prefix("::libc")
        .derive_default(true)
        .derive_eq(true)
        .derive_ord(true)
        .derive_hash(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");
    
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}