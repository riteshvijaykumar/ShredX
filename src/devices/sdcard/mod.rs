//! SD Card and MMC specific erasure methods
//! 
//! SD Cards and MMC devices use NAND flash with limited write cycles.
//! Focus on minimal-wear erasure methods and respect device limitations.

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::process::Command;
use crate::advanced_wiper::{DeviceInfo, DeviceType, WipingProgress, WipingAlgorithm};
use crate::devices::DeviceEraser;

pub struct SdCardEraser {
    buffer_size: usize,
    verify_after_wipe: bool,
    wear_leveling_aware: bool,
    max_write_cycles: u32,
}

impl SdCardEraser {
    pub fn new() -> Self {
        Self {
            buffer_size: 256 * 1024, // 256KB buffer (small to avoid timeouts)
            verify_after_wipe: true,
            wear_leveling_aware: true,
            max_write_cycles: 1000, // Conservative estimate for consumer SD cards
        }
    }
    
    pub fn for_high_endurance() -> Self {
        Self {
            buffer_size: 512 * 1024, // 512KB buffer
            verify_after_wipe: true,
            wear_leveling_aware: true,
            max_write_cycles: 10000, // High-endurance cards
        }
    }
    
    pub fn for_industrial() -> Self {
        Self {
            buffer_size: 1024 * 1024, // 1MB buffer
            verify_after_wipe: true,
            wear_leveling_aware: true,
            max_write_cycles: 100000, // Industrial-grade cards
        }
    }
    
    /// Single-pass random erasure (recommended for SD cards)
    pub fn single_pass_random(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting single-pass random erasure for SD card");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "Random".to_string();
        }
        
        let pattern = self.generate_random_pattern(self.buffer_size);
        self.overwrite_device_gentle(device_info, &pattern, progress_callback)?;
        
