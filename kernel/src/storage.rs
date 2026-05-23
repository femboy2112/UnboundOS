//! Storage stage 1 contracts — spec §7.1–§7.3 and §13.8.
//!
//! M6 starts with raw-sector reads and finite polling. ATA PIO port I/O is
//! exposed through an explicit unsafe hardware entry point and a testable port
//! backend contract.

pub const SECTOR_WORDS: usize = 256;
pub const SECTOR_BYTES: u32 = 512;
pub const WRITE_SUPPORT_ENABLED: bool = false;
pub const ACTIVE_NODE_NONE: u32 = u32::MAX;
pub const STATUS_UNAVAILABLE: u8 = 0xFF;
pub const DEFAULT_ATA_TIMEOUT_POLLS: u32 = 100_000;

pub const ATA_STATUS_ERR: u8 = 0x01;
pub const ATA_STATUS_DRQ: u8 = 0x08;
pub const ATA_STATUS_DF: u8 = 0x20;
pub const ATA_STATUS_BSY: u8 = 0x80;

pub const ATA_LBA28_MAX: u64 = 0x0FFF_FFFF;

const ATA_REG_DATA: u16 = 0x1F0;
const ATA_REG_SECTOR_COUNT: u16 = 0x1F2;
const ATA_REG_LBA_LOW: u16 = 0x1F3;
const ATA_REG_LBA_MID: u16 = 0x1F4;
const ATA_REG_LBA_HIGH: u16 = 0x1F5;
const ATA_REG_DRIVE_HEAD: u16 = 0x1F6;
const ATA_REG_STATUS_COMMAND: u16 = 0x1F7;

const ATA_CMD_READ_SECTORS: u8 = 0x20;
const ATA_DRIVE_MASTER_LBA: u8 = 0xE0;

