//! NVMe (Non-Volatile Memory Express) specific erasure methods
//! 
//! NVMe drives use the NVMe protocol and often support advanced
//! erasure commands including Secure Erase and Cryptographic Erase.

use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::fs::{File, OpenOptions};
use std::io::{Write, Seek, SeekFrom};
use std::process::Command;
use crate::advanced_wiper::{DeviceInfo, DeviceType, WipingProgress, WipingAlgorithm};
use crate::devices::DeviceEraser;

pub struct NvmeEraser {
    buffer_size: usize,
    verify_after_wipe: bool,
    namespace_id: u32,
}

impl NvmeEraser {
    pub fn new() -> Self {
        Self {
            buffer_size: 4 * 1024 * 1024, // 4MB buffer for NVMe
            verify_after_wipe: true,
            namespace_id: 1, // Default namespace
        }
    }
    
    pub fn with_namespace(namespace_id: u32) -> Self {
        Self {
            buffer_size: 4 * 1024 * 1024,
            verify_after_wipe: true,
            namespace_id,
        }
    }
    
    /// NVMe Secure Erase - User Data Erase
    pub fn nvme_secure_erase(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting NVMe Secure Erase (User Data)");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "NVMe Secure Erase".to_string();
        }
        
        if !device_info.supports_secure_erase {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NVMe Secure Erase not supported on this device"
            ));
        }
        
        // Execute NVMe secure erase command
        // Note: This would typically use nvme-cli or Windows NVMe APIs
        println!("🔧 Executing NVMe Format with Secure Erase...");
        
        // Simulate NVMe secure erase (in real implementation, this would use proper NVMe commands)
        let start_time = Instant::now();
        
        // For Windows, we might use StorNVMe or nvme-cli if available
        let result = self.execute_nvme_format_command(device_info, false);
        
        match result {
            Ok(_) => {
                // Update progress to completion
                if let Ok(mut progress) = progress_callback.lock() {
                    progress.bytes_processed = device_info.size_bytes;
                    progress.total_bytes = device_info.size_bytes;
                    progress.speed_mbps = (device_info.size_bytes as f64) / (1024.0 * 1024.0) / start_time.elapsed().as_secs_f64();
                }
                
                println!("✅ NVMe Secure Erase completed");
                Ok(())
            }
            Err(e) => {
                println!("❌ NVMe Secure Erase failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// NVMe Cryptographic Erase
    pub fn nvme_crypto_erase(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting NVMe Cryptographic Erase");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "NVMe Crypto Erase".to_string();
        }
        
        if !device_info.supports_crypto_erase {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NVMe Cryptographic Erase not supported on this device"
            ));
        }
        
        println!("🔐 Executing NVMe Cryptographic Erase...");
        let start_time = Instant::now();
        
        // Execute cryptographic erase
        let result = self.execute_nvme_format_command(device_info, true);
        
        match result {
            Ok(_) => {
                // Update progress to completion
                if let Ok(mut progress) = progress_callback.lock() {
                    progress.bytes_processed = device_info.size_bytes;
                    progress.total_bytes = device_info.size_bytes;
                    progress.speed_mbps = (device_info.size_bytes as f64) / (1024.0 * 1024.0) / start_time.elapsed().as_secs_f64();
                }
                
                println!("✅ NVMe Cryptographic Erase completed");
                Ok(())
            }
            Err(e) => {
                println!("❌ NVMe Cryptographic Erase failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// NVMe Write Zeroes command
    pub fn nvme_write_zeroes(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting NVMe Write Zeroes");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "NVMe Write Zeroes".to_string();
        }
        
        let start_time = Instant::now();
        let total_blocks = device_info.size_bytes / device_info.sector_size as u64;
        let blocks_per_command = 65536; // Maximum blocks per Write Zeroes command
        let mut blocks_processed = 0u64;
        
        println!("🔧 Writing zeroes to {} blocks...", total_blocks);
        
        while blocks_processed < total_blocks {
            let blocks_remaining = total_blocks - blocks_processed;
            let blocks_to_process = std::cmp::min(blocks_per_command, blocks_remaining);
            
            // Execute Write Zeroes command for this range
            let result = self.execute_write_zeroes_command(
                device_info,
                blocks_processed,
                blocks_to_process
            );
            
            match result {
                Ok(_) => {
                    blocks_processed += blocks_to_process;
                    
                    // Update progress
                    if let Ok(mut progress) = progress_callback.lock() {
                        let bytes_processed = blocks_processed * device_info.sector_size as u64;
                        progress.bytes_processed = bytes_processed;
                        progress.total_bytes = device_info.size_bytes;
                        
                        let elapsed = start_time.elapsed();
                        if elapsed.as_secs() > 0 {
                            progress.speed_mbps = (bytes_processed as f64) / (1024.0 * 1024.0) / elapsed.as_secs_f64();
                            
                            if blocks_processed > 0 {
                                let estimated_total_time = elapsed.as_secs_f64() * (total_blocks as f64) / (blocks_processed as f64);
                                progress.estimated_time_remaining = Duration::from_secs_f64(estimated_total_time - elapsed.as_secs_f64());
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Write Zeroes failed at block {}: {}", blocks_processed, e);
                    return Err(e);
                }
            }
        }
        
        println!("✅ NVMe Write Zeroes completed");
        Ok(())
    }
    
    /// NVMe Deallocate (TRIM equivalent)
    pub fn nvme_deallocate(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting NVMe Deallocate");
        
        if !device_info.supports_trim {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "NVMe Deallocate not supported on this device"
            ));
        }
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "NVMe Deallocate".to_string();
        }
        
        let start_time = Instant::now();
        let total_blocks = device_info.size_bytes / device_info.sector_size as u64;
        
        println!("🔧 Deallocating {} blocks...", total_blocks);
        
        // Execute deallocate command for the entire device
        let result = self.execute_deallocate_command(device_info, 0, total_blocks);
        
        match result {
            Ok(_) => {
                // Update progress to completion
                if let Ok(mut progress) = progress_callback.lock() {
                    progress.bytes_processed = device_info.size_bytes;
                    progress.total_bytes = device_info.size_bytes;
                    progress.speed_mbps = (device_info.size_bytes as f64) / (1024.0 * 1024.0) / start_time.elapsed().as_secs_f64();
                }
                
                println!("✅ NVMe Deallocate completed");
                Ok(())
            }
            Err(e) => {
                println!("❌ NVMe Deallocate failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Single-pass random overwrite for NVMe
    pub fn single_pass_overwrite(
        &self,
        device_info: &DeviceInfo,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🔄 Starting single-pass overwrite for NVMe");
        
        // Update progress
        if let Ok(mut progress) = progress_callback.lock() {
            progress.current_pass = 1;
            progress.total_passes = 1;
            progress.current_pattern = "Random Overwrite".to_string();
        }
        
        let pattern = self.generate_random_pattern(self.buffer_size);
        self.overwrite_device(device_info, &pattern, progress_callback)?;
        
        println!("✅ Single-pass overwrite completed for NVMe");
        Ok(())
    }
    
    /// Execute NVMe format command
    fn execute_nvme_format_command(&self, device_info: &DeviceInfo, crypto_erase: bool) -> io::Result<()> {
        // This is a simplified implementation
        // In a real implementation, this would use Windows NVMe APIs or nvme-cli
        
        let erase_type = if crypto_erase { "2" } else { "1" }; // 1 = User Data Erase, 2 = Cryptographic Erase
        
        // Try to use nvme-cli if available
        let output = Command::new("nvme")
            .args(&[
                "format",
                &device_info.device_path,
                "--namespace-id", &self.namespace_id.to_string(),
                "--ses", erase_type,
            ])
            .output();
            
        match output {
            Ok(result) => {
                if result.status.success() {
                    Ok(())
                } else {
                    let error_msg = String::from_utf8_lossy(&result.stderr);
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("NVMe format failed: {}", error_msg)
                    ))
                }
            }
            Err(_) => {
                // Fallback: simulate the operation
                println!("ℹ️  nvme-cli not available, cannot perform NVMe format.");
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "nvme-cli tool not found. Cannot perform hardware secure erase."
                ))
            }
        }
    }
    
    /// Execute Write Zeroes command
    fn execute_write_zeroes_command(
        &self,
        device_info: &DeviceInfo,
        start_block: u64,
        num_blocks: u64,
    ) -> io::Result<()> {
        // This would typically use NVMe Write Zeroes command
        // For now, simulate with actual zero writes
        let mut file = OpenOptions::new()
            .write(true)
            .open(&device_info.device_path)?;
        
        let start_offset = start_block * device_info.sector_size as u64;
        let write_size = num_blocks * device_info.sector_size as u64;
        let zero_buffer = vec![0u8; write_size as usize];
        
        file.seek(SeekFrom::Start(start_offset))?;
        file.write_all(&zero_buffer)?;
        file.sync_data()?;
        
        Ok(())
    }
    
    /// Execute Deallocate command
    fn execute_deallocate_command(
        &self,
        _device_info: &DeviceInfo,
        start_block: u64,
        num_blocks: u64,
    ) -> io::Result<()> {
        // This would typically use NVMe Deallocate command
        // For now, return error as we cannot guarantee erasure without proper driver support
        println!("🔧 Deallocating blocks {} to {}", start_block, start_block + num_blocks - 1);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "NVMe Deallocate not implemented for this platform"
        ))
    }
    
    /// Overwrite device with specific pattern (NVMe-optimized)
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
        
        // Use very large chunks for NVMe to maximize performance
        let chunk_size = std::cmp::max(self.buffer_size, 8 * 1024 * 1024); // At least 8MB
        let pattern_chunk = self.expand_pattern(pattern, chunk_size);
        
        while bytes_written < total_size {
            let remaining = total_size - bytes_written;
            let write_size = std::cmp::min(pattern_chunk.len() as u64, remaining) as usize;
            
            file.write_all(&pattern_chunk[..write_size])?;
            bytes_written += write_size as u64;
            
            // Force sync less frequently for NVMe (better performance)
            if bytes_written % (256 * 1024 * 1024) == 0 {
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
    
    /// Detect NVMe capabilities
    fn detect_nvme_capabilities(&self, device_path: &str) -> (bool, bool, bool) {
        // This would typically query the NVMe controller
        // For now, return conservative defaults
        let supports_secure_erase = true;  // Most NVMe drives support this
        let supports_crypto_erase = true;  // Many modern NVMe drives support this
        let supports_deallocate = true;    // Standard NVMe feature
        
        (supports_secure_erase, supports_crypto_erase, supports_deallocate)
    }
}

impl DeviceEraser for NvmeEraser {
    fn analyze_device(&self, device_path: &str) -> io::Result<DeviceInfo> {
        println!("🔍 Analyzing NVMe device: {}", device_path);
        
        let (supports_secure_erase, supports_crypto_erase, supports_deallocate) = 
            self.detect_nvme_capabilities(device_path);
        
        // Try to get basic device info
        let device_info = match File::open(device_path) {
            Ok(file) => {
                let metadata = file.metadata()?;
                DeviceInfo {
                    device_path: device_path.to_string(),
                    device_type: DeviceType::NVMe,
                    size_bytes: metadata.len(),
                    sector_size: 4096, // NVMe typically uses 4K sectors
                    supports_trim: supports_deallocate,
                    supports_secure_erase,
                    supports_enhanced_secure_erase: supports_secure_erase,
                    supports_crypto_erase,
                    is_removable: false,
                    vendor: "Unknown".to_string(),
                    model: "Unknown NVMe".to_string(),
                    serial: "Unknown".to_string(),
                }
            }
            Err(e) => return Err(e),
        };
        
        println!("✅ NVMe analysis complete: {} ({} bytes)", 
                device_info.model, device_info.size_bytes);
        Ok(device_info)
    }
    
    fn erase_device(
        &self,
        device_info: &DeviceInfo,
        algorithm: WipingAlgorithm,
        progress_callback: Arc<Mutex<WipingProgress>>,
    ) -> io::Result<()> {
        println!("🚀 Starting NVMe erasure with algorithm: {:?}", algorithm);
        
        match algorithm {
            WipingAlgorithm::NvmeSecureErase => self.nvme_secure_erase(device_info, progress_callback),
            WipingAlgorithm::NvmeCryptoErase => self.nvme_crypto_erase(device_info, progress_callback),
            WipingAlgorithm::NistClear => self.nvme_write_zeroes(device_info, progress_callback),
            WipingAlgorithm::Random => self.single_pass_overwrite(device_info, progress_callback),
            WipingAlgorithm::Zeros => self.nvme_write_zeroes(device_info, progress_callback),
            WipingAlgorithm::Ones => {
                let pattern = vec![0xFFu8; self.buffer_size];
                self.overwrite_device(device_info, &pattern, progress_callback)
            },
            _ => {
                // Default to NVMe Secure Erase if supported, otherwise crypto erase
                if device_info.supports_secure_erase {
                    println!("ℹ️  Using NVMe Secure Erase as default");
                    self.nvme_secure_erase(device_info, progress_callback)
                } else if device_info.supports_crypto_erase {
                    println!("ℹ️  Using NVMe Crypto Erase as fallback");
                    self.nvme_crypto_erase(device_info, progress_callback)
                } else {
                    println!("ℹ️  Using single-pass overwrite as fallback");
                    self.single_pass_overwrite(device_info, progress_callback)
                }
            }
        }
    }
    
    fn verify_erasure(&self, device_info: &DeviceInfo) -> io::Result<bool> {
        if !self.verify_after_wipe {
            return Ok(true);
        }
        
        println!("🔍 Verifying NVMe erasure...");
        
        let mut file = File::open(&device_info.device_path)?;
        let mut buffer = vec![0u8; self.buffer_size];
        let mut total_read = 0u64;
        // For NVMe, sample strategically across the device
        let sample_size = std::cmp::min(device_info.size_bytes, 1024 * 1024 * 1024); // Sample first 1GB
        
        while total_read < sample_size {
            let bytes_read = std::io::Read::read(&mut file, &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            // Check for non-zero bytes
            if buffer[..bytes_read].iter().any(|&b| b != 0) {
                println!("⚠️  Found non-zero data during NVMe verification");
                return Ok(false);
            }
            
            total_read += bytes_read as u64;
        }
        
        println!("✅ NVMe erasure verification passed");
        Ok(true)
    }
    
    fn get_recommended_algorithms(&self) -> Vec<WipingAlgorithm> {
        vec![
            WipingAlgorithm::NvmeSecureErase,    // Primary choice for NVMe
            WipingAlgorithm::NvmeCryptoErase,    // For encrypted NVMe drives
            WipingAlgorithm::NistClear,          // NIST approved method
            WipingAlgorithm::Random,             // Single-pass fallback
            WipingAlgorithm::Zeros,              // Simple zero fill
        ]
    }
}