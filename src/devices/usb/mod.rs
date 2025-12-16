//! USB Drive specific erasure methods
//! 
//! USB drives typically use NAND flash memory and have limited
//! write endurance. Focus on efficient, single-pass methods.

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::process::Command;
use crate::advanced_wiper::{DeviceInfo, DeviceType, WipingProgress, WipingAlgorithm};
use crate::devices::DeviceEraser;

pub struct UsbEraser {
    buffer_size: usize,
    verify_after_wipe: bool,
    conservative_approach: bool,
}

impl UsbEraser {
    pub fn new() -> Self {
        Self {
            buffer_size: 512 * 1024, // 512KB buffer for USB (smaller to avoid timeout)
            verify_after_wipe: true,
            conservative_approach: true, // Protect USB drive lifespan
        }
    }
    
    pub fn with_buffer_size(buffer_size: usize) -> Self {
        Self {
            buffer_size,
            verify_after_wipe: true,
            conservative_approach: true,
        }
    }
    
    pub fn aggressive_mode() -> Self {
        Self {
            buffer_size: 1024 * 1024, // 1MB buffer
            verify_after_wipe: true,
            conservative_approach: false,
        }
    }
    
    /// Single-pass random erasure (recommended for USB drives)
    pub fn single_pass_random(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting single-pass random erasure for USB drive");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "Random".to_string();
        }
        
        let pattern = self.generate_random_pattern(self.buffer_size);
        self.overwrite_device(device_info, &pattern, progress_callback)?;
        
