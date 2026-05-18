use std::path::{Path, PathBuf};

fn build_image(name: &str, env_var: &str, out_dir: &Path) {
    let kernel = PathBuf::from(std::env::var_os(env_var).unwrap());

    let uefi = out_dir.join(format!("{name}_uefi.img"));
    let bios = out_dir.join(format!("{name}_bios.img"));

    bootloader::UefiBoot::new(&kernel)
        .create_disk_image(&uefi)
        .unwrap();

    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios)
        .unwrap();

    println!(
        "cargo:rustc-env={}_UEFI_PATH={}",
        name.to_uppercase(),
        uefi.display()
    );

    println!(
        "cargo:rustc-env={}_BIOS_PATH={}",
        name.to_uppercase(),
        bios.display()
    );
}

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());

    build_image("kernel", "CARGO_BIN_FILE_KERNEL_kernel", &out_dir);

    build_image("test_kernel", "CARGO_BIN_FILE_KERNEL_test_kernel", &out_dir);

    build_image("test_panic", "CARGO_BIN_FILE_KERNEL_test_panic", &out_dir);

    build_image(
        "test_stack_overflow",
        "CARGO_BIN_FILE_KERNEL_test_stack_overflow",
        &out_dir,
    );

    build_image("test_memory", "CARGO_BIN_FILE_KERNEL_test_memory", &out_dir);
}
