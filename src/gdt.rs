use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_ds = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_ds = gdt.add_entry(Descriptor::user_data_segment());
        let user_cs = gdt.add_entry(Descriptor::user_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (
            gdt,
            Selectors {
                code_selector,
                kernel_ds,
                user_cs,
                user_ds,
                tss_selector,
            },
        )
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    kernel_ds: SegmentSelector,
    user_cs: SegmentSelector,
    user_ds: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, SS, DS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        SS::set_reg(GDT.1.kernel_ds);
        CS::set_reg(GDT.1.code_selector);
        DS::set_reg(GDT.1.kernel_ds);
        load_tss(GDT.1.tss_selector);
    }
}
