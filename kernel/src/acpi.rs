use core::ptr::NonNull;
use spin::{Mutex, Once};

pub static HANDLER: Once<Mutex<KernelAcpiHandler>> = Once::new();

pub fn init(physical_memory_offset: u64) -> KernelAcpiHandler {
    let handler = KernelAcpiHandler::new(physical_memory_offset);
    HANDLER.call_once(|| Mutex::new(handler));

    handler
}

#[derive(Debug, Clone, Copy)]
pub struct KernelAcpiHandler {
    physical_memory_offset: u64,
}

impl KernelAcpiHandler {
    const fn new(physical_memory_offset: u64) -> Self {
        Self {
            physical_memory_offset,
        }
    }

    #[inline]
    fn get_virtual_address(&self, address: usize) -> u64 {
        self.physical_memory_offset + address as u64
    }

    #[inline]
    unsafe fn read_physical<T>(&self, physical_address: usize) -> T {
        let virt = self.get_virtual_address(physical_address);
        // SAFETY: The caller guarantees `physical_address` points to a valid
        // mapped physical location for `T` under the static phys->virt mapping.
        unsafe { core::ptr::read_volatile(virt as *const T) }
    }

    #[inline]
    unsafe fn write_physical<T>(&self, physical_address: usize, value: T) {
        let virt = self.get_virtual_address(physical_address);
        // SAFETY: The caller guarantees `physical_address` points to a valid
        // mapped physical location for `T` under the static phys->virt mapping.
        unsafe { core::ptr::write_volatile(virt as *mut T, value) }
    }

    #[inline]
    fn io_read<T: x86_64::instructions::port::PortRead>(&self, port: u16) -> T {
        // SAFETY: ACPI AML may require port I/O, and the platform firmware
        // provides valid port addresses for these accesses.
        unsafe { x86_64::instructions::port::PortReadOnly::<T>::new(port).read() }
    }

    #[inline]
    fn io_write<T: x86_64::instructions::port::PortWrite>(&self, port: u16, value: T) {
        // SAFETY: ACPI AML may require port I/O, and the platform firmware
        // provides valid port addresses for these accesses.
        unsafe { x86_64::instructions::port::PortWriteOnly::<T>::new(port).write(value) }
    }
}

impl acpi::Handler for KernelAcpiHandler {
    #[allow(clippy::expect_used)]
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        acpi::PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(self.get_virtual_address(physical_address) as *mut T)
                .expect("Virtual start address cannot be zero!"),
            region_length: size,
            mapped_length: size,
            handler: *self,
        }
    }

    /// Unmaps a physical region. Since our physical memory mapping is static
    /// and lasts for the lifetime of the kernel, this is a no-op.
    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {
        // Nothing to clean up
    }

    fn read_u8(&self, address: usize) -> u8 {
        // SAFETY: ACPI supplies valid table/region addresses and the handler
        // maps physical memory statically for the kernel lifetime.
        unsafe { self.read_physical(address) }
    }

    fn read_u16(&self, address: usize) -> u16 {
        // SAFETY: ACPI supplies valid table/region addresses and the handler
        // maps physical memory statically for the kernel lifetime.
        unsafe { self.read_physical(address) }
    }

    fn read_u32(&self, address: usize) -> u32 {
        // SAFETY: ACPI supplies valid table/region addresses and the handler
        // maps physical memory statically for the kernel lifetime.
        unsafe { self.read_physical(address) }
    }

    fn read_u64(&self, address: usize) -> u64 {
        // SAFETY: ACPI supplies valid table/region addresses and the handler
        // maps physical memory statically for the kernel lifetime.
        unsafe { self.read_physical(address) }
    }

    fn write_u8(&self, address: usize, value: u8) {
        // SAFETY: ACPI supplies valid table/region addresses and the handler
        // maps physical memory statically for the kernel lifetime.
        unsafe { self.write_physical(address, value) }
    }

    fn write_u16(&self, address: usize, value: u16) {
        // SAFETY: ACPI supplies valid table/region addresses and the handler
        // maps physical memory statically for the kernel lifetime.
        unsafe { self.write_physical(address, value) }
    }

    fn write_u32(&self, address: usize, value: u32) {
        // SAFETY: ACPI supplies valid table/region addresses and the handler
        // maps physical memory statically for the kernel lifetime.
        unsafe { self.write_physical(address, value) }
    }

    fn write_u64(&self, address: usize, value: u64) {
        // SAFETY: ACPI supplies valid table/region addresses and the handler
        // maps physical memory statically for the kernel lifetime.
        unsafe { self.write_physical(address, value) }
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        self.io_read(port)
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        self.io_read(port)
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        self.io_read(port)
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        self.io_write(port, value);
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        self.io_write(port, value);
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        self.io_write(port, value);
    }
    fn read_pci_u8(&self, _address: acpi::PciAddress, _offset: u16) -> u8 {
        panic!("ACPI handler requested PCI read before PCI bus initialization!");
    }

    fn read_pci_u16(&self, _address: acpi::PciAddress, _offset: u16) -> u16 {
        panic!("ACPI handler requested PCI read before PCI bus initialization!");
    }

    fn read_pci_u32(&self, _address: acpi::PciAddress, _offset: u16) -> u32 {
        panic!("ACPI handler requested PCI read before PCI bus initialization!");
    }

    fn write_pci_u8(&self, _address: acpi::PciAddress, _offset: u16, _value: u8) {}

    fn write_pci_u16(&self, _address: acpi::PciAddress, _offset: u16, _value: u16) {}

    fn write_pci_u32(&self, _address: acpi::PciAddress, _offset: u16, _value: u32) {}

    #[allow(clippy::expect_used)]
    fn nanos_since_boot(&self) -> u64 {
        crate::time::uptime()
            .as_nanos()
            .try_into()
            .expect("Uptime as nanos to fit into u64!")
    }

    fn stall(&self, microseconds: u64) {
        let start = crate::time::uptime();

        while crate::time::uptime() - start < core::time::Duration::from_micros(microseconds) {
            core::hint::spin_loop();
        }
    }

    fn sleep(&self, milliseconds: u64) {
        // TODO: This should not block the thread
        // Fall back to stalling for the equivalent microseconds
        self.stall(milliseconds * 1000);
    }

    fn create_mutex(&self) -> acpi::Handle {
        unimplemented!("ACPI mutex operations!")
    }

    fn acquire(&self, _mutex: acpi::Handle, _timeout: u16) -> Result<(), acpi::aml::AmlError> {
        unimplemented!("ACPI mutex operations!")
    }

    fn release(&self, _mutex: acpi::Handle) {
        unimplemented!("ACPI mutex operations!")
    }
}
