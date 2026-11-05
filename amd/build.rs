pub fn main() {
    use std::collections::HashSet;

    use rustflags::Flag;

    // Find out target cpu and enabled features
    let mut target_features = env::var("CARGO_CFG_TARGET_FEATURE")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<HashSet<_>>();

    let mut target_cpu = None;
    for flag in rustflags::from_env() {
        if let Flag::Codegen { opt, value } = flag {
            if opt == "target-cpu" {
                target_cpu = value;
            } else if opt == "target-feature"
                && let Some(feat) = value
            {
                if let Some(feat) = feat.strip_prefix('-') {
                    target_features.remove(feat);
                } else {
                    let feat = feat.trim_start_matches('+');
                    target_features.insert(feat.into());
                }
            }
        }
    }
    let target_cpu = target_cpu.expect("Did not find target-cpu in RUSTFLAGS");
    let is_wave64_enabled = target_features.contains("wavefrontsize64");
    let mut build = get_link_args(is_wave64_enabled, &target_cpu);
    build.used_env_vars.push("CARGO_CFG_TARGET_FEATURE".into());

    for v in &build.used_env_vars {
        println!("cargo::rerun-if-env-changed={v}");
    }
    for f in &build.used_files {
        println!("cargo::rerun-if-changed={f}");
    }
    for a in &build.link_args {
        println!("cargo::rustc-link-arg={a}");
    }
}