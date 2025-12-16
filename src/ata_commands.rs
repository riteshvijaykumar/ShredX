/*!
 * ATA Command Interface for Low-Level Drive Operations
 * 
 * This module provides low-level ATA command functionality required for:
 * - HPA (Host Protected Area) detection and manipulation
 * - DCO (Device Configuration Overlay) analysis
 * - Security feature assessment
 * - Native capacity determination
 * 
 * ⚠️ WARNING: These commands directly interface with drive hardware.
 * Improper usage can result in data loss or drive damage.
 */

// ATA command interface for low-level drive operations
// Required for HPA/DCO detection and manipulation

use std::io;
use std::mem;

// Platform-specific imports
#[cfg(windows)]
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::IO::DeviceIoControl,
    },
};

#[cfg(unix)]
use {
    std::fs::File,
    std::os::unix::io::{AsRawFd, RawFd},
    libc::{ioctl, c_int, c_ulong},
};

// ============================================================================
// ATA COMMAND CODES AND CONSTANTS
// ============================================================================

// ATA command codes
/// ATA IDENTIFY DEVICE command (0xEC)
pub const ATA_IDENTIFY_DEVICE: u8 = 0xEC;
/// ATA READ NATIVE MAX ADDRESS command (0xF8) - 28-bit
pub const ATA_READ_NATIVE_MAX_ADDRESS: u8 = 0xF8;
/// ATA READ NATIVE MAX ADDRESS EXT command (0x27) - 48-bit
pub const ATA_READ_NATIVE_MAX_ADDRESS_EXT: u8 = 0x27;
/// ATA SET MAX ADDRESS command (0xF9) - 28-bit
pub const ATA_SET_MAX_ADDRESS: u8 = 0xF9;
/// ATA SET MAX ADDRESS EXT command (0x37) - 48-bit
pub const ATA_SET_MAX_ADDRESS_EXT: u8 = 0x37;
/// ATA SECURITY SET PASSWORD command (0xF1)
pub const ATA_SECURITY_SET_PASSWORD: u8 = 0xF1;
/// ATA SECURITY UNLOCK command (0xF2)
pub const ATA_SECURITY_UNLOCK: u8 = 0xF2;
/// ATA SECURITY ERASE PREPARE command (0xF3)
pub const ATA_SECURITY_ERASE_PREPARE: u8 = 0xF3;
/// ATA SECURITY ERASE UNIT command (0xF4)
pub const ATA_SECURITY_ERASE_UNIT: u8 = 0xF4;
/// ATA SECURITY FREEZE LOCK command (0xF5)
pub const ATA_SECURITY_FREEZE_LOCK: u8 = 0xF5;
/// ATA SECURITY DISABLE PASSWORD command (0xF6)
pub const ATA_SECURITY_DISABLE_PASSWORD: u8 = 0xF6;

// ============================================================================
// WINDOWS IOCTL CODES
// ============================================================================

// IOCTL codes for Windows
/// IOCTL for ATA pass-through commands
const IOCTL_ATA_PASS_THROUGH: u32 = 0x0004D02C;
/// IOCTL for ATA pass-through direct commands
const IOCTL_ATA_PASS_THROUGH_DIRECT: u32 = 0x0004D030;

// ============================================================================
// ATA DATA STRUCTURES
// ============================================================================

/// ATA Pass-Through Extended structure for Windows IOCTL
#[repr(C)]
pub struct AtaPassThroughEx {
    pub length: u16,
    pub ata_flags: u16,
    pub path_id: u8,
    pub target_id: u8,
    pub lun: u8,
    pub reserved_as_uchar: u8,
    pub data_transfer_length: u32,
    pub timeout_value: u32,
    pub reserved_as_ulong: u32,
    pub data_buffer_offset: usize,
    pub previous_task_file: [u8; 8],
    pub current_task_file: [u8; 8],
}

/// ATA IDENTIFY DEVICE data structure (512 bytes)
#[repr(C)]
pub struct IdentifyDeviceData {
    /// Raw identify data (256 words = 512 bytes)
    pub data: [u16; 256],
}

