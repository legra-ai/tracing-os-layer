//! Build script: compiles the C wrapper for `os_log` macros on
//! Apple platforms.

fn main() {
    rhusky::Rhusky::new()
        .hooks_dir(".githooks")
        .skip_in_env("GITHUB_ACTIONS")
        .with_default_hooks()
        .install_from_build_script()
        .expect("failed to install repository Git hooks");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=csrc/wrapper.h");
    println!("cargo:rerun-if-changed=csrc/wrapper.c");

    if std::env::var("CARGO_CFG_TARGET_VENDOR").as_deref() != Ok("apple") {
        return;
    }

    cc::Build::new()
        .file("csrc/wrapper.c")
        .compile("tracing_os_layer_wrapper");
}
