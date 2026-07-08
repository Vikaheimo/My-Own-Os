use log::info;
use spin::Once;
use x86_64::{
    VirtAddr,
    instructions::tables::load_tss,
    registers::segmentation::{CS, Segment},
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub const DOUBLE_FAULT_STACK_SIZE: usize = 4096 * 5;
static mut DOUBLE_FAULT_STACK: [u8; DOUBLE_FAULT_STACK_SIZE] = [0; DOUBLE_FAULT_STACK_SIZE];

static TSS: Once<TaskStateSegment> = Once::new();

#[derive(Debug, Clone, Copy)]
struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

static GDT: Once<(GlobalDescriptorTable, Selectors)> = Once::new();

pub fn init() {
    #[allow(clippy::indexing_slicing)]
    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        let stack_start = VirtAddr::from_ptr(&raw const DOUBLE_FAULT_STACK);
        let stack_end = stack_start + DOUBLE_FAULT_STACK_SIZE as u64;

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;
        tss
    });

    let gdt = GDT.call_once(|| {
        let mut gdt = GlobalDescriptorTable::new();

        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(tss));
        let data_selector = gdt.append(Descriptor::kernel_data_segment());

        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
                data_selector,
            },
        )
    });

    gdt.0.load();

    // SAFETY: The GDT and TSS were initialized above and the selectors
    // come from that loaded GDT, so loading segment registers and TSS is valid.
    unsafe {
        CS::set_reg(gdt.1.code_selector);
        x86_64::instructions::segmentation::SS::set_reg(gdt.1.data_selector);
        load_tss(gdt.1.tss_selector);
    }

    info!("GDT loaded")
}
