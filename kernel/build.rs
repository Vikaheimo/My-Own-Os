fn main() {
    println!(
        "cargo:rustc-check-cfg=cfg(kernel_log_level, \
         values(\"off\", \"error\", \"warn\", \"info\", \"debug\", \"trace\"))"
    );

    println!("cargo:rerun-if-env-changed=KERNEL_LOG_LEVEL");

    let level = std::env::var("KERNEL_LOG_LEVEL")
        .unwrap_or_else(|_| "debug".into());

    println!("cargo:rustc-cfg=kernel_log_level=\"{level}\"");
}