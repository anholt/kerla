use arrayvec::{ArrayString, ArrayVec};

use crate::address::PAddr;

pub struct RamArea {
    pub base: PAddr,
    pub len: usize,
}

pub struct VirtioMmioDevice {
    pub mmio_base: PAddr,
    pub irq: u8,
}

pub struct AllowedPciDevice {
    pub bus: u8,
    pub slot: u8,
}

/// Complete boot setup to pass to boot_kernel() from arch-specific initial
/// boot, including command line configuration.
pub struct BootInfo {
    pub ram_areas: ArrayVec<RamArea, 8>,
    pub virtio_mmio_devices: ArrayVec<VirtioMmioDevice, 4>,
    pub log_filter: ArrayString<64>,
    pub pci_enabled: bool,
    pub pci_allowlist: ArrayVec<AllowedPciDevice, 4>,
    pub use_second_serialport: bool,
    pub dhcp_enabled: bool,
    pub ip4: Option<ArrayString<18>>,
    pub gateway_ip4: Option<ArrayString<15>>,
}

impl BootInfo {
    pub fn new_from_command_line(ram_areas: ArrayVec<RamArea, 8>, cmdline: &[u8]) -> BootInfo {
        let cmdline = Cmdline::parse(cmdline);
        BootInfo {
            ram_areas,
            pci_enabled: cmdline.pci_enabled,
            pci_allowlist: cmdline.pci_allowlist,
            virtio_mmio_devices: cmdline.virtio_mmio_devices,
            log_filter: cmdline.log_filter,
            use_second_serialport: cmdline.use_second_serialport,
            dhcp_enabled: cmdline.dhcp_enabled,
            ip4: cmdline.ip4,
            gateway_ip4: cmdline.gateway_ip4,
        }
    }
}

/// Temporary structure for the output of parsing the kernel command line in the
/// process of setting up a BootInfo
pub struct Cmdline {
    pub pci_enabled: bool,
    pub virtio_mmio_devices: ArrayVec<VirtioMmioDevice, 4>,
    pub log_filter: ArrayString<64>,
    pub use_second_serialport: bool,
    pub dhcp_enabled: bool,
    pub ip4: Option<ArrayString<18>>,
    pub gateway_ip4: Option<ArrayString<15>>,
    pub pci_allowlist: ArrayVec<AllowedPciDevice, 4>,
}

impl Cmdline {
    pub fn parse(cmdline: &[u8]) -> Cmdline {
        let s = core::str::from_utf8(cmdline).expect("cmdline is not a utf-8 string");
        info!("cmdline: {}", if s.is_empty() { "(empty)" } else { s });

        let mut pci_enabled = true;
        let mut pci_allowlist = ArrayVec::new();
        let mut virtio_mmio_devices = ArrayVec::new();
        let mut log_filter = ArrayString::new();
        let mut use_second_serialport = false;
        let mut dhcp_enabled = true;
        let mut ip4 = None;
        let mut gateway_ip4 = None;
        if !s.is_empty() {
            for config in s.split(' ') {
                if config.is_empty() {
                    continue;
                }

                let mut words = config.splitn(2, '=');
                match (words.next(), words.next()) {
                    (Some("pci"), Some("off")) => {
                        warn!("bootinfo: PCI disabled");
                        pci_enabled = false;
                    }
                    (Some("pci_device"), Some(bus_and_slot)) => {
                        warn!("bootinfo: allowed PCI device: {}", bus_and_slot);
                        let mut iter = bus_and_slot.splitn(2, ':');
                        let bus = iter
                            .next()
                            .and_then(|w| w.parse().ok())
                            .expect("bootinfo.bus_and_slot must be formed as bus:slot");
                        let slot = iter
                            .next()
                            .and_then(|w| w.parse().ok())
                            .expect("bootinfo.bus_and_slot must be formed as bus:slot");
                        pci_allowlist.push(AllowedPciDevice { bus, slot });
                    }
                    (Some("serial1"), Some("on")) => {
                        info!("bootinfo: secondary serial port enabled");
                        use_second_serialport = true;
                    }
                    (Some("log"), Some(value)) => {
                        info!("bootinfo: log filter = \"{}\"", value);
                        if log_filter.try_push_str(value).is_err() {
                            warn!("bootinfo: log filter is too long");
                        }
                    }
                    (Some("virtio_mmio.device"), Some(value)) => {
                        let (_size, rest) = value.split_once("@0x").unwrap();
                        let (addr, irq) = rest.split_once(':').unwrap();
                        let addr = usize::from_str_radix(addr, 16).unwrap();
                        let irq = irq.parse().unwrap();

                        info!(
                            "bootinfo: virtio MMIO device: base={:016x}, irq={}",
                            addr, irq
                        );
                        virtio_mmio_devices.push(VirtioMmioDevice {
                            mmio_base: PAddr::new(addr),
                            irq,
                        })
                    }
                    (Some("dhcp"), Some("off")) => {
                        warn!("bootinfo: DHCP disabled");
                        dhcp_enabled = false;
                    }
                    (Some("ip4"), Some(value)) => {
                        let mut s = ArrayString::new();
                        if s.try_push_str(value).is_err() {
                            warn!("bootinfo: ip4 is too long");
                        }
                        ip4 = Some(s);
                    }
                    (Some("gateway_ip4"), Some(value)) => {
                        let mut s = ArrayString::new();
                        if s.try_push_str(value).is_err() {
                            warn!("bootinfo: gateway_ip4 is too long");
                        }
                        gateway_ip4 = Some(s);
                    }
                    (Some(path), None) if path.starts_with('/') => {
                        // QEMU appends a kernel image path. Just ignore it.
                    }
                    _ => {
                        warn!("cmdline: unsupported option, ignoring: '{}'", config);
                    }
                }
            }
        }

        Cmdline {
            pci_enabled,
            pci_allowlist,
            virtio_mmio_devices,
            log_filter,
            use_second_serialport,
            dhcp_enabled,
            ip4,
            gateway_ip4,
        }
    }
}
