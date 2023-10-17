use super::mem::PhysToVirt;
use acpi::{AcpiHandler, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::{PhysAddr, VirtAddr};

#[derive(Clone, Copy)]
pub struct SystemAcpiHandler;

impl AcpiHandler for SystemAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        todo!();
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {}
}
