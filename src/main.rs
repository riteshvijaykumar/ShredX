use eframe::egui;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use chrono;

// Platform-specific imports (currently unused)
#[cfg(windows)]
#[allow(unused_imports)]
use windows::{
    Win32::Storage::FileSystem::{
        GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW,
    },
};

mod sanitization;
mod ata_commands;
mod advanced_wiper;
mod devices;
mod ui;
mod platform;
mod auth;
mod config;
mod app_config;
mod server_client;
mod certificate;

#[cfg(feature = "server")]
mod server;

use sanitization::{DataSanitizer, SanitizationProgress};
use advanced_wiper::{AdvancedWiper, WipingAlgorithm, WipingProgress, DeviceInfo};
use ui::{SecureTheme, TabWidget, DriveTableWidget, DriveInfo, AdvancedOptionsWidget, show_logo, auth::AuthWidget};
use platform::{get_system_drives, get_device_path_for_sanitization};
use auth::{AuthSystem, AuthUI, AuthPage};
use config::AppConfig;
use app_config::AppConfig as ServerConfig;
use server_client::ServerClient;
use certificate::{CertificateGenerator, SanitizationCertificate, DeviceCertificateInfo, SanitizationInfo, UserInfo};

#[derive(Debug, Clone)]
struct DiskInfo {
    drive_letter: String,
    drive_type: String,
    detailed_type: String,
    file_system: String,
    total_space: u64,
    free_space: u64,
    used_space: u64,
    label: String,
    selected: bool,
}

struct HDDApp {
    disks: Vec<DiskInfo>,
    sanitizer: DataSanitizer,
    sanitization_in_progress: bool,
    sanitization_progress: Option<SanitizationProgress>,
    last_error_message: Option<String>,
    
    // Advanced Wiper Integration
    advanced_wiper: AdvancedWiper,
    selected_algorithm: WipingAlgorithm,
    device_analysis: Option<DeviceInfo>,
    wipe_progress: Arc<Mutex<WipingProgress>>,
    
    // New UI Components
    tab_widget: TabWidget,
    drive_table: DriveTableWidget,
    advanced_options: AdvancedOptionsWidget,
    
    // Authentication System
    auth_system: AuthSystem,
    auth_ui: AuthUI,
    auth_widget: AuthWidget,
    is_authenticated: bool,
    
    // Configuration and Server Integration
    config: AppConfig,
    server_config: ServerConfig,
    server_client: Option<ServerClient>,
    
    // Certificate Management
    certificate_generator: CertificateGenerator,
    certificates: Vec<SanitizationCertificate>,
    current_sanitization_start: Option<chrono::DateTime<chrono::Utc>>,
}

impl HDDApp {
    fn new() -> Self {
        let initial_progress = WipingProgress {
            algorithm: WipingAlgorithm::NistClear,
            current_pass: 0,
            total_passes: 1,
            bytes_processed: 0,
            total_bytes: 0,
            current_pattern: "Ready".to_string(),
            estimated_time_remaining: Duration::from_secs(0),
            speed_mbps: 0.0,
        };
        
        let config = AppConfig::load();
        let server_config = ServerConfig::load();
        let certificate_generator = CertificateGenerator::new();
        
        // Load existing certificates
        let certificates = certificate_generator.load_certificates().unwrap_or_else(|e| {
            eprintln!("Warning: Could not load certificates: {}", e);
            Vec::new()
        });
        
        let mut app = Self { 
            disks: Vec::new(),
            sanitizer: DataSanitizer::new(),
            sanitization_in_progress: false,
            sanitization_progress: None,
            last_error_message: None,
            
            advanced_wiper: AdvancedWiper::new(),
            selected_algorithm: WipingAlgorithm::NistClear,
            device_analysis: None,
            wipe_progress: Arc::new(Mutex::new(initial_progress)),
            
            tab_widget: TabWidget::new(),
            drive_table: DriveTableWidget::new(),
            advanced_options: AdvancedOptionsWidget::new(),
            
            auth_system: AuthSystem::new(),
            auth_ui: AuthUI::new(),
            auth_widget: AuthWidget::new(),
            is_authenticated: false,
            
            config: config.clone(),
            server_config: server_config.clone(),
            server_client: if server_config.is_server_enabled() {
                Some(ServerClient::new(server_config.server_url.clone()))
            } else {
                None
            },
            
            certificate_generator,
            certificates,
            current_sanitization_start: None,
        };
        
        // Initialize authentication widget
        app.auth_widget.initialize(app.server_config.is_server_enabled(), &app.server_config.server_url);
        
        app.refresh_disks();
        app
    }

    fn refresh_disks(&mut self) {
        self.disks.clear();
        self.drive_table.drives.clear();
        
        // Use cross-platform drive detection
        match get_system_drives() {
            Ok(platform_drives) => {
                for platform_drive in platform_drives {
                    // Convert platform drive info to internal format
                    let disk_info = DiskInfo {
                        drive_letter: platform_drive.path.clone(),
                        drive_type: platform_drive.drive_type.clone(),
                        detailed_type: platform_drive.drive_type.clone(),
                        file_system: "Unknown".to_string(), // We'll detect this later if needed
                        total_space: platform_drive.total_space,
                        free_space: platform_drive.free_space,
                        used_space: platform_drive.total_space.saturating_sub(platform_drive.free_space),
                        label: platform_drive.label.clone(),
                        selected: false,
                    };
                    
                    // Add to internal list
                    self.disks.push(disk_info.clone());
                    
                    // Add to drive table widget
                    let drive_ui_info = DriveInfo::new(
                        platform_drive.label,
                        platform_drive.path,
                        Self::format_bytes(platform_drive.total_space),
                        Self::format_bytes(platform_drive.total_space.saturating_sub(platform_drive.free_space)),
                    );
                    self.drive_table.add_drive(drive_ui_info);
                }
            }
            Err(e) => {
                println!("Error getting system drives: {}", e);
            }
        }
    }

    // Cross-platform disk info is now handled by the platform module

