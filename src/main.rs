use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::env;
use std::process::{Command, exit};

const TEST_NAMES: &[&str] = &[
    "TEST_KERNEL",
    "TEST_PANIC",
    "TEST_STACK_OVERFLOW",
    "TEST_MEMORY",
    "TEST_GRAPHICS",
    "TEST_FILESYSTEM",
];

const OPEN_DISPLAY_IN_TESTS: bool = false;
const OPEN_DISPLAY_IN_NORMAL: bool = true;

fn main() {
    let args: Vec<String> = env::args().collect();
    let prog = &args[0];

    let mode = args.get(1).map(|s| s.as_str());
    let firmware = args.get(2).map(|s| s.as_str());

    match (mode, firmware) {
        (Some("uefi"), None) => run_single(false, "uefi", prog),
        (Some("bios"), None) => run_single(false, "bios", prog),

        (Some("test"), Some(fw)) => run_single(true, fw, prog),

        (Some("test-all"), Some(fw)) => run_all(fw, prog),

        _ => usage_and_exit(prog),
    }
}

fn run_single(test_mode: bool, firmware: &str, prog: &str) {
    let uefi = parse_firmware(firmware, prog);

    let prefix = if test_mode { "TEST_KERNEL" } else { "KERNEL" };

    let image = get_image(prefix, uefi).unwrap_or_else(|| {
        eprintln!("Missing image for {prefix}");
        exit(1);
    });

    let open_display = if test_mode {
        OPEN_DISPLAY_IN_TESTS
    } else {
        OPEN_DISPLAY_IN_NORMAL
    };

    exit(run_qemu(&image, uefi, open_display));
}

fn run_all(firmware: &str, prog: &str) {
    let uefi = parse_firmware(firmware, prog);

    for test in TEST_NAMES {
        if let Some(image) = get_image(test, uefi) {
            println!("Running {test}...");
            let result = run_qemu(&image, uefi, OPEN_DISPLAY_IN_TESTS);

            if result != 0 {
                eprintln!("❌ {test} failed");
                exit(1);
            }

            println!("✅ {test} passed");
        }
    }

    println!("🎉 All tests passed!");
    exit(0);
}

fn get_image(prefix: &str, uefi: bool) -> Option<String> {
    let key = if uefi {
        format!("{prefix}_UEFI_PATH")
    } else {
        format!("{prefix}_BIOS_PATH")
    };

    env::var(&key).ok()
}

fn run_qemu(image: &str, uefi: bool, display: bool) -> i32 {
    let mut cmd = Command::new("qemu-system-x86_64");

    if display {
        cmd.arg("-vga").arg("std");
    } else {
        cmd.arg("-display").arg("none");
    }

    cmd.arg("-serial").arg("mon:stdio");
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    cmd.arg("-enable-kvm");
    cmd.arg("-cpu").arg("host");
    cmd.arg("-no-reboot");

    if uefi {
        let prebuilt =
            Prebuilt::fetch(Source::LATEST, "target/ovmf").expect("Failed to fetch OVMF");

        let code = prebuilt.get_file(Arch::X64, FileType::Code);
        let vars = prebuilt.get_file(Arch::X64, FileType::Vars);

        cmd.arg("-drive").arg(format!("format=raw,file={image}"));

        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,unit=0,file={},readonly=on",
            code.display()
        ));

        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,unit=1,file={},snapshot=on",
            vars.display()
        ));
    } else {
        cmd.arg("-drive").arg(format!("format=raw,file={image}"));
    }

    let status = cmd.spawn().unwrap().wait().unwrap();
    let raw = status.code().unwrap_or(1);

    if raw & 1 == 1 {
        let decoded = raw >> 1;
        if decoded == 0x10 {
            return 0;
        }
        if decoded == 0x11 {
            return 1;
        }
    }

    2
}

fn parse_firmware(firmware: &str, prog: &str) -> bool {
    match firmware {
        "uefi" => true,
        "bios" => false,
        _ => usage_and_exit(prog),
    }
}

fn usage_and_exit(prog: &str) -> ! {
    eprintln!("Usage:");
    eprintln!("  {prog} uefi");
    eprintln!("  {prog} bios");
    eprintln!("  {prog} test uefi");
    eprintln!("  {prog} test bios");
    eprintln!("  {prog} test-all uefi");
    eprintln!("  {prog} test-all bios");
    exit(1);
}