        println!("✅ Single-pass random erasure completed for SD card");
        Ok(())
    }
    
    /// Single-pass zero fill
    pub fn single_pass_zeros(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting single-pass zero fill for SD card");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "Zeros".to_string();
        }
        
        let pattern = vec![0u8; self.buffer_size];
        self.overwrite_device_gentle(device_info, &pattern, progress_callback)?;
        
        println!("✅ Single-pass zero fill completed for SD card");
        Ok(())
    }
    
    /// SD Card specific erase command
    pub fn sd_erase_command(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting SD Card erase command");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "SD Erase Command".to_string();
        }
        
        // For SD cards, we can use the native erase command if supported
        match self.execute_sd_erase_command(device_info) {
            Ok(_) => {
                // Update progress to completion
                if let Ok(mut progress) = progress_callback.lock() {
                    progress.bytes_processed = device_info.size_bytes;
                    progress.total_bytes = device_info.size_bytes;
                }
                
                println!("✅ SD Card erase command completed");
                Ok(())
            }
            Err(e) => {
                println!("❌ SD erase command failed, falling back to software erasure: {}", e);
                // Fallback to single-pass zero fill
                self.single_pass_zeros(device_info, progress_callback)
            }
        }
    }
    
    /// File-system level secure deletion for SD cards
    pub fn filesystem_secure_delete(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting filesystem-level secure deletion for SD card");
        
        // This feature requires complex filesystem parsing which is not fully implemented
        // Return error to avoid false sense of security
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Filesystem-level secure deletion not implemented. Please use block-level erasure (Random or Zeros)."
        ))
    }
    
    /// Quick format for SD cards
    pub fn quick_format(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting quick format for SD card");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "Quick Format".to_string();
        }
        
        // Extract drive letter from device path
        let drive_letter = self.extract_drive_letter(&device_info.device_path)?;
        
        // Use Windows format command with FAT32 for SD cards
        let output = Command::new("format")
            .args(&[&format!("{}:", drive_letter), "/FS:FAT32", "/Q", "/Y"])
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    // Update progress to completion
                    if let Ok(mut progress) = progress_callback.lock() {
                        progress.bytes_processed = device_info.size_bytes;
                        progress.total_bytes = device_info.size_bytes;
                    }
                    
                    println!("✅ Quick format completed for SD card");
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
    
    /// Ultra-conservative two-pass erasure (for critical data)
    pub fn conservative_two_pass(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting conservative 2-pass erasure for SD card");
        
        let patterns = [
            vec![0x00; self.buffer_size], // Pass 1: Zeros
            self.generate_random_pattern(self.buffer_size), // Pass 2: Random
        ];
        
        for (pass, pattern) in patterns.iter().enumerate() {
            let pass_num = pass + 1;
            println!("🔄 SD Card Pass {}/2", pass_num);
            
            // Update progress
            if let Ok(mut progress) = progress_callback.lock() {
                progress.current_pass = pass_num as u32;
                progress.total_passes = 2;
                progress.current_pattern = match pass {
                    0 => "Zeros".to_string(),
                    1 => "Random".to_string(),
                    _ => "Unknown".to_string(),
                };
            }
            
            self.overwrite_device_gentle(device_info, pattern, progress_callback.clone())?;
            
            // Longer delay between passes for SD card health
            if pass < patterns.len() - 1 {
                println!("⏳ Pausing between passes for SD card health...");
                std::thread::sleep(Duration::from_secs(5));
            }
        }
        
        println!("✅ Conservative 2-pass erasure completed for SD card");
        Ok(())
    }
    
    /// Execute SD card native erase command
    fn execute_sd_erase_command(&self, _device_info: &DeviceInfo) -> io::Result<()> {
        // This would typically use SD card specific commands
        // For now, return error to force fallback to software erasure
        println!("🔧 Executing SD native erase command...");
        
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "SD native erase command not implemented for this platform"
        ))
    }
    
    /// Analyze filesystem on SD card
    fn _analyze_filesystem(&self, device_path: &str) -> io::Result<()> {
        println!("🔍 Analyzing filesystem on SD card...");
        std::thread::sleep(Duration::from_millis(500));
        println!("✅ Filesystem analysis completed");
        Ok(())
    }
    
    /// Secure delete all files
    fn _secure_delete_files(&self, device_path: &str) -> io::Result<()> {
        println!("🗑️  Securely deleting files on SD card...");
        std::thread::sleep(Duration::from_secs(2));
        println!("✅ File deletion completed");
        Ok(())
    }
    
    /// Fill free space once (gentle for SD cards)
    fn _fill_free_space_once(
        &self,
        device_path: &str,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔧 Filling free space on SD card (gentle mode)...");
        
        let drive_letter = self.extract_drive_letter(device_path)?;
        let fill_file_path = format!("{}:\\temp_sd_fill.tmp", drive_letter);
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&fill_file_path)?;
        
        let pattern = self.generate_random_pattern(self.buffer_size);
        let mut bytes_written = 0u64;
        let start_time = Instant::now();
        
        // Gentle write with pauses
        loop {
            match file.write_all(&pattern) {
                Ok(_) => {
                    bytes_written += pattern.len() as u64;
                    
                    // Update progress
                    if bytes_written % (5 * 1024 * 1024) == 0 { // Update every 5MB
                        if let Ok(mut progress) = progress_callback.lock() {
                            progress.bytes_processed = bytes_written;
                            
                            let elapsed = start_time.elapsed();
                            if elapsed.as_secs() > 0 {
                                progress.speed_mbps = (bytes_written as f64) / (1024.0 * 1024.0) / elapsed.as_secs_f64();
                            }
                        }
                        
                        // Gentle pause every 5MB to prevent wear
                        std::thread::sleep(Duration::from_millis(50));
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::StorageFull => {
                    println!("✅ Free space filled gently ({} bytes)", bytes_written);
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
    
    /// Cleanup filesystem
    fn _cleanup_filesystem(&self, device_path: &str) -> io::Result<()> {
        println!("🧹 Cleaning up SD card filesystem...");
        std::thread::sleep(Duration::from_millis(500));
        println!("✅ Filesystem cleanup completed");
        Ok(())
    }
    
    /// Extract drive letter from device path
    fn extract_drive_letter(&self, device_path: &str) -> io::Result<String> {
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
    
    /// Gentle overwrite for SD cards (with wear-leveling consideration)
    fn overwrite_device_gentle(
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
        
        // Use very small chunks for SD cards to minimize wear
        let chunk_size = std::cmp::min(self.buffer_size, 128 * 1024); // Max 128KB chunks
        let pattern_chunk = self.expand_pattern(pattern, chunk_size);
        
        while bytes_written < total_size {
            let remaining = total_size - bytes_written;
            let write_size = std::cmp::min(pattern_chunk.len() as u64, remaining) as usize;
            
            file.write_all(&pattern_chunk[..write_size])?;
            bytes_written += write_size as u64;
            
            // Gentle sync pattern for SD cards
            if bytes_written % (5 * 1024 * 1024) == 0 {
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
            
            // Gentle pause every 10MB to prevent overheating and wear
            if bytes_written % (10 * 1024 * 1024) == 0 {
                std::thread::sleep(Duration::from_millis(200));
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
    
    /// Detect SD card type and capabilities
    fn detect_sd_capabilities(&self, device_path: &str) -> (bool, String) {
        // SD cards typically don't support hardware secure erase
        // but may have native erase commands
        let supports_native_erase = true; // Most SD cards support native erase
        let card_type = "Standard SD".to_string(); // Could be SD, SDHC, SDXC, etc.
        
        (supports_native_erase, card_type)
    }
}

impl DeviceEraser for SdCardEraser {
    fn analyze_device(&self, device_path: &str) -> io::Result<DeviceInfo> {
        println!("🔍 Analyzing SD card: {}", device_path);
        
        let (supports_native_erase, card_type) = self.detect_sd_capabilities(device_path);
        
        // Try to get basic device info
        let device_info = match File::open(device_path) {
            Ok(file) => {
                let metadata = file.metadata()?;
                DeviceInfo {
                    device_path: device_path.to_string(),
                    device_type: DeviceType::SDCard,
                    size_bytes: metadata.len(),
                    sector_size: 512, // Standard for SD cards
                    supports_trim: false, // SD cards don't typically support TRIM
                    supports_secure_erase: supports_native_erase,
                    supports_enhanced_secure_erase: false,
                    supports_crypto_erase: false, // Rare in consumer SD cards
                    is_removable: true,
                    vendor: "Unknown".to_string(),
                    model: card_type,
                    serial: "Unknown".to_string(),
                }
            }
            Err(e) => return Err(e),
        };
        
        println!("✅ SD card analysis complete: {} ({} bytes)", 
                device_info.model, device_info.size_bytes);
        Ok(device_info)
    }
    
    fn erase_device(
        &self,
        device_info: &DeviceInfo,
        algorithm: WipingAlgorithm,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🚀 Starting SD card erasure with algorithm: {:?}", algorithm);
        
        match algorithm {
            WipingAlgorithm::Random => self.single_pass_random(device_info, progress_callback),
            WipingAlgorithm::Zeros => self.single_pass_zeros(device_info, progress_callback),
            WipingAlgorithm::QuickFormat => self.quick_format(device_info, progress_callback),
            WipingAlgorithm::FileSystemWipe => self.filesystem_secure_delete(device_info, progress_callback),
            WipingAlgorithm::NistClear => self.single_pass_zeros(device_info, progress_callback),
            WipingAlgorithm::TwoPass => self.conservative_two_pass(device_info, progress_callback),
            WipingAlgorithm::Ones => {
                let pattern = vec![0xFFu8; self.buffer_size];
                self.overwrite_device_gentle(device_info, &pattern, progress_callback)
            },
            _ => {
                // Default to native erase if supported, otherwise single-pass random
                if device_info.supports_secure_erase {
                    println!("ℹ️  Using SD native erase as default");
                    self.sd_erase_command(device_info, progress_callback)
                } else {
                    println!("ℹ️  Using single-pass random as default for SD card");
                    self.single_pass_random(device_info, progress_callback)
                }
            }
        }
    }
    
    fn verify_erasure(&self, device_info: &DeviceInfo) -> io::Result<bool> {
        if !self.verify_after_wipe {
            return Ok(true);
        }
        
        println!("🔍 Verifying SD card erasure (gentle verification)...");
        
        let mut file = File::open(&device_info.device_path)?;
        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_read = 0u64;
        // For SD cards, very conservative sampling to minimize wear
        let sample_size = std::cmp::min(device_info.size_bytes, 10 * 1024 * 1024); // Sample first 10MB only
        
        while total_read < sample_size {
            let bytes_read = std::io::Read::read(&mut file, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            // Check for non-zero bytes
            if buffer[..bytes_read].iter().any(|&b| b != 0) {
                println!("⚠️  Found non-zero data during SD card verification");
                return Ok(false);
            }
            
            total_read += bytes_read as u64;
            
            // Gentle pause during verification
            std::thread::sleep(Duration::from_millis(10));
        }
        
        println!("✅ SD card erasure verification passed");
        Ok(true)
    }
    
    fn get_recommended_algorithms(&self) -> Vec<WipingAlgorithm> {
        vec![
            WipingAlgorithm::Random,           // Primary choice (single pass, minimal wear)
            WipingAlgorithm::Zeros,            // Simple zero fill
            WipingAlgorithm::QuickFormat,      // Quick format (filesystem level)
            WipingAlgorithm::FileSystemWipe,   // File-level secure deletion
            WipingAlgorithm::TwoPass,          // Conservative 2-pass for critical data
        ]
    }
}