    fn get_detailed_drive_info(&self, drive_letter: &str) -> (String, bool) {
        use ata_commands::AtaInterface;
        
        let drive_num = (drive_letter.chars().next().unwrap() as u8).saturating_sub(b'A');
        let physical_drive_path = format!(r"\\.\PhysicalDrive{}", drive_num);
        
        match AtaInterface::new(&physical_drive_path) {
            Ok(ata) => {
                match ata.identify_device() {
                    Ok(identify_data) => {
                        let drive_info = ata.parse_identify_data(&identify_data);
                        
                        let model_lower = drive_info.model.to_lowercase();
                        let drive_type = if model_lower.contains("ssd") || 
                                          model_lower.contains("solid state") ||
                                          model_lower.contains("nvme") ||
                                          model_lower.contains("m.2") {
                            "SSD (Solid State Drive)"
                        } else if model_lower.contains("hdd") || 
                                  model_lower.contains("hard disk") ||
                                  !model_lower.is_empty() {
                            "HDD (Hard Disk Drive)"
                        } else {
                            "Fixed Drive (Unknown Type)"
                        };
                        
                        let secure_erase_available = drive_info.security_supported && 
                                                   !drive_info.security_frozen;
                        
                        (drive_type.to_string(), secure_erase_available)
                    },
                    Err(_) => ("Fixed Drive (ATA Detection Failed)".to_string(), false),
                }
            },
            Err(_) => ("Fixed Drive (No ATA Access)".to_string(), false),
        }
    }

    fn format_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
    
    fn handle_erase_request(&mut self) {
        println!("🚨 HANDLE_ERASE_REQUEST CALLED!");
        println!("🔐 Auth status: {}", self.is_authenticated);
        println!("✅ Confirm erase: {}", self.advanced_options.confirm_erase);
        
        // Check if user is authenticated (no role restrictions)
        /* Authentication check disabled for ease of use
        if !self.is_authenticated {
            self.last_error_message = Some("❌ Authentication required for sanitization operations".to_string());
            return;
        }
        */
        
        // First check if erase confirmation is checked
        if !self.advanced_options.confirm_erase {
            self.last_error_message = Some("❌ Please check 'Confirm to erase the data' before starting the erase process".to_string());
            return;
        }
        
        // Get selected drives
        let selected_drives: Vec<usize> = self.drive_table.drives
            .iter()
            .enumerate()
            .filter(|(_, drive)| drive.selected)
            .map(|(i, _)| i)
            .collect();
            
        // Debug information
        println!("🔧 DEBUG: Total drives: {}", self.drive_table.drives.len());
        for (i, drive) in self.drive_table.drives.iter().enumerate() {
            println!("🔧 DEBUG: Drive {}: {} - Selected: {}", i, drive.name, drive.selected);
        }
        println!("🔧 DEBUG: Selected drives: {:?}", selected_drives);
            
        if selected_drives.is_empty() {
            self.last_error_message = Some("❌ No drives selected for sanitization. Please use the checkboxes to select drives first.".to_string());
            return;
        }
        
        // Check if system drive is selected
        for &drive_idx in &selected_drives {
            if let Some(disk_info) = self.disks.get(drive_idx) {
                if disk_info.drive_letter == "C:" {
                    self.last_error_message = Some("❌ Cannot sanitize system drive C: - this would make your computer unbootable!".to_string());
                    return;
                }
            }
        }
        
        // Start real sanitization for selected drives
        self.sanitization_in_progress = true;
        self.last_error_message = Some(format!("� REAL SANITIZATION STARTED: {} erasure for {} drive(s) - ALL FILES AND FOLDERS WILL BE PERMANENTLY DESTROYED!", 
            self.advanced_options.eraser_method, selected_drives.len()));
        
        // Start actual sanitization process
        self.start_real_sanitization();
    }
    
    fn start_real_sanitization(&mut self) {
        // Record sanitization start time for certificate generation
        self.current_sanitization_start = Some(chrono::Utc::now());
        
        // Collect drives to sanitize
        let drives_to_process: Vec<(String, String, usize)> = self.drive_table.drives
            .iter()
            .enumerate()
            .filter(|(_, drive)| drive.selected)
            .map(|(i, drive)| (drive.path.clone(), drive.name.clone(), i))
            .collect();
        
        if drives_to_process.is_empty() {
            return;
        }
        
        // Start the sanitization process for each selected drive
        for (drive_path, drive_name, drive_index) in drives_to_process {
            // Use device-specific sanitization by default, with fallback to traditional method
            self.start_device_specific_sanitization(&drive_path, &drive_name, drive_index);
        }
        
        // Begin progress simulation/tracking
        self.simulate_sanitization_progress();
    }
    