/// Comprehensive drive information extracted from ATA commands
#[derive(Debug)]
pub struct DriveInfo {
    /// Drive model string
    pub model: String,
    /// Drive serial number
    pub serial: String,
    /// Firmware revision
    pub firmware: String,
    /// User-addressable capacity in bytes
    pub user_capacity: u64,
    /// Native (maximum) capacity in bytes
    pub native_capacity: u64,
    /// Whether Host Protected Area is present
    pub has_hpa: bool,
    /// Whether Device Configuration Overlay is present
    pub has_dco: bool,
    /// Whether ATA Security feature set is supported
    pub security_supported: bool,
    /// Whether security is currently enabled
    pub security_enabled: bool,
    /// Whether drive is security locked
    pub security_locked: bool,
    /// Whether security is frozen (requires power cycle to unlock)
    pub security_frozen: bool,
    /// Drive type description
    pub drive_type: String,
}

// ============================================================================
// ATA INTERFACE IMPLEMENTATION
// ============================================================================

/// Low-level ATA command interface (cross-platform)
pub struct AtaInterface {
    #[cfg(windows)]
    handle: HANDLE,
    #[cfg(unix)]
    file: File,
}

impl AtaInterface {
    pub fn new(drive_path: &str) -> io::Result<Self> {
        #[cfg(windows)]
        {
            unsafe {
                let drive_path_wide: Vec<u16> = drive_path.encode_utf16().chain(std::iter::once(0)).collect();
                let drive_path_pwstr = PWSTR::from_raw(drive_path_wide.as_ptr() as *mut u16);

                let handle = CreateFileW(
                    drive_path_pwstr,
                    0x40000000u32 | 0x80000000u32, // GENERIC_READ | GENERIC_WRITE
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    HANDLE::default(),
                ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open drive: {}", e)))?;

                Ok(AtaInterface { handle })
            }
        }
        
        #[cfg(unix)]
        {
            let file = File::open(drive_path)?;
            Ok(AtaInterface { file })
        }
    }

    pub fn identify_device(&self) -> io::Result<IdentifyDeviceData> {
        let mut identify_data = IdentifyDeviceData { data: [0; 256] };
        
        let mut ata_pt = AtaPassThroughEx {
            length: mem::size_of::<AtaPassThroughEx>() as u16,
            ata_flags: 0x02, // ATA_FLAGS_DATA_IN
            path_id: 0,
            target_id: 0,
            lun: 0,
            reserved_as_uchar: 0,
            data_transfer_length: 512,
            timeout_value: 30,
            reserved_as_ulong: 0,
            data_buffer_offset: mem::size_of::<AtaPassThroughEx>(),
            previous_task_file: [0; 8],
            current_task_file: [0; 8],
        };

        // Set up the command
        ata_pt.current_task_file[6] = ATA_IDENTIFY_DEVICE;

        let mut bytes_returned = 0u32;
        let mut buffer = vec![0u8; mem::size_of::<AtaPassThroughEx>() + 512];
        
        unsafe {
            // Copy the ATA_PASS_THROUGH_EX structure to buffer
            let ata_pt_bytes = std::slice::from_raw_parts(
                &ata_pt as *const _ as *const u8,
                mem::size_of::<AtaPassThroughEx>()
            );
            buffer[..mem::size_of::<AtaPassThroughEx>()].copy_from_slice(ata_pt_bytes);

            let success = DeviceIoControl(
                self.handle,
                IOCTL_ATA_PASS_THROUGH,
                Some(buffer.as_ptr() as *const _),
                buffer.len() as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            );

            if success.is_err() {
                return Err(io::Error::new(io::ErrorKind::Other, "IDENTIFY DEVICE command failed"));
            }

            // Copy data from buffer to identify_data
            let data_start = mem::size_of::<AtaPassThroughEx>();
            let data_bytes = &buffer[data_start..data_start + 512];
            let data_words = std::slice::from_raw_parts(data_bytes.as_ptr() as *const u16, 256);
            identify_data.data.copy_from_slice(data_words);
        }

        Ok(identify_data)
    }

    pub fn read_native_max_address(&self, use_ext: bool) -> io::Result<u64> {
        let mut ata_pt = AtaPassThroughEx {
            length: mem::size_of::<AtaPassThroughEx>() as u16,
            ata_flags: 0x02, // ATA_FLAGS_DATA_IN
            path_id: 0,
            target_id: 0,
            lun: 0,
            reserved_as_uchar: 0,
            data_transfer_length: 0,
            timeout_value: 30,
            reserved_as_ulong: 0,
            data_buffer_offset: 0,
            previous_task_file: [0; 8],
            current_task_file: [0; 8],
        };

        // Set up the command
        if use_ext {
            ata_pt.current_task_file[6] = ATA_READ_NATIVE_MAX_ADDRESS_EXT;
        } else {
            ata_pt.current_task_file[6] = ATA_READ_NATIVE_MAX_ADDRESS;
        }

        let mut bytes_returned = 0u32;
        let mut buffer = vec![0u8; mem::size_of::<AtaPassThroughEx>()];
        
        unsafe {
            let ata_pt_bytes = std::slice::from_raw_parts(
                &ata_pt as *const _ as *const u8,
                mem::size_of::<AtaPassThroughEx>()
            );
            buffer.copy_from_slice(ata_pt_bytes);

            let success = DeviceIoControl(
                self.handle,
                IOCTL_ATA_PASS_THROUGH,
                Some(buffer.as_ptr() as *const _),
                buffer.len() as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            );

            if success.is_err() {
                return Err(io::Error::new(io::ErrorKind::Other, "READ NATIVE MAX ADDRESS command failed"));
            }

            // Extract result from task file registers
            let result_ata_pt = &*(buffer.as_ptr() as *const AtaPassThroughEx);
            let lba = if use_ext {
                // 48-bit LBA
                let lba_low = result_ata_pt.current_task_file[3] as u64;
                let lba_mid = result_ata_pt.current_task_file[4] as u64;
                let lba_high = result_ata_pt.current_task_file[5] as u64;
                let lba_low_prev = result_ata_pt.previous_task_file[3] as u64;
                let lba_mid_prev = result_ata_pt.previous_task_file[4] as u64;
                let lba_high_prev = result_ata_pt.previous_task_file[5] as u64;
                
                lba_low | (lba_mid << 8) | (lba_high << 16) | 
                (lba_low_prev << 24) | (lba_mid_prev << 32) | (lba_high_prev << 40)
            } else {
                // 28-bit LBA
                let lba_low = result_ata_pt.current_task_file[3] as u64;
                let lba_mid = result_ata_pt.current_task_file[4] as u64;
                let lba_high = result_ata_pt.current_task_file[5] as u64;
                let device = result_ata_pt.current_task_file[6] as u64;
                
                lba_low | (lba_mid << 8) | (lba_high << 16) | ((device & 0x0F) << 24)
            };

            Ok(lba)
        }
    }

    pub fn set_max_address(&self, lba: u64, use_ext: bool) -> io::Result<()> {
        let mut ata_pt = AtaPassThroughEx {
            length: mem::size_of::<AtaPassThroughEx>() as u16,
            ata_flags: 0x00, // No data transfer
            path_id: 0,
            target_id: 0,
            lun: 0,
            reserved_as_uchar: 0,
            data_transfer_length: 0,
            timeout_value: 30,
            reserved_as_ulong: 0,
            data_buffer_offset: 0,
            previous_task_file: [0; 8],
            current_task_file: [0; 8],
        };

        // Set up the command and LBA
        if use_ext {
            ata_pt.current_task_file[6] = ATA_SET_MAX_ADDRESS_EXT;
            // 48-bit LBA
            ata_pt.current_task_file[3] = (lba & 0xFF) as u8;
            ata_pt.current_task_file[4] = ((lba >> 8) & 0xFF) as u8;
            ata_pt.current_task_file[5] = ((lba >> 16) & 0xFF) as u8;
            ata_pt.previous_task_file[3] = ((lba >> 24) & 0xFF) as u8;
            ata_pt.previous_task_file[4] = ((lba >> 32) & 0xFF) as u8;
            ata_pt.previous_task_file[5] = ((lba >> 40) & 0xFF) as u8;
        } else {
            ata_pt.current_task_file[6] = ATA_SET_MAX_ADDRESS;
            // 28-bit LBA
            ata_pt.current_task_file[3] = (lba & 0xFF) as u8;
            ata_pt.current_task_file[4] = ((lba >> 8) & 0xFF) as u8;
            ata_pt.current_task_file[5] = ((lba >> 16) & 0xFF) as u8;
            ata_pt.current_task_file[6] |= ((lba >> 24) & 0x0F) as u8;
        }

        let mut bytes_returned = 0u32;
        let mut buffer = vec![0u8; mem::size_of::<AtaPassThroughEx>()];
        
        unsafe {
            let ata_pt_bytes = std::slice::from_raw_parts(
                &ata_pt as *const _ as *const u8,
                mem::size_of::<AtaPassThroughEx>()
            );
            buffer.copy_from_slice(ata_pt_bytes);

            let success = DeviceIoControl(
                self.handle,
                IOCTL_ATA_PASS_THROUGH,
                Some(buffer.as_ptr() as *const _),
                buffer.len() as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            );

            if success.is_err() {
                return Err(io::Error::new(io::ErrorKind::Other, "SET MAX ADDRESS command failed"));
            }
        }

        Ok(())
    }

    pub fn parse_identify_data(&self, data: &IdentifyDeviceData) -> DriveInfo {
        let words = &data.data;
        
        // Extract strings (ATA strings are word-swapped)
        let model = Self::extract_ata_string(&words[27..47]);
        let serial = Self::extract_ata_string(&words[10..20]);
        let firmware = Self::extract_ata_string(&words[23..27]);
        
        // User addressable capacity
        let user_capacity = if words[83] & 0x0400 != 0 {
            // 48-bit addressing
            ((words[103] as u64) << 48) | ((words[102] as u64) << 32) | 
            ((words[101] as u64) << 16) | (words[100] as u64)
        } else {
            // 28-bit addressing
            ((words[61] as u64) << 16) | (words[60] as u64)
        } * 512; // Convert sectors to bytes

        // Security features
        let security_word = words[128];
        let security_supported = security_word & 0x0001 != 0;
        let security_enabled = security_word & 0x0002 != 0;
        let security_locked = security_word & 0x0004 != 0;
        let security_frozen = security_word & 0x0008 != 0;

        DriveInfo {
            model,
            serial,
            firmware,
            user_capacity,
            native_capacity: 0, // Will be filled by READ NATIVE MAX ADDRESS
            has_hpa: false,    // Will be determined by comparing capacities
            has_dco: false,    // Will be determined by DCO detection
            security_supported,
            security_enabled,
            security_locked,
            security_frozen,
            drive_type: "Unknown".to_string(), // Will be determined by drive detection
        }
    }

    fn extract_ata_string(words: &[u16]) -> String {
        let mut bytes = Vec::new();
        for &word in words {
            bytes.push((word >> 8) as u8);
            bytes.push((word & 0xFF) as u8);
        }
        
        // Remove trailing spaces and null bytes
        while let Some(&last) = bytes.last() {
            if last == 0 || last == b' ' {
                bytes.pop();
            } else {
                break;
            }
        }
        
        String::from_utf8_lossy(&bytes).into_owned()
    }
    
    /// Get drive information (convenience method that combines identify and parse)
    pub fn get_drive_info(&self) -> io::Result<DriveInfo> {
        let identify_data = self.identify_device()?;
        Ok(self.parse_identify_data(&identify_data))
    }
    
    /// Perform ATA Security Erase
    pub fn security_erase(&self, enhanced: bool) -> io::Result<()> {
        // This is a simplified implementation
        // In a real implementation, this would:
        // 1. Check if security is supported
        // 2. Set a temporary password
        // 3. Issue the security erase command
        // 4. Wait for completion
        
        println!("🔧 Performing ATA Security Erase (Enhanced: {})", enhanced);
        
        // Return error to force fallback to software overwrite
        // This is safer than simulating success without actually erasing data
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "ATA Security Erase not fully implemented. Falling back to software overwrite."
        ))
    }
}

impl Drop for AtaInterface {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle).ok();
        }
    }
}