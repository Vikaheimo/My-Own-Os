#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use kernel::filesystem::vfs::{VfsPath, VirtualFileMetadata, VirtualFilesystem};
use kernel::{
    filesystem, init,
    qemu::{QemuExitCode, exit_qemu},
};
use log::{error, info};

entry_point!(main, config = &kernel::BOOTLOADER_CONFIG);

#[allow(unreachable_code)]
fn main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info.try_into().unwrap());

    info!("Running filesystem test!");

    let fs = filesystem::tempfs::TempFilesystem::default();

    let root = fs.root();

    let file = root
        .create(VirtualFileMetadata {
            is_dir: false,
            name: "test.txt".into(),
        })
        .unwrap()
        .into_file()
        .unwrap();

    const TEXT: &[u8] = b"Hello kernel!";

    file.write(0, TEXT).unwrap();

    let mut buffer = [0u8; 32];
    let size = file.read(0, &mut buffer).unwrap();
    assert_eq!(&buffer[..size], TEXT);

    let folder = root
        .create(VirtualFileMetadata {
            is_dir: true,
            name: "test".into(),
        })
        .unwrap()
        .into_directory()
        .unwrap();

    let file2 = folder
        .create(VirtualFileMetadata {
            is_dir: false,
            name: "test.txt".into(),
        })
        .unwrap()
        .into_file()
        .unwrap();

    file2.write(0, TEXT).unwrap();

    let size = file2.read(0, &mut buffer).unwrap();
    assert_eq!(&buffer[..size], TEXT);

    info!("Testing removing files!");
    let files = root.list_files().unwrap();
    assert_eq!(files.len(), 2);

    root.remove("test.txt").unwrap();

    let files = root.list_files().unwrap();
    assert_eq!(files.len(), 1);
    info!("Removing files ok!");

    info!("Testing finding files!");

    let writable_file = fs.resolve_path(&VfsPath::parse("/test/test.txt")).unwrap().into_file().unwrap();
    writable_file.read(0, &mut buffer).unwrap();
    assert_eq!(&buffer[..size], TEXT);

    info!("Finding files ok!");

    info!("Filesystem test ok!");

    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
