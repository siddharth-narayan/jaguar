use std::{env, path::PathBuf, process::Command};
use rustflags::Flag;

pub fn main() {
    println!("cargo::rerun-if-env-changed=CARGO_CFG_TARGET_FEATURE");
    println!("cargo::rerun-if-env-changed=HIP_DEVICE_LIB_PATH");
    println!("cargo::rerun-if-env-changed=ROCM_DEVICE_LIB_PATH");
    println!("cargo::rerun-if-env-changed=ROCM_PATH");

    let mut flags: Vec<_> = rustflags::from_env().collect();
    flags.reverse();

    let target_cpu = flags.iter().find_map(|f| match f {
        Flag::Codegen { opt, value } if opt == "target-cpu" => value.as_ref(),
        _ => None,
    }).expect("target-cpu needs to be set").strip_prefix("gfx").expect("target-cpu must begin with gfx");

    let mut is_wave64 = env::var("CARGO_CFG_TARGET_FEATURE")
        .unwrap_or_default()
        .split(',')
        .any(|f| f == "wavefrontsize64");

    for f in &flags {
        if let Flag::Codegen { opt, value } = f && opt == "target_feature" {
            if *value == Some("+wavefrontsize64".into()) {
                is_wave64 = true
            } else if *value == Some("-wavefrontsize64".into()) {
                is_wave64 = false;
            }
        }
    }

    let device_libs = PathBuf::from(
        env::var("HIP_DEVICE_LIB_PATH").ok()
            .or_else(|| env::var("ROCM_DEVICE_LIB_PATH").ok())
            .or_else(|| env::var("HIP_PATH").ok().map(|p|format!("{}/amdgcn/bitcode", p)))
            .or_else(|| env::var("ROCM_PATH").ok())
            .expect("No device lib path found")
    );


    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap().into_string().unwrap());
    let bundle_path = out_dir.join("libamdgpu-device-libs.a");
    let _ = std::fs::remove_file(&bundle_path);
    
    let archiver_path = archiver_command_path();
    let output = Command::new(archiver_path).args([
        "rcs".into(),
        bundle_path.clone(),
        device_libs.join("ockl.bc"),
        device_libs.join(format!("oclc_isa_version_{}.bc", target_cpu)),
        device_libs.join(format!("oclc_abi_version_600.bc")),
        device_libs.join(format!("oclc_wavefrontsize64_{}.bc", if is_wave64 { "on" } else { "off" })),
    ]).spawn().expect("Failed to spawn llvm-ar").wait_with_output().unwrap().stdout;

    println!("output: {}", String::try_from(output).unwrap());


    println!("cargo::rustc-link-search=native={}", out_dir.display());
    println!("cargo::rustc-link-lib=static=amdgpu-device-libs");
}

fn archiver_command_path() -> PathBuf {
    let rustc_path = env::var("RUSTC").unwrap();
    let host = env::var("HOST").unwrap();
    let sysroot_path = String::from_utf8(Command::new(rustc_path).args(["--print", "sysroot"]).output().unwrap().stdout).unwrap();
    let llvm_ar_path = PathBuf::from(sysroot_path).join(format!("lib/rustlib/{}/bin/llvm-ar{}", host, env::consts::EXE_SUFFIX));

    if llvm_ar_path.exists() {
        return llvm_ar_path;
    }

    return which::which("llvm-ar").or(which::which("ar")).expect("Failed to find archiver on path");
}