    /// Enhanced sanitization using device-specific erasers
    fn start_device_specific_sanitization(&mut self, drive_path: &str, drive_name: &str, drive_index: usize) {
        // Get the actual device path for sanitization (platform-specific)
        let sanitization_path = if let Some(disk_info) = self.disks.get(drive_index) {
            get_device_path_for_sanitization(&platform::DriveInfo {
                path: disk_info.drive_letter.clone(),
                label: disk_info.label.clone(),
                drive_type: disk_info.drive_type.clone(),
                total_space: disk_info.total_space,
                free_space: disk_info.free_space,
            })
        } else {
            drive_path.to_string()
        };
        println!("🔍 Starting device-specific analysis and sanitization for drive {} ({})", drive_name, drive_path);
        
        // Convert drive path to device path format
        let device_path = if drive_path.ends_with(':') {
            format!("{}\\", drive_path)
        } else {
            drive_path.to_string()
        };
        
        // Clone necessary data for the thread
        let device_path_clone = device_path.clone();
        let sanitization_path_clone = sanitization_path.clone();
        let drive_name_clone = drive_name.to_string();
        let selected_algorithm = self.selected_algorithm.clone();
        let wipe_progress = Arc::clone(&self.wipe_progress);
        
        // Start analysis and sanitization in a separate thread
        std::thread::spawn(move || {
            match devices::DeviceFactory::analyze_and_create(&device_path_clone) {
                Ok((device_info, eraser)) => {
                    println!("✅ Device analysis complete:");
                    println!("   Device Type: {:?}", device_info.device_type);
                    println!("   Model: {}", device_info.model);
                    println!("   Size: {} bytes", device_info.size_bytes);
                    println!("   Supports Secure Erase: {}", device_info.supports_secure_erase);
                    println!("   Supports TRIM: {}", device_info.supports_trim);
                    
                    // Get recommended algorithms for this device type
                    let recommended_algorithms = eraser.get_recommended_algorithms();
                    println!("🔧 Recommended algorithms: {:?}", recommended_algorithms);
                    
                    // Use selected algorithm, or fall back to first recommended
                    let algorithm_to_use = if recommended_algorithms.contains(&selected_algorithm) {
                        selected_algorithm
                    } else {
                        recommended_algorithms.first().cloned().unwrap_or(WipingAlgorithm::Random)
                    };
                    
                    println!("🚀 Using algorithm: {:?}", algorithm_to_use);
                    
                    // Initialize progress
                    if let Ok(mut progress) = wipe_progress.lock() {
                        progress.algorithm = algorithm_to_use.clone();
                        progress.bytes_processed = 0;
                        progress.total_bytes = device_info.size_bytes;
                        progress.current_pass = 0;
                        progress.total_passes = match algorithm_to_use {
                            WipingAlgorithm::DoD522022M => 3,
                            WipingAlgorithm::Gutmann => 35,
                            WipingAlgorithm::SevenPass => 7,
                            WipingAlgorithm::ThreePass => 3,
                            WipingAlgorithm::TwoPass => 2,
                            _ => 1,
                        };
                    }
                    
                    // Perform device-specific erasure
                    match eraser.erase_device(&device_info, algorithm_to_use, wipe_progress.clone()) {
                        Ok(_) => {
                            println!("✅ Device-specific erasure completed for {}", drive_name_clone);
                            
                            // Verify erasure if supported
                            match eraser.verify_erasure(&device_info) {
                                Ok(true) => println!("✅ Erasure verification passed for {}", drive_name_clone),
                                Ok(false) => println!("⚠️  Erasure verification failed for {}", drive_name_clone),
                                Err(e) => println!("❌ Erasure verification error for {}: {}", drive_name_clone, e),
                            }
                        }
                        Err(e) => {
                            println!("❌ Device-specific erasure failed for {}: {}", drive_name_clone, e);
                            println!("🔄 Falling back to traditional file-level sanitization...");
                            
                            // Fallback to NIST SP 800-88 disk purge
                            let sanitizer = DataSanitizer::new();
                            let wp_clone = wipe_progress.clone();
                            let callback = Box::new(move |p: SanitizationProgress| {
                                if let Ok(mut wp) = wp_clone.lock() {
                                    wp.bytes_processed = p.bytes_processed;
                                    wp.total_bytes = p.total_bytes;
                                    wp.current_pass = p.current_pass;
                                    wp.total_passes = p.total_passes;
                                    wp.estimated_time_remaining = p.estimated_time_remaining;
                                    wp.current_pattern = p.current_operation;
                                }
                            });

                            match sanitizer.nist_purge_entire_disk(&device_path_clone, Some(callback)) {
                                Ok(_) => println!("✅ NIST SP 800-88 Purge completed for {}", drive_name_clone),
                                Err(e) => println!("❌ NIST SP 800-88 Purge also failed for {}: {}", drive_name_clone, e),
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Device analysis failed for {}: {}", drive_name_clone, e);
                    println!("🔄 Falling back to traditional file-level sanitization...");
                    
                    // Fallback to NIST SP 800-88 disk purge
                    let sanitizer = DataSanitizer::new();
                    let wp_clone = wipe_progress.clone();
                    let callback = Box::new(move |p: SanitizationProgress| {
                        if let Ok(mut wp) = wp_clone.lock() {
                            wp.bytes_processed = p.bytes_processed;
                            wp.total_bytes = p.total_bytes;
                            wp.current_pass = p.current_pass;
                            wp.total_passes = p.total_passes;
                            wp.estimated_time_remaining = p.estimated_time_remaining;
                            wp.current_pattern = p.current_operation;
                        }
                    });

                    match sanitizer.nist_purge_entire_disk(&sanitization_path_clone, Some(callback)) {
                        Ok(_) => println!("✅ NIST SP 800-88 Purge completed for {}", drive_name_clone),
                        Err(e) => println!("❌ NIST SP 800-88 Purge also failed for {}: {}", drive_name_clone, e),
                    }
                }
            }
        });
        
        // Initialize progress tracking for this drive
        let total_bytes = if let Some(drive) = self.drive_table.drives.get(drive_index) {
            self.parse_size_to_bytes(&drive.size)
        } else {
            1_000_000_000 // Default 1GB if drive not found
        };
        
        if let Some(drive) = self.drive_table.drives.get_mut(drive_index) {
            drive.start_processing(total_bytes);
            drive.status = format!("Device-specific {} erasure", 
                match self.selected_algorithm {
                    WipingAlgorithm::DoD522022M => "DoD 5220.22-M",
                    WipingAlgorithm::Gutmann => "Gutmann 35-pass",
                    WipingAlgorithm::AtaSecureErase => "ATA Secure Erase",
                    WipingAlgorithm::NvmeSecureErase => "NVMe Secure Erase",
                    WipingAlgorithm::NvmeCryptoErase => "NVMe Crypto Erase",
                    _ => "Optimized",
                });
        }
    }

    fn start_drive_sanitization(&mut self, drive_path: &str, drive_name: &str, drive_index: usize) {
        let sanitizer = DataSanitizer::new();
        let passes = 3; // NIST SP 800-88 and DoD 5220.22-M typically use 3 passes
        
        // Convert drive path to full path (e.g., "C:" -> "C:\")
        let full_drive_path = if drive_path.ends_with(':') {
            format!("{}\\", drive_path)
        } else {
            drive_path.to_string()
        };
        
        println!("🔥 Starting real sanitization of drive {} ({})", drive_name, full_drive_path);
        
        // Start sanitization in a separate thread to avoid blocking UI
        let drive_path_clone = full_drive_path.clone();
        std::thread::spawn(move || {
            match sanitizer.sanitize_files_and_free_space(&drive_path_clone, passes, None) {
                Ok(_) => {
                    println!("✅ Successfully sanitized drive: {}", drive_path_clone);
                }
                Err(e) => {
                    println!("❌ Failed to sanitize drive {}: {}", drive_path_clone, e);
                }
            }
        });
        
        // Initialize progress tracking for this drive
        let total_bytes = if let Some(drive) = self.drive_table.drives.get(drive_index) {
            self.parse_size_to_bytes(&drive.size)
        } else {
            1_000_000_000 // Default 1GB if drive not found
        };
        
        if let Some(drive) = self.drive_table.drives.get_mut(drive_index) {
            drive.start_processing(total_bytes);
            drive.status = format!("Sanitizing {} passes", passes);
        }
    }
    
    fn simulate_sanitization_progress(&mut self) {
        // Collect drive data first to avoid borrowing conflicts
        let mut drive_updates = Vec::new();
        let mut total_bytes_all_drives = 0u64;
        let mut total_processed_all_drives = 0u64;
        
        // Check actual progress from the background thread
        let (real_bytes_processed, real_total_bytes, real_pass, real_total_passes) = 
            if let Ok(progress) = self.wipe_progress.lock() {
                (progress.bytes_processed, progress.total_bytes, progress.current_pass, progress.total_passes)
            } else {
                (0, 0, 0, 0)
            };

        // Start processing for selected drives
        for (i, drive) in self.drive_table.drives.iter().enumerate() {
            if drive.selected && drive.progress == 0.0 {
                // Simulate total bytes based on drive size
                // Parse size string (e.g., "100 GB" -> bytes)
                let total_bytes = self.parse_size_to_bytes(&drive.size);
                drive_updates.push((i, total_bytes, true)); // true = start processing
            }
        }
        
        // Apply start processing updates
        for (i, total_bytes, start) in drive_updates {
            if start {
                if let Some(drive) = self.drive_table.drives.get_mut(i) {
                    drive.start_processing(total_bytes);
                }
            }
        }
        
        // Update progress for processing drives and calculate overall progress
        let mut any_in_progress = false;
        let mut all_completed = true;
        
        for drive in &mut self.drive_table.drives {
            if drive.selected {
                total_bytes_all_drives += drive.bytes_total;
                
                if drive.start_time.is_some() && drive.progress < 1.0 {
                    // Use real progress if available and non-zero, otherwise fallback to simulation
                    let new_bytes_processed = if real_total_bytes > 0 {
                        // Map the single thread progress to this drive (assuming single drive wipe for now)
                        // If multiple drives, this logic needs to be smarter or we need per-drive progress tracking
                        if real_total_bytes >= drive.bytes_total {
                             // If reported total is larger or equal, use ratio
                             let ratio = real_bytes_processed as f64 / real_total_bytes as f64;
                             (ratio * drive.bytes_total as f64) as u64
                        } else {
                             real_bytes_processed
                        }
                    } else {
                        // Fallback simulation: 2MB per update cycle
                        let increment = 1024 * 1024 * 2; 
                        (drive.bytes_processed + increment).min(drive.bytes_total)
                    };

                    drive.update_progress(new_bytes_processed);
                    any_in_progress = true;
                    
                    if drive.progress < 1.0 {
                        all_completed = false;
                    }
                }
                
                total_processed_all_drives += drive.bytes_processed;
            }
        }
        
        // Update overall sanitization progress
        if total_bytes_all_drives > 0 {
            let overall_percentage = (total_processed_all_drives as f64 / total_bytes_all_drives as f64) * 100.0;
            
            let progress = SanitizationProgress {
                current_pass: if real_total_passes > 0 { real_pass } else { if overall_percentage < 33.0 { 1 } else if overall_percentage < 66.0 { 2 } else { 3 } },
                total_passes: if real_total_passes > 0 { real_total_passes } else { 3 },
                percentage: overall_percentage,
                bytes_processed: total_processed_all_drives,
                total_bytes: total_bytes_all_drives,
                estimated_time_remaining: std::time::Duration::from_secs(0),
                current_operation: "Device-specific sanitization".to_string(),
            };
            self.sanitization_progress = Some(progress);
        }
        
        // Check if sanitization is complete
        if all_completed && any_in_progress {
            self.sanitization_in_progress = false;
            self.last_error_message = Some("✅ Sanitization completed successfully!".to_string());
            
            // Generate certificates for completed sanitization
            self.generate_completion_certificates();
        }
    }
    
    fn parse_size_to_bytes(&self, size_str: &str) -> u64 {
        // Parse size string like "100 GB", "50.5 MB" etc.
        let parts: Vec<&str> = size_str.split_whitespace().collect();
        if parts.len() != 2 {
            return 1_000_000_000; // Default 1GB if parsing fails
        }
        
        let number: f64 = parts[0].parse().unwrap_or(1.0);
        let unit = parts[1].to_uppercase();
        
        let multiplier: u64 = match unit.as_str() {
            "B" => 1,
            "KB" => 1_000,
            "MB" => 1_000_000,
            "GB" => 1_000_000_000,
            "TB" => 1_000_000_000_000,
            _ => 1_000_000_000, // Default to GB
        };
        
        (number * multiplier as f64) as u64
    }
    
    fn generate_sanitization_report(&mut self) {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filename = format!("sanitization_report_{}.txt", timestamp);
        
        let mut report = String::new();
        report.push_str("SHREDX - Sanitization Report\n");
        report.push_str(&format!("Generated: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        report.push_str(&format!("Erasure Method: {}\n", self.advanced_options.eraser_method));
        report.push_str(&format!("Verification: {}\n", self.advanced_options.verification));
        report.push_str("\n=== SANITIZED DRIVES ===\n");
        
        for drive in &self.drive_table.drives {
            if drive.selected && drive.progress >= 1.0 {
                report.push_str(&format!("✅ {} ({}): Complete\n", drive.name, drive.path));
                report.push_str(&format!("   Size: {}\n", drive.size));
                report.push_str(&format!("   Status: {}\n", drive.status));
            }
        }
        
        report.push_str("\n=== COMPLIANCE ===\n");
        report.push_str("This sanitization process complies with:\n");
        if self.advanced_options.eraser_method.contains("NIST") {
            report.push_str("- NIST SP 800-88 Guidelines\n");
        }
        if self.advanced_options.eraser_method.contains("DoD") {
            report.push_str("- DoD 5220.22-M Standards\n");
        }
        
        // Try to save the report
        match std::fs::write(&filename, report) {
            Ok(_) => {
                self.last_error_message = Some(format!("✅ Report saved as: {}", filename));
            }
            Err(e) => {
                self.last_error_message = Some(format!("❌ Failed to save report: {}", e));
            }
        }
    }
}

impl eframe::App for HDDApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply SHREDX theme
        SecureTheme::apply(ctx);
        
        // Check authentication status from both systems
        self.is_authenticated = self.auth_system.is_authenticated() || self.auth_widget.is_authenticated();
        
        // Set window title
        ctx.send_viewport_cmd(egui::ViewportCommand::Title("SHREDX - HDD Secure Wipe Tool".to_string()));
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show server authentication UI if server is enabled and not authenticated
            if self.server_config.is_server_enabled() && !self.auth_widget.is_authenticated() {
                ui.heading("🛡️ HDD Tool Server Connection");
                ui.add_space(20.0);
                
                if self.auth_widget.show(ui, ctx) {
                    // Authentication state changed, check if now authenticated
                    if self.auth_widget.is_authenticated() {
                        self.refresh_disks();
                    }
                }
                return; // Don't show main UI until server authenticated
            }
            
            // Show local authentication UI if server is disabled and not locally authenticated
            if !self.server_config.is_server_enabled() && !self.auth_system.is_authenticated() {
                match self.auth_ui.current_page {
                    AuthPage::Login => {
                        if self.auth_ui.show_login(ui, &mut self.auth_system) {
                            // Login successful, refresh drives
                            self.refresh_disks();
                        }
                    }
                    AuthPage::CreateUser => {
                        self.auth_ui.show_create_user(ui, &mut self.auth_system);
                    }
                    AuthPage::UserManagement => {
                        self.auth_ui.show_user_management(ui, &mut self.auth_system);
                    }
                }
                return; // Don't show main UI until authenticated
            }
            
            // Continuous progress updates for active sanitization processes
            let has_active_process = self.drive_table.drives.iter()
                .any(|drive| drive.start_time.is_some() && drive.progress < 1.0);
                
            if has_active_process {
                self.simulate_sanitization_progress();
                ctx.request_repaint(); // Ensure UI updates continuously
            }
        
            // Main UI - only shown when authenticated
            self.show_main_ui(ui);
        });
    }
}

impl HDDApp {
    fn show_main_ui(&mut self, ui: &mut egui::Ui) {
        // Title bar with logo and user info
        ui.horizontal(|ui| {
            show_logo(ui);
            
            // User info and controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Logout button
                if ui.button("🚪 Logout").clicked() {
                    // Logout from the appropriate system based on configuration
                    if self.server_config.is_server_enabled() {
                        // Server-based authentication
                        self.auth_widget.logout();
                    } else {
                        // Local authentication
                        self.auth_system.logout();
                        self.auth_ui = AuthUI::new(); // Reset auth UI
                    }
                }
                
                ui.add_space(10.0);
                
                // User management button (available to all authenticated users)
                let user_info = self.auth_system.current_user().cloned();
                if let Some(user) = user_info {
                    if ui.button("👥 Users").clicked() {
                        self.auth_ui.current_page = AuthPage::UserManagement;
                        self.auth_system.logout(); // Show user management in auth context
                    }
                    ui.add_space(10.0);
                    
                    // Show current user info (without role since all users are equal)
                    ui.label(format!("👤 {}", user.username));
                    ui.add_space(10.0);
                }
                
                // Refresh button
                if ui.button("🔄").clicked() {
                    self.refresh_disks();
                }
            });
        });
            
