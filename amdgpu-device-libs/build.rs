use std::env;
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

    let device_libs = env::var("HIP_DEVICE_LIB_PATH").ok()
            .or_else(|| env::var("ROCM_DEVICE_LIB_PATH").ok())
            .or_else(|| env::var("ROCM_PATH").ok())
            .expect("No device lib path found");

    println!("cargo::rustc-cdylib-link-arg={}/ockl.bc", device_libs);
    println!("cargo::rustc-cdylib-link-arg={}/oclc_isa_version_{}.bc", device_libs, target_cpu);
    println!("cargo::rustc-cdylib-link-arg={}/oclc_abi_version_600.bc", device_libs);
    println!("cargo::rustc-cdylib-link-arg={}/oclc_wavefrontsize64_{}.bc", device_libs, if is_wave64 { "on" } else { "off" });

    println!("cargo::rustc-cdylib-link-arg=--undefined-version");
    println!("cargo::rustc-cdylib-link-arg=--no-gc-sections");
    println!("cargo::rustc-cdylib-link-arg=--plugin-opt=mcpu=gfx{target_cpu}");
}