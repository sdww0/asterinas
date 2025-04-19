// SPDX-License-Identifier: MPL-2.0

#![allow(dead_code)]

//! Virtio over MMIO

pub mod bus;
pub mod common_device;

use alloc::vec::Vec;
use core::ops::Range;

use cfg_if::cfg_if;
use log::debug;

use self::bus::MmioBus;
use crate::{
    bus::mmio::common_device::MmioCommonDevice, mm::paddr_to_vaddr, sync::SpinLock, trap::IrqLine,
};

const VIRTIO_MMIO_MAGIC: u32 = 0x74726976;

/// MMIO bus instance
pub static MMIO_BUS: SpinLock<MmioBus> = SpinLock::new(MmioBus::new());
static IRQS: SpinLock<Vec<IrqLine>> = SpinLock::new(Vec::new());

pub(crate) fn init() {
    #[cfg(target_arch = "riscv64")]
    riscv64_mmio_probe();
}

#[cfg(target_arch = "riscv64")]
fn riscv64_mmio_probe() {
    use crate::arch::{boot::DEVICE_TREE, device::plic::enable_external_interrupt};

    let mut lock = MMIO_BUS.lock();
    for node in DEVICE_TREE
        .get()
        .unwrap()
        .find_all_nodes("/soc/virtio_mmio")
    {
        let region = node.reg().unwrap().next().unwrap();
        let interrupt = node.interrupts().unwrap().next().unwrap();
        let handle = IrqLine::alloc_specific(interrupt as u8).unwrap();
        log::debug!(
            "Initialize Virtio MMIO at {:#x?}, interrupt: {}",
            region.starting_address,
            interrupt
        );
        enable_external_interrupt(interrupt as u16);

        let device = MmioCommonDevice::new(region.starting_address as usize, handle);
        lock.register_mmio_device(device);
    }
}