        println!("✅ Single-pass random erasure completed for USB drive");
        Ok(())
    }
    
    /// Single-pass zero fill
    pub fn single_pass_zeros(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting single-pass zero fill for USB drive");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "Zeros".to_string();
        }
        
        let pattern = vec![0u8; self.buffer_size];
        self.overwrite_device(device_info, &pattern, progress_callback)?;
        
        println!("✅ Single-pass zero fill completed for USB drive");
        Ok(())
    }
    
    /// Quick format + overwrite
    pub fn quick_format_overwrite(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting quick format + overwrite for USB drive");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 2;
            progress.current_pattern = "Quick Format".to_string();
        }
        
        // Step 1: Quick format
        self.quick_format(device_info)?;
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 2;
            progress.current_pattern = "Random Overwrite".to_string();
        }
        
        // Step 2: Overwrite with random data
        let pattern = self.generate_random_pattern(self.buffer_size);
        self.overwrite_device(device_info, &pattern, progress_callback)?;
        
        println!("✅ Quick format + overwrite completed for USB drive");
        Ok(())
    }
    
    /// Conservative 3-pass erasure (only if not in conservative mode)
    pub fn conservative_three_pass(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        if self.conservative_approach {
            println!("ℹ️  Conservative mode enabled, using single-pass instead");
            return self.single_pass_random(device_info, progress_callback);
        }
        
        println!("🔄 Starting 3-pass erasure for USB drive");
        
        let patterns = [
            vec![0x00; self.buffer_size], // Pass 1: Zeros
            vec![0xFF; self.buffer_size], // Pass 2: Ones
            self.generate_random_pattern(self.buffer_size), // Pass 3: Random
        ];
        
        for (pass, pattern) in patterns.iter().enumerate() {
            let pass_num = pass + 1;
            println!("🔄 USB Pass {}/3", pass_num);
            
            // Update progress
            if let Ok(mut progress) = progress_callback.lock() {
                progress.current_pass = pass_num as u32;
                progress.total_passes = 3;
                progress.current_pattern = match pass {
                    0 => "Zeros".to_string(),
                    1 => "Ones".to_string(),
                    2 => "Random".to_string(),
                    _ => "Unknown".to_string(),
                };
            }
            
            self.overwrite_device(device_info, pattern, progress_callback.clone())?;
            
            // Add delay between passes to prevent overheating
            if pass < patterns.len() - 1 {
                std::thread::sleep(Duration::from_secs(1));
            }
        }
        
        println!("✅ 3-pass erasure completed for USB drive");
        Ok(())
    }
    
    /// File-system level secure deletion
    pub fn filesystem_secure_delete(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting filesystem-level secure deletion for USB drive");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 3;
            progress.current_pattern = "File Deletion".to_string();
        }
        
        // Step 1: Delete all files
        self.delete_all_files(&device_info.device_path)?;
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 2;
            progress.current_pattern = "Free Space Fill".to_string();
        }
        
        // Step 2: Fill free space
        self.fill_free_space(&device_info.device_path, progress_callback.clone())?;
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 3;
            progress.current_pattern = "Cleanup".to_string();
        }
        
        // Step 3: Final cleanup
        self.cleanup_temp_files(&device_info.device_path)?;
        
        println!("✅ Filesystem-level secure deletion completed for USB drive");
        Ok(())
    }
    
    /// Quick format the USB drive
    fn quick_format(&self, device_info: &DeviceInfo) -> io::Result<()> {
        println!("🔧 Performing quick format...");
        
        // Extract drive letter from device path
        let drive_letter = self.extract_drive_letter(&device_info.device_path)?;
        
        // Use Windows format command
        let output = Command::new("format")
            .args(&[&format!("{}:", drive_letter), "/Q", "/Y"])
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ Quick format completed");
                    Ok(())
                } else {
                    let error_msg = String::from_utf8_lossy(&result.stderr);
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Format failed: {}", error_msg)
                    ))
                }
            }
            Err(e) => {
                println!("❌ Format command failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Delete all files on the drive
    fn delete_all_files(&self, _device_path: &str) -> io::Result<()> {
        println!("🗑️  Deleting all files...");
        
        // This would recursively delete all files and directories
        // For now, return error as it is not implemented
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Recursive file deletion not implemented"
        ))
    }
    
    /// Fill free space with random data
    fn fill_free_space(
        &self,
        device_path: &str,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔧 Filling free space...");
        
        let drive_letter = self.extract_drive_letter(device_path)?;
        let fill_file_path = format!("{}:\\temp_fill_file.tmp", drive_letter);
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&fill_file_path)?;
        
        let pattern = self.generate_random_pattern(self.buffer_size);
        let mut bytes_written = 0u64;
        let start_time = Instant::now();
        
        // Keep writing until disk is full
        loop {
            match file.write_all(&pattern) {
                Ok(_) => {
                    bytes_written += pattern.len() as u64;
                    
                    // Update progress periodically
                    if bytes_written % (10 * 1024 * 1024) == 0 { // Update every 10MB
                        if let Ok(mut progress) = progress_callback.lock() {
                            progress.bytes_processed = bytes_written;
                            
                            let elapsed = start_time.elapsed();
                            if elapsed.as_secs() > 0 {
                                progress.speed_mbps = (bytes_written as f64) / (1024.0 * 1024.0) / elapsed.as_secs_f64();
                            }
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::StorageFull => {
                    println!("✅ Free space filled ({} bytes)", bytes_written);
                    break;
                }
                Err(e) => {
                    println!("❌ Error filling free space: {}", e);
                    break;
                }
            }
        }
        
        // Close and delete the temporary file
        drop(file);
        let _ = std::fs::remove_file(&fill_file_path);
        
        Ok(())
    }
    
    /// Clean up temporary files
    fn cleanup_temp_files(&self, _device_path: &str) -> io::Result<()> {
        println!("🧹 Cleaning up temporary files...");
        
        // This would clean up any remaining temporary files
        // For now, return error as it is not implemented
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Cleanup temp files not implemented"
        ))
    }
    
    /// Extract drive letter from device path
    fn extract_drive_letter(&self, device_path: &str) -> io::Result<String> {
        // Simple extraction for Windows drive letters
        if device_path.len() >= 1 {
            let first_char = device_path.chars().next().unwrap();
            if first_char.is_alphabetic() {
                return Ok(first_char.to_string().to_uppercase());
            }
        }
        
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not extract drive letter"
        ))
    }
    
    /// Overwrite device with specific pattern (USB-optimized)
    fn overwrite_device(
        &self,
        device_info: &DeviceInfo,
        pattern: &[u8],
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        let start_time = Instant::now();
        let mut file = OpenOptions::new()
            .write(true)
            .open(&device_info.device_path)?;
        
        let total_size = device_info.size_bytes;
        let mut bytes_written = 0u64;
        
        file.seek(SeekFrom::Start(0))?;
        
        // Use smaller chunks for USB drives to avoid timeouts
        let chunk_size = std::cmp::min(self.buffer_size, 256 * 1024); // Max 256KB chunks
        let pattern_chunk = self.expand_pattern(pattern, chunk_size);
        
        while bytes_written < total_size {
            let remaining = total_size - bytes_written;
            let write_size = std::cmp::min(pattern_chunk.len() as u64, remaining) as usize;
            
            file.write_all(&pattern_chunk[..write_size])?;
            bytes_written += write_size as u64;
            
            // Sync more frequently for USB drives
            if bytes_written % (10 * 1024 * 1024) == 0 {
                file.sync_data()?;
            }
            
            // Update progress
            if let Ok(mut progress) = progress_callback.lock() {
                progress.bytes_processed = bytes_written;
                progress.total_bytes = total_size;
                
                let elapsed = start_time.elapsed();
                if elapsed.as_secs() > 0 {
                    progress.speed_mbps = (bytes_written as f64) / (1024.0 * 1024.0) / elapsed.as_secs_f64();
                    
                    if bytes_written > 0 {
                        let estimated_total_time = elapsed.as_secs_f64() * (total_size as f64) / (bytes_written as f64);
                        progress.estimated_time_remaining = Duration::from_secs_f64(estimated_total_time - elapsed.as_secs_f64());
                    }
                }
            }
            
            // Small delay to prevent overheating USB drive
            if bytes_written % (50 * 1024 * 1024) == 0 {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        
        file.sync_all()?;
        Ok(())
    }
    
    /// Generate random pattern
    fn generate_random_pattern(&self, size: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..size).map(|_| rng.r#gen::<u8>()).collect()
    }
    
    /// Expand pattern to specified size
    fn expand_pattern(&self, pattern: &[u8], size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(size);
        let pattern_len = pattern.len();
        for i in 0..size {
            result.push(pattern[i % pattern_len]);
        }
        result
    }
    
    /// Detect USB drive capabilities
    fn detect_usb_capabilities(&self, device_path: &str) -> (bool, bool) {
        // USB drives typically don't support hardware secure erase
        // but may support TRIM (depends on controller)
        let supports_secure_erase = false;
        let supports_trim = false; // Conservative assumption
        
        (supports_secure_erase, supports_trim)
    }
}

impl DeviceEraser for UsbEraser {
    fn analyze_device(&self, device_path: &str) -> io::Result<DeviceInfo> {
        println!("🔍 Analyzing USB drive: {}", device_path);
        
        let (supports_secure_erase, supports_trim) = self.detect_usb_capabilities(device_path);
        
        // Try to get basic device info
        let device_info = match File::open(device_path) {
            Ok(file) => {
                let metadata = file.metadata()?;
                DeviceInfo {
                    device_path: device_path.to_string(),
                    device_type: DeviceType::USBDrive,
                    size_bytes: metadata.len(),
                    sector_size: 512, // Standard for most USB drives
                    supports_trim,
                    supports_secure_erase,
                    supports_enhanced_secure_erase: false,
                    supports_crypto_erase: false, // Rare in USB drives
                    is_removable: true,
                    vendor: "Unknown".to_string(),
                    model: "Unknown USB Drive".to_string(),
                    serial: "Unknown".to_string(),
                }
            }
            Err(e) => return Err(e),
        };
        
        println!("✅ USB drive analysis complete: {} ({} bytes)", 
                device_info.model, device_info.size_bytes);
        Ok(device_info)
    }
    
    fn erase_device(
        &self,
        device_info: &DeviceInfo,
        algorithm: WipingAlgorithm,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🚀 Starting USB drive erasure with algorithm: {:?}", algorithm);
        
        match algorithm {
            WipingAlgorithm::Random => self.single_pass_random(device_info, progress_callback),
            WipingAlgorithm::Zeros => self.single_pass_zeros(device_info, progress_callback),
            WipingAlgorithm::QuickFormat => self.quick_format_overwrite(device_info, progress_callback),
            WipingAlgorithm::ThreePass => self.conservative_three_pass(device_info, progress_callback),
            WipingAlgorithm::FileSystemWipe => self.filesystem_secure_delete(device_info, progress_callback),
            WipingAlgorithm::NistClear => self.single_pass_zeros(device_info, progress_callback),
            WipingAlgorithm::Ones => {
                let pattern = vec![0xFFu8; self.buffer_size];
                self.overwrite_device(device_info, &pattern, progress_callback)
            },
            _ => {
                // Default to single-pass random for USB drives (preserves lifespan)
                println!("ℹ️  Using single-pass random as default for USB drive");
                self.single_pass_random(device_info, progress_callback)
            }
        }
    }
    
    fn verify_erasure(&self, device_info: &DeviceInfo) -> io::Result<bool> {
        if !self.verify_after_wipe {
            return Ok(true);
        }
        
        println!("🔍 Verifying USB drive erasure...");
        
        let mut file = File::open(&device_info.device_path)?;
        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_read = 0u64;
        // For USB drives, sample conservatively to avoid wear
        let sample_size = std::cmp::min(device_info.size_bytes, 50 * 1024 * 1024); // Sample first 50MB
        
        while total_read < sample_size {
            let bytes_read = std::io::Read::read(&mut file, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            // Check for non-zero bytes
            if buffer[..bytes_read].iter().any(|&b| b != 0) {
                println!("⚠️  Found non-zero data during USB drive verification");
                return Ok(false);
            }
            
            total_read += bytes_read as u64;
        }
        
        println!("✅ USB drive erasure verification passed");
        Ok(true)
    }
    
    fn get_recommended_algorithms(&self) -> Vec<WipingAlgorithm> {
        if self.conservative_approach {
            vec![
                WipingAlgorithm::Random,           // Primary choice (single pass)
                WipingAlgorithm::Zeros,            // Simple zero fill
                WipingAlgorithm::FileSystemWipe,   // File-level erasure
                WipingAlgorithm::QuickFormat,      // Quick format + overwrite
                WipingAlgorithm::NistClear,        // NIST approved single pass
            ]
        } else {
            vec![
                WipingAlgorithm::ThreePass,        // 3-pass for higher security
                WipingAlgorithm::Random,           // Single-pass random
                WipingAlgorithm::FileSystemWipe,   // File-level erasure
                WipingAlgorithm::QuickFormat,      // Quick format + overwrite
                WipingAlgorithm::NistClear,        // NIST approved method
            ]
        }
    }
}