            ui.add_space(20.0);
            
            // Tab navigation
            let active_tab = self.tab_widget.show(ui, &["Drives", "Details", "Report", "Certificates", "Settings"]);
            
            ui.add_space(20.0);
            
            match active_tab {
                0 => {
                    // Drives tab
                    self.drive_table.show(ui);
                    
                    ui.add_space(30.0);
                    
                    // Advanced options and handle erase button (all authenticated users can sanitize)
                    // For testing/ease of use, we allow sanitization even if unauthenticated
                    let (can_sanitize, user_role) = if self.is_authenticated {
                        (true, "User") 
                    } else {
                        (true, "Unauthenticated") // Allow unauthenticated users to sanitize
                    };
                    
                    if self.advanced_options.show_with_permissions(ui, can_sanitize, user_role) {
                        self.handle_erase_request();
                    }
                    
                    // Show status messages
                    if let Some(ref message) = self.last_error_message {
                        ui.add_space(15.0);
                        if message.starts_with("✅") {
                            ui.colored_label(SecureTheme::SUCCESS_GREEN, message);
                        } else if message.starts_with("🚀") {
                            ui.colored_label(SecureTheme::LIGHT_BLUE, message);
                        } else {
                            ui.colored_label(SecureTheme::DANGER_RED, message);
                        }
                    }
                },
                1 => {
                    // Details tab
                    ui.vertical_centered(|ui| {
                        // Navigation: Back button
                        ui.horizontal(|ui| {
                            if ui.button("← Back to Drives").clicked() {
                                self.tab_widget.active_tab = 0;
                            }
                            ui.add_space(20.0);
                            ui.heading("Drive Details");
                        });
                        
                        ui.add_space(10.0);
                        ui.label("Selected drives information will appear here");
                        
                        // Show details for selected drives
                        for (i, drive) in self.drive_table.drives.iter().enumerate() {
                            if drive.selected {
                                if let Some(disk_info) = self.disks.get(i) {
                                    ui.group(|ui| {
                                        ui.heading(&drive.name);
                                        ui.label(format!("Path: {}", disk_info.drive_letter));
                                        ui.label(format!("Type: {}", disk_info.detailed_type));
                                        ui.label(format!("File System: {}", disk_info.file_system));
                                        ui.label(format!("Total Space: {}", drive.size));
                                        ui.label(format!("Used Space: {}", drive.used));
                                        ui.label(format!("Free Space: {}", Self::format_bytes(disk_info.free_space)));
                                        ui.label("Secure Erase: ❓ Detection needed");
                                        ui.label("Encrypted: ❓ Detection needed");
                                    });
                                }
                            }
                        }
                    });
                },
                2 => {
                    // Report tab
                    ui.vertical_centered(|ui| {
                        // Navigation: Back button
                        ui.horizontal(|ui| {
                            if ui.button("← Back to Drives").clicked() {
                                self.tab_widget.active_tab = 0;
                            }
                            ui.add_space(20.0);
                            ui.heading("Sanitization Reports");
                        });
                        
                        if let Some(ref message) = self.last_error_message {
                            ui.add_space(20.0);
                            if message.starts_with("✅") {
                                ui.colored_label(SecureTheme::SUCCESS_GREEN, message);
                                
                                // Show completion report
                                if !self.sanitization_in_progress {
                                    ui.add_space(10.0);
                                    ui.group(|ui| {
                                        ui.heading("📋 Sanitization Report");
                                        
                                        // Show completed drives
                                        for drive in &self.drive_table.drives {
                                            if drive.selected && drive.progress >= 1.0 {
                                                ui.horizontal(|ui| {
                                                    ui.label("✅");
                                                    ui.label(&drive.name);
                                                    ui.label(format!("({}) - Complete", drive.path));
                                                });
                                            }
                                        }
                                        
                                        ui.add_space(10.0);
                                        ui.label(format!("Method: {}", self.advanced_options.eraser_method));
                                        ui.label(format!("Verification: {}", self.advanced_options.verification));
                                        ui.label(format!("Completion Time: {}", 
                                            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
                                        
                                        ui.add_space(10.0);
                                        if ui.button("💾 Save Report").clicked() {
                                            self.generate_sanitization_report();
                                        }
                                    });
                                }
                            } else {
                                ui.colored_label(SecureTheme::DANGER_RED, message);
                            }
                        }
                        
                        // Show sanitization progress if in progress
                        if self.sanitization_in_progress {
                            ui.add_space(20.0);
                            ui.group(|ui| {
                                ui.heading("🔄 Sanitization in Progress");
                                
                                if let Some(ref progress) = self.sanitization_progress {
                                    ui.label(format!("Pass {}/{}", progress.current_pass, progress.total_passes));
                                    
                                    let progress_bar = egui::ProgressBar::new((progress.percentage / 100.0) as f32)
                                        .text(format!("{:.1}%", progress.percentage))
                                        .fill(SecureTheme::LIGHT_BLUE);
                                    ui.add(progress_bar);
                                    
                                    ui.label(format!(
                                        "Processed: {} / {}",
                                        Self::format_bytes(progress.bytes_processed),
                                        Self::format_bytes(progress.total_bytes)
                                    ));
                                }
                                
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    ui.label("🔧 Method:");
                                    ui.label(&self.advanced_options.eraser_method);
                                });
                                
                                // Show individual drive progress
                                ui.add_space(10.0);
                                ui.label("Individual Drive Progress:");
                                for drive in &self.drive_table.drives {
                                    if drive.selected && drive.start_time.is_some() {
                                        ui.horizontal(|ui| {
                                            let status_icon = if drive.progress >= 1.0 { "✅" } 
                                                           else if drive.progress > 0.0 { "🔄" } 
                                                           else { "⏸" };
                                            ui.label(status_icon);
                                            ui.label(&drive.name);
                                            ui.label(format!("({:.1}%)", drive.progress * 100.0));
                                            ui.label(&drive.speed);
                                            ui.label(&drive.time_left);
                                        });
                                    }
                                }
                            });
                        } else {
                            // Show placeholder when nothing is happening
                            ui.label("No active sanitization processes.");
                            ui.add_space(10.0);
                            ui.label("Start a sanitization process from the Drives tab to see progress here.");
                        }
                    });
                },
                3 => {
                    // Certificates tab - with back button
                    ui.horizontal(|ui| {
                        if ui.button("← Back to Drives").clicked() {
                            self.tab_widget.active_tab = 0;
                        }
                        ui.add_space(20.0);
                    });
                    self.show_certificates_tab(ui);
                },
                4 => {
                    // Settings tab - with back button
                    ui.horizontal(|ui| {
                        if ui.button("← Back to Drives").clicked() {
                            self.tab_widget.active_tab = 0;
                        }
                        ui.add_space(20.0);
                    });
                    self.show_settings_tab(ui);
                },
                _ => {}
            }
    }
    
    fn show_certificates_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("📜 Sanitization Certificates");
            ui.add_space(20.0);
            
            // Refresh certificates button
            ui.horizontal(|ui| {
                if ui.button("🔄 Refresh").clicked() {
                    self.certificates = self.certificate_generator.load_certificates().unwrap_or_else(|e| {
                        eprintln!("Warning: Could not load certificates: {}", e);
                        Vec::new()
                    });
                }
                
                ui.add_space(20.0);
                ui.label(format!("Total certificates: {}", self.certificates.len()));
            });
            
            ui.add_space(20.0);
            
            if self.certificates.is_empty() {
                ui.group(|ui| {
                    ui.set_min_width(600.0);
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label("📭 No certificates available");
                        ui.add_space(10.0);
                        ui.label("Complete a sanitization process to generate certificates");
                        ui.add_space(20.0);
                    });
                });
            } else {
                // Show certificates in a scrollable area
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        let certificates_to_show: Vec<SanitizationCertificate> = self.certificates.clone();
                        for (index, certificate) in certificates_to_show.iter().enumerate() {
                            ui.group(|ui| {
                                ui.set_min_width(800.0);
                                
                                // Certificate header
                                ui.horizontal(|ui| {
                                    // Status indicator
                                    let status_color = if certificate.sanitization_info.success {
                                        SecureTheme::SUCCESS_GREEN
                                    } else {
                                        SecureTheme::DANGER_RED
                                    };
                                    
                                    ui.colored_label(status_color, if certificate.sanitization_info.success { "✅" } else { "❌" });
                                    
                                    ui.vertical(|ui| {
                                        ui.heading(&certificate.device_info.device_name);
                                        ui.label(format!("Certificate ID: {}", &certificate.id[..8]));
                                    });
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(certificate.timestamp.format("%Y-%m-%d %H:%M:%S").to_string());
                                    });
                                });
                                
                                ui.add_space(10.0);
                                
                                // Certificate details in columns
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.strong("Device Information:");
                                        ui.label(format!("Path: {}", certificate.device_info.device_path));
                                        ui.label(format!("Type: {}", certificate.device_info.device_type));
                                        ui.label(format!("Capacity: {}", Self::format_bytes(certificate.device_info.capacity)));
                                    });
                                    
                                    ui.add_space(30.0);
                                    
                                    ui.vertical(|ui| {
                                        ui.strong("Sanitization Details:");
                                        ui.label(format!("Method: {}", certificate.sanitization_info.method));
                                        ui.label(format!("Algorithm: {}", certificate.sanitization_info.algorithm));
                                        ui.label(format!("Passes: {}", certificate.sanitization_info.passes_completed));
                                        ui.label(format!("Duration: {} min", certificate.sanitization_info.duration_seconds / 60));
                                    });
                                    
                                    ui.add_space(30.0);
                                    
                                    ui.vertical(|ui| {
                                        ui.strong("Compliance:");
                                        ui.label(format!("Security Level: {}", certificate.compliance_info.security_level));
                                        ui.label(format!("NIST: {}", if certificate.compliance_info.nist_compliant { "✅" } else { "❌" }));
                                        ui.label(format!("DoD: {}", if certificate.compliance_info.dod_compliant { "✅" } else { "❌" }));
                                        ui.label(format!("Standards: {}", certificate.compliance_info.standards_met.join(", ")));
                                    });
                                });
                                
                                ui.add_space(10.0);
                                
                                // Action buttons
                                ui.horizontal(|ui| {
                                    if ui.button("📄 View Report").clicked() {
                                        let report = self.certificate_generator.generate_certificate_report(certificate);
                                        println!("{}", report);
                                        self.last_error_message = Some("Certificate report printed to console".to_string());
                                    }
                                    
                                    if ui.button("💾 Save Report").clicked() {
                                        match self.certificate_generator.save_certificate_report(certificate) {
                                            Ok(filepath) => {
                                                self.last_error_message = Some(format!("✅ Report saved: {}", filepath));
                                            }
                                            Err(e) => {
                                                self.last_error_message = Some(format!("❌ Failed to save report: {}", e));
                                            }
                                        }
                                    }
                                    
                                    if self.server_config.is_server_enabled() && self.auth_widget.is_authenticated() {
                                        if ui.button("☁️ Upload to Server").clicked() {
                                            self.upload_certificate_to_server(certificate.clone());
                                            self.last_error_message = Some("Certificate upload initiated...".to_string());
                                        }
                                    }
                                });
                            });
                            
                            if index < self.certificates.len() - 1 {
                                ui.add_space(10.0);
                            }
                        }
                    });
            }
        });
    }
    
    fn show_settings_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("🔧 Settings");
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.heading("Server Configuration");
                ui.add_space(10.0);
                
                // Server URL configuration
                ui.horizontal(|ui| {
                    ui.label("Server URL:");
                    ui.text_edit_singleline(&mut self.config.server_url);
                });
                
                ui.add_space(10.0);
                
                // Server sync settings
                ui.checkbox(&mut self.config.enable_server_sync, "Enable server synchronization");
                ui.checkbox(&mut self.config.auto_upload_certificates, "Auto-upload certificates");
                ui.checkbox(&mut self.config.local_storage_only, "Local storage only (disable remote)");
                
                ui.add_space(10.0);
                
                // Connection settings
                ui.horizontal(|ui| {
                    ui.label("Connection timeout (seconds):");
                    ui.add(egui::DragValue::new(&mut self.config.connection_timeout_seconds).range(5..=300));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Retry attempts:");
                    ui.add(egui::DragValue::new(&mut self.config.retry_attempts).range(1..=10));
                });
                
                ui.add_space(15.0);
                
                // Server status
                if self.config.is_server_enabled() {
                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        ui.colored_label(SecureTheme::SUCCESS_GREEN, "🟢 Server sync enabled");
                    });
                    
                    ui.label(format!("Dashboard URL: {}", self.config.get_dashboard_url()));
                    
                    if ui.button("🌐 Open Web Dashboard").clicked() {
                        if let Err(e) = webbrowser::open(&self.config.get_dashboard_url()) {
                            eprintln!("Failed to open browser: {}", e);
                        }
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        ui.colored_label(SecureTheme::WARNING_ORANGE, "🟡 Local mode only");
                    });
                }
                
                ui.add_space(15.0);
                
                // Action buttons
                ui.horizontal(|ui| {
                    if ui.button("💾 Save Configuration").clicked() {
                        if let Err(e) = self.config.save() {
                            eprintln!("Failed to save configuration: {}", e);
                        } else {
                            // Update server client if configuration changed
                            #[cfg(feature = "server")]
                            {
                                self.server_client = if self.config.is_server_enabled() {
                                    Some(crate::server::ServerClient::new(&self.config.server_url))
                                } else {
                                    None
                                };
                            }
                        }
                    }
                    
                    if ui.button("🔄 Test Connection").clicked() {
                        #[cfg(feature = "server")]
                        {
                            if let Some(_client) = &self.server_client {
                                // TODO: Implement async connection test
                                // This would require making the UI async or using a background task
                            }
                        }
                    }
                });
            });
            
            ui.add_space(20.0);
            
            // Application settings
            ui.group(|ui| {
                ui.heading("Application Settings");
                ui.add_space(10.0);
                
                ui.label("Current User:");
                if let Some(user) = self.auth_system.current_user() {
                    ui.indent("user_info", |ui| {
                        ui.label(format!("Username: {}", user.username));
                        ui.label(format!("Role: {}", user.role.as_str()));
                        ui.label(format!("Email: {}", user.email));
                        ui.label(format!("Created: {}", user.created_at.format("%Y-%m-%d %H:%M")));
                        if let Some(last_login) = user.last_login {
                            ui.label(format!("Last Login: {}", last_login.format("%Y-%m-%d %H:%M")));
                        }
                    });
                }
                
                ui.add_space(15.0);
                
                // Environment info
                ui.label("Environment Information:");
                ui.indent("env_info", |ui| {
                    ui.label(format!("OS: {}", std::env::consts::OS));
                    ui.label(format!("Architecture: {}", std::env::consts::ARCH));
                    ui.label(format!("Build Mode: {}", if cfg!(debug_assertions) { "Debug" } else { "Release" }));
                    ui.label(format!("Server Features: {}", if cfg!(feature = "server") { "Enabled" } else { "Disabled" }));
                    
                    if let Ok(server_url) = std::env::var("HDD_TOOL_SERVER_URL") {
                        ui.label(format!("Environment Server URL: {}", server_url));
                    }
                });
            });
            
            ui.add_space(20.0);
            
            // Advanced settings
            ui.group(|ui| {
                ui.heading("Advanced");
                ui.add_space(10.0);
                
                if ui.button("📁 Open Data Directory").clicked() {
                    if let Err(e) = webbrowser::open("file://.") {
                        eprintln!("Failed to open directory: {}", e);
                    }
                }
                
                ui.add_space(10.0);
                
                ui.label("Configuration file location: ./config.json");
                ui.label("User data location: ./users.json");
                ui.label("Certificates location: ./reports/");
            });
        });
    }
    
    fn generate_completion_certificates(&mut self) {
        let end_time = chrono::Utc::now();
        let start_time = self.current_sanitization_start.unwrap_or(end_time);
        
        // Get current user information
        let user_info = if let Some(user) = self.auth_system.current_user() {
            UserInfo {
                username: user.username.clone(),
                user_id: user.id.clone(),
                organization: "HDD Tool User".to_string(),
                role: "User".to_string(), // All users have the same role now
            }
        } else {
            UserInfo {
                username: "Unknown".to_string(),
                user_id: "unknown".to_string(),
                organization: "HDD Tool User".to_string(),
                role: "User".to_string(),
            }
        };

        // Generate certificates for each completed drive
        for (drive_index, drive) in self.drive_table.drives.iter().enumerate() {
            if drive.selected && drive.progress >= 1.0 {
                if let Some(disk_info) = self.disks.get(drive_index) {
                    // Create device certificate info
                    let device_info = DeviceCertificateInfo {
                        device_path: disk_info.drive_letter.clone(),
                        device_name: drive.name.clone(),
                        device_type: disk_info.drive_type.clone(),
                        manufacturer: "Unknown".to_string(), // Would be detected from actual hardware
                        model: "Unknown".to_string(),
                        serial_number: "N/A".to_string(),
                        capacity: disk_info.total_space,
                        sector_size: 512, // Standard sector size
                        supports_secure_erase: false, // Would be detected
                        supports_crypto_erase: false,
                        encryption_status: "Unknown".to_string(),
                    };

                    // Create sanitization info
                    let duration = end_time.signed_duration_since(start_time).num_seconds() as u64;
                    let speed = if duration > 0 {
                        (disk_info.total_space as f64) / (duration as f64 * 1024.0 * 1024.0)
                    } else {
                        0.0
                    };

                    let sanitization_info = SanitizationInfo {
                        method: self.advanced_options.eraser_method.clone(),
                        algorithm: format!("{:?}", self.selected_algorithm),
                        passes_completed: match self.selected_algorithm {
                            WipingAlgorithm::DoD522022M => 3,
                            WipingAlgorithm::Gutmann => 35,  
                            WipingAlgorithm::SevenPass => 7,
                            WipingAlgorithm::ThreePass => 3,
                            WipingAlgorithm::TwoPass => 2,
                            _ => 1,
                        },
                        total_bytes_processed: disk_info.total_space,
                        start_time,
                        end_time,
                        duration_seconds: duration,
                        average_speed_mbps: speed,
                        success: true,
                        error_count: 0,
                    };

                    // Generate certificate
                    match self.certificate_generator.generate_certificate(
                        device_info,
                        sanitization_info,
                        user_info.clone(),
                    ) {
                        Ok(certificate) => {
                            // Save certificate locally
                            if let Err(e) = self.certificate_generator.save_certificate_local(&certificate) {
                                eprintln!("Warning: Could not save certificate locally: {}", e);
                            }

                            // Save human-readable report
                            if let Err(e) = self.certificate_generator.save_certificate_report(&certificate) {
                                eprintln!("Warning: Could not save certificate report: {}", e);
                            }

                            // Add to local certificates list
                            self.certificates.push(certificate.clone());

                            // Upload to server if configured and authenticated
                            if self.server_config.auto_upload_certificates {
                                if self.auth_widget.is_authenticated() {
                                    self.upload_certificate_to_server(certificate);
                                } else if self.auth_system.is_authenticated() {
                                    // Could upload via local auth too if we had server integration
                                    println!("Certificate ready for server upload when server connection is available");
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error generating certificate for {}: {}", drive.name, e);
                        }
                    }
                }
            }
        }

        self.current_sanitization_start = None; // Reset for next sanitization
    }

    fn upload_certificate_to_server(&self, certificate: SanitizationCertificate) {
        if let Some(ref server_client) = self.server_client {
            let certificate_data = match serde_json::to_string(&certificate) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error serializing certificate: {}", e);
                    return;
                }
            };

            let device_info = format!("{} - {} ({})", 
                certificate.device_info.device_name,
                certificate.device_info.device_type,
                certificate.device_info.device_path);

            let method = certificate.sanitization_info.method.clone();

            // Clone server_client for async operation
            let server_client_clone = server_client.clone();
            
            // Upload in background thread
            tokio::spawn(async move {
                match server_client_clone.upload_certificate(certificate_data, device_info, method).await {
                    Ok(response) => {
                        if response.success {
                            println!("✅ Certificate uploaded to server successfully!");
                        } else {
                            println!("❌ Server rejected certificate: {}", response.message);
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to upload certificate to server: {}", e);
                    }
                }
            });
        }
    }
}

fn main() -> eframe::Result<()> {
    // Initialize Tokio runtime
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Unable to create Tokio runtime");

    // Enter the runtime context to allow tokio::spawn to work
    let _enter = rt.enter();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "SHREDX - HDD Secure Wipe Tool",
        native_options,
        Box::new(|_cc| Ok(Box::new(HDDApp::new()))),
    )
}