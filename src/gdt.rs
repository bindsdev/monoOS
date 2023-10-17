use spin::Lazy;
use x86_64::{
    registers::segmentation::{Segment, CS, SS},
    structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
};

struct SegmentSelectors {
    kcode: SegmentSelector,
    kdata: SegmentSelector,
    ucode: SegmentSelector,
    udata: SegmentSelector,
}

static GDT: Lazy<(GlobalDescriptorTable, SegmentSelectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let kcode = gdt.add_entry(Descriptor::kernel_code_segment());
    let kdata = gdt.add_entry(Descriptor::kernel_data_segment());
    let ucode = gdt.add_entry(Descriptor::user_code_segment());
    let udata = gdt.add_entry(Descriptor::user_data_segment());

    (
        gdt,
        SegmentSelectors {
            kcode,
            kdata,
            ucode,
            udata,
        },
    )
});

/// Initialize the GDT.
pub fn init() {
    GDT.0.load();
    log::info!("loaded GDT");

    // Reload segment registers.
    unsafe {
        CS::set_reg(GDT.1.kcode);
        SS::set_reg(GDT.1.kdata);
    }
}