pub type SectorBuffer = [u16; SECTOR_WORDS];

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBackend {
    AtaPio = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageOperation {
    ReadSector = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageErrorKind {
    Timeout = 1,
    DeviceError = 2,
    LbaOutOfRange = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageDiagnostic {
    pub backend: StorageBackend,
    pub operation: StorageOperation,
    pub kind: StorageErrorKind,
    pub status_register: u8,
    pub reserved0: u8,
    pub reserved1: u16,
    pub lba: u64,
    pub timeout_count: u32,
    pub active_node: u32,
}

impl StorageDiagnostic {
    pub const fn new(
        backend: StorageBackend,
        operation: StorageOperation,
        kind: StorageErrorKind,
        status_register: u8,
        lba: u64,
        timeout_count: u32,
        active_node: u32,
    ) -> Self {
        Self {
            backend,
            operation,
            kind,
            status_register,
            reserved0: 0,
            reserved1: 0,
            lba,
            timeout_count,
            active_node,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageError {
    diagnostic: StorageDiagnostic,
}

impl StorageError {
    pub const fn diagnostic(&self) -> StorageDiagnostic {
        self.diagnostic
    }

    const fn from_diagnostic(diagnostic: StorageDiagnostic) -> Self {
        Self { diagnostic }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadSectorRequest {
    lba: u64,
    active_node: u32,
}

impl ReadSectorRequest {
    pub const fn new(lba: u64) -> Result<Self, StorageError> {
        if lba > ATA_LBA28_MAX {
            return Err(StorageError::from_diagnostic(StorageDiagnostic::new(
                StorageBackend::AtaPio,
                StorageOperation::ReadSector,
                StorageErrorKind::LbaOutOfRange,
                STATUS_UNAVAILABLE,
                lba,
                0,
                ACTIVE_NODE_NONE,
            )));
        }
        Ok(Self {
            lba,
            active_node: ACTIVE_NODE_NONE,
        })
    }

    pub const fn with_active_node(mut self, active_node: u32) -> Self {
        self.active_node = active_node;
        self
    }

    pub const fn lba(&self) -> u64 {
        self.lba
    }

    pub const fn active_node(&self) -> u32 {
        self.active_node
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeoutBudget {
    max_polls: u32,
}

impl TimeoutBudget {
    pub const fn new(max_polls: u32) -> Self {
        Self { max_polls }
    }

    pub const fn max_polls(self) -> u32 {
        self.max_polls
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadyStatus {
    pub status_register: u8,
    pub polls: u32,
}

pub trait AtaPioPortIo {
    fn outb(&mut self, port: u16, value: u8);
    fn inb(&mut self, port: u16) -> u8;
    fn inw(&mut self, port: u16) -> u16;
}

pub struct AtaPioHardware;

impl AtaPioPortIo for AtaPioHardware {
    fn outb(&mut self, port: u16, value: u8) {
        // SAFETY: caller reached this implementation through
        // `ata_pio_read_sector`, whose safety contract requires exclusive
        // ownership of the legacy ATA command/data port range for this boot
        // phase.
        unsafe {
            core::arch::asm!(
                "out dx, al",
                in("dx") port,
                in("al") value,
                options(nomem, nostack, preserves_flags),
            );
        }
    }

    fn inb(&mut self, port: u16) -> u8 {
        let value: u8;
        // SAFETY: caller reached this implementation through
        // `ata_pio_read_sector`, whose safety contract requires exclusive
        // ownership of the legacy ATA status port for this boot phase.
        unsafe {
            core::arch::asm!(
                "in al, dx",
                out("al") value,
                in("dx") port,
                options(nomem, nostack, preserves_flags),
            );
        }
        value
    }

    fn inw(&mut self, port: u16) -> u16 {
        let value: u16;
        // SAFETY: caller reached this implementation through
        // `ata_pio_read_sector`, whose safety contract requires exclusive
        // ownership of the legacy ATA data port and a caller-provided sector
        // buffer large enough for exactly 256 words.
        unsafe {
            core::arch::asm!(
                "in ax, dx",
                out("ax") value,
                in("dx") port,
                options(nomem, nostack, preserves_flags),
            );
        }
        value
    }
}

/// Poll ATA status until DRQ is ready, an error bit appears, or the finite
/// budget expires. This is the host-testable contract used by the later unsafe
/// ATA PIO port reader.
pub fn poll_ata_status<F>(
    request: ReadSectorRequest,
    budget: TimeoutBudget,
    mut read_status: F,
) -> Result<ReadyStatus, StorageError>
where
    F: FnMut() -> u8,
{
    let mut polls = 0;
    let mut last_status = STATUS_UNAVAILABLE;
    while polls < budget.max_polls() {
        let status = read_status();
        polls += 1;
        last_status = status;

        if status & (ATA_STATUS_ERR | ATA_STATUS_DF) != 0 {
            return Err(StorageError::from_diagnostic(StorageDiagnostic::new(
                StorageBackend::AtaPio,
                StorageOperation::ReadSector,
                StorageErrorKind::DeviceError,
                status,
                request.lba(),
                polls,
                request.active_node(),
            )));
        }

        if status & ATA_STATUS_BSY == 0 && status & ATA_STATUS_DRQ != 0 {
            return Ok(ReadyStatus {
                status_register: status,
                polls,
            });
        }
    }

    Err(StorageError::from_diagnostic(StorageDiagnostic::new(
        StorageBackend::AtaPio,
        StorageOperation::ReadSector,
        StorageErrorKind::Timeout,
        last_status,
        request.lba(),
        polls,
        request.active_node(),
    )))
}

/// Read one 512-byte sector through the legacy ATA PIO command sequence from
/// spec §7.3.
pub fn ata_pio_read_sector_with_ports<P: AtaPioPortIo>(
    ports: &mut P,
    request: ReadSectorRequest,
    budget: TimeoutBudget,
    sector: &mut SectorBuffer,
) -> Result<ReadyStatus, StorageError> {
    let lba = request.lba() as u32;
    let drive_head = ATA_DRIVE_MASTER_LBA | ((lba >> 24) as u8 & 0x0F);

    ports.outb(ATA_REG_DRIVE_HEAD, drive_head);
    ports.outb(ATA_REG_SECTOR_COUNT, 1);
    ports.outb(ATA_REG_LBA_LOW, lba as u8);
    ports.outb(ATA_REG_LBA_MID, (lba >> 8) as u8);
    ports.outb(ATA_REG_LBA_HIGH, (lba >> 16) as u8);
    ports.outb(ATA_REG_STATUS_COMMAND, ATA_CMD_READ_SECTORS);

    let ready = poll_ata_status(request, budget, || ports.inb(ATA_REG_STATUS_COMMAND))?;
    for word in sector.iter_mut() {
        *word = ports.inw(ATA_REG_DATA);
    }
    Ok(ready)
}

/// Read one sector from the legacy primary ATA PIO backend.
///
/// # Safety
///
/// Caller must ensure the primary ATA PIO port range (`0x1F0..=0x1F7`) belongs
/// to the kernel for this boot phase and that no concurrent storage code is
/// issuing commands on the same device.
pub unsafe fn ata_pio_read_sector(
    request: ReadSectorRequest,
    budget: TimeoutBudget,
    sector: &mut SectorBuffer,
) -> Result<ReadyStatus, StorageError> {
    let mut ports = AtaPioHardware;
    ata_pio_read_sector_with_ports(&mut ports, request, budget, sector)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakePorts {
        writes: Vec<(u16, u8)>,
        statuses: Vec<u8>,
        words: Vec<u16>,
        status_reads: usize,
        word_reads: usize,
    }

    impl AtaPioPortIo for FakePorts {
        fn outb(&mut self, port: u16, value: u8) {
            self.writes.push((port, value));
        }

        fn inb(&mut self, port: u16) -> u8 {
            assert_eq!(port, ATA_REG_STATUS_COMMAND);
            let status = self.statuses[self.status_reads];
            self.status_reads += 1;
            status
        }

        fn inw(&mut self, port: u16) -> u16 {
            assert_eq!(port, ATA_REG_DATA);
            let word = self.words[self.word_reads];
            self.word_reads += 1;
            word
        }
    }

    #[test]
    fn request_accepts_lba28_boundary() {
        let request = ReadSectorRequest::new(ATA_LBA28_MAX).unwrap();
        assert_eq!(request.lba(), ATA_LBA28_MAX);
        assert_eq!(request.active_node(), ACTIVE_NODE_NONE);
    }

    #[test]
    fn request_rejects_out_of_range_lba_with_diagnostic() {
        let err = ReadSectorRequest::new(ATA_LBA28_MAX + 1).unwrap_err();
        assert_eq!(
            err.diagnostic(),
            StorageDiagnostic::new(
                StorageBackend::AtaPio,
                StorageOperation::ReadSector,
                StorageErrorKind::LbaOutOfRange,
                STATUS_UNAVAILABLE,
                ATA_LBA28_MAX + 1,
                0,
                ACTIVE_NODE_NONE,
            )
        );
    }

    #[test]
    fn poll_returns_ready_when_drq_arrives() {
        let request = ReadSectorRequest::new(7).unwrap().with_active_node(42);
        let statuses = [ATA_STATUS_BSY, ATA_STATUS_DRQ];
        let mut i = 0usize;
        let ready = poll_ata_status(request, TimeoutBudget::new(4), || {
            let status = statuses[i];
            i += 1;
            status
        })
        .unwrap();

        assert_eq!(
            ready,
            ReadyStatus {
                status_register: ATA_STATUS_DRQ,
                polls: 2,
            }
        );
    }

    #[test]
    fn poll_reports_device_error_before_timeout() {
        let request = ReadSectorRequest::new(9).unwrap();
        let err = poll_ata_status(request, TimeoutBudget::new(4), || ATA_STATUS_ERR).unwrap_err();

        assert_eq!(
            err.diagnostic(),
            StorageDiagnostic::new(
                StorageBackend::AtaPio,
                StorageOperation::ReadSector,
                StorageErrorKind::DeviceError,
                ATA_STATUS_ERR,
                9,
                1,
                ACTIVE_NODE_NONE,
            )
        );
    }

    #[test]
    fn poll_timeout_is_finite_and_reports_last_status() {
        let request = ReadSectorRequest::new(11).unwrap().with_active_node(3);
        let err = poll_ata_status(request, TimeoutBudget::new(3), || ATA_STATUS_BSY).unwrap_err();

        assert_eq!(
            err.diagnostic(),
            StorageDiagnostic::new(
                StorageBackend::AtaPio,
                StorageOperation::ReadSector,
                StorageErrorKind::Timeout,
                ATA_STATUS_BSY,
                11,
                3,
                3,
            )
        );
    }

    #[test]
    fn zero_poll_budget_times_out_without_touching_status() {
        let request = ReadSectorRequest::new(13).unwrap();
        let mut called = false;
        let err = poll_ata_status(request, TimeoutBudget::new(0), || {
            called = true;
            ATA_STATUS_DRQ
        })
        .unwrap_err();

        assert!(!called);
        assert_eq!(
            err.diagnostic(),
            StorageDiagnostic::new(
                StorageBackend::AtaPio,
                StorageOperation::ReadSector,
                StorageErrorKind::Timeout,
                STATUS_UNAVAILABLE,
                13,
                0,
                ACTIVE_NODE_NONE,
            )
        );
    }

    #[test]
    fn write_support_is_not_enabled_in_stage_one() {
        assert!(!WRITE_SUPPORT_ENABLED);
    }

    #[test]
    fn ata_pio_read_sector_emits_command_sequence_and_reads_one_sector() {
        let mut ports = FakePorts {
            statuses: vec![ATA_STATUS_BSY, ATA_STATUS_DRQ],
            words: (0..SECTOR_WORDS as u16).collect(),
            ..FakePorts::default()
        };
        let request = ReadSectorRequest::new(0x0A12_3456).unwrap();
        let mut sector = [0u16; SECTOR_WORDS];

        let ready = ata_pio_read_sector_with_ports(
            &mut ports,
            request,
            TimeoutBudget::new(DEFAULT_ATA_TIMEOUT_POLLS),
            &mut sector,
        )
        .unwrap();

        assert_eq!(
            ports.writes,
            [
                (ATA_REG_DRIVE_HEAD, 0xEA),
                (ATA_REG_SECTOR_COUNT, 1),
                (ATA_REG_LBA_LOW, 0x56),
                (ATA_REG_LBA_MID, 0x34),
                (ATA_REG_LBA_HIGH, 0x12),
                (ATA_REG_STATUS_COMMAND, ATA_CMD_READ_SECTORS),
            ]
        );
        assert_eq!(
            ready,
            ReadyStatus {
                status_register: ATA_STATUS_DRQ,
                polls: 2,
            }
        );
        assert_eq!(ports.word_reads, SECTOR_WORDS);
        assert_eq!(sector[0], 0);
        assert_eq!(sector[255], 255);
    }

    #[test]
    fn ata_pio_error_does_not_read_data_port() {
        let mut ports = FakePorts {
            statuses: vec![ATA_STATUS_ERR],
            words: (0..SECTOR_WORDS as u16).collect(),
            ..FakePorts::default()
        };
        let request = ReadSectorRequest::new(5).unwrap();
        let mut sector = [0u16; SECTOR_WORDS];

        let err =
            ata_pio_read_sector_with_ports(&mut ports, request, TimeoutBudget::new(4), &mut sector)
                .unwrap_err();

        assert_eq!(err.diagnostic().kind, StorageErrorKind::DeviceError);
        assert_eq!(err.diagnostic().status_register, ATA_STATUS_ERR);
        assert_eq!(ports.word_reads, 0);
    }
}
