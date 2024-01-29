#![no_std]
#![forbid(unsafe_code)]
#![allow(dead_code)]
#![feature(fn_traits)]

use aster_frame::{
    bus::pci::{
        bus::PciDevice, cfg_space::Bar, common_device::PciCommonDevice, PciDeviceId, PCI_BUS,
    },
    vm::VmIo,
};
use log::info;

pub static VIRTIO_PCI_DRIVER: Once<Arc<VirtioPciDriver>> = Once::new();

pub fn init() {
    VIRTIO_PCI_DRIVER.call_once(|| Arc::new(VirtioPciDriver::new()));
    PCI_BUS
        .lock()
        .register_driver(VIRTIO_PCI_DRIVER.get().unwrap().clone());
}

#[derive(Debug)]
pub struct VirtioPciDriver {
    devices: SpinLock<Vec<VirtioPciDevice>>,
}

impl VirtioPciDriver {
    pub(super) fn new() -> Self {
        VirtioPciDriver {
            devices: SpinLock::new(Vec::new()),
        }
    }
}

impl PciDriver for VirtioPciDriver {
    fn probe(
        &self,
        device: PciCommonDevice,
    ) -> Result<Arc<dyn PciDevice>, (BusProbeError, PciCommonDevice)> {
        const VIRTIO_DEVICE_VENDOR_ID: u16 = 0x1af4;
        if device.device_id().vendor_id != VIRTIO_DEVICE_VENDOR_ID {
            return Err((BusProbeError::DeviceNotMatch, device));
        }

        // Create devce..
        let transport = VirtioPciDevice::new(device)?;
        let device = transport.pci_device().clone();
        self.devices.lock().push(transport);
        Ok(device)
    }
}

pub struct VirtioPciDevice {
    common_device: PciCommonDevice,
}

impl PciDevice for VirtioPciDevice {
    fn device_id(&self) -> PciDeviceId {
        *self.common_device.device_id()
    }
}

impl VirtioPciDevice {
    pub fn new(common_device: PciCommonDevice) -> Arc<Self> {
        let mut device = Self { common_device };

        // Initialize device...
        for capability in device.common_device.capabilities().iter() {
            info!("Capability:{:#x?}", capability);
        }

        const MAX_BAR_INDEX: u8 = 6;
        for bar_index in 0..MAX_BAR_INDEX {
            match device.common_device.bar_manager().bar(bar_index) {
                Some(bar) => match bar {
                    Bar::Memory(memory) => {
                        memory.io_mem().write_bytes(0, "Data".to_bytes());
                    }
                    Bar::Io(io) => {}
                },
                None => {}
            }
        }
    }
}
