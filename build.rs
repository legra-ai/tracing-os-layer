//! Build script: compiles the C wrapper for `os_log` macros on
//! Apple platforms.

fn main() {
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
