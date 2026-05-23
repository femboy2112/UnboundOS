//! Storage stage 1 contracts — spec §7.1–§7.3 and §13.8.
//!
//! M6 starts with raw-sector reads and finite polling. Real ATA PIO port I/O is
//! intentionally deferred to the next step; this module defines the bounded
//! diagnostic and timeout surface that unsafe hardware access must use.

pub const SECTOR_WORDS: u32 = 256;
pub const SECTOR_BYTES: u32 = SECTOR_WORDS * 2;
pub const WRITE_SUPPORT_ENABLED: bool = false;
pub const ACTIVE_NODE_NONE: u32 = u32::MAX;
pub const STATUS_UNAVAILABLE: u8 = 0xFF;

pub const ATA_STATUS_ERR: u8 = 0x01;
pub const ATA_STATUS_DRQ: u8 = 0x08;
pub const ATA_STATUS_DF: u8 = 0x20;
pub const ATA_STATUS_BSY: u8 = 0x80;

pub const ATA_LBA28_MAX: u64 = 0x0FFF_FFFF;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
