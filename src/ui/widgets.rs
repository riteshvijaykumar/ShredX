use eframe::egui;
use crate::ui::themes::SecureTheme;

#[derive(Clone, Debug)]
pub struct DriveInfo {
    pub selected: bool,
    pub name: String,
    pub path: String,
    pub size: String,
    pub used: String,
    pub progress: f32,          // Progress as 0.0 to 1.0
    pub time_left: String,      // Calculated time remaining
    pub speed: String,          // Current processing speed
    pub status: String,         // Current status
    pub bytes_total: u64,       // Total bytes to process
    pub bytes_processed: u64,   // Bytes processed so far
    pub start_time: Option<std::time::Instant>, // When processing started
    pub last_update: Option<std::time::Instant>, // Last progress update
}

impl DriveInfo {
    pub fn new(name: String, path: String, size: String, used: String) -> Self {
        Self {
            selected: false,
            name,
            path,
            size,
            used,
            progress: 0.0,
            time_left: "-".to_string(),
            speed: "-".to_string(),
            status: "Ready".to_string(),
            bytes_total: 0,
            bytes_processed: 0,
            start_time: None,
            last_update: None,
        }
    }
    
    pub fn start_processing(&mut self, total_bytes: u64) {
        self.bytes_total = total_bytes;
        self.bytes_processed = 0;
        self.progress = 0.0;
        self.start_time = Some(std::time::Instant::now());
        self.last_update = Some(std::time::Instant::now());
        self.status = "Processing...".to_string();
    }
    
    pub fn update_progress(&mut self, bytes_processed: u64) {
        let now = std::time::Instant::now();
        self.bytes_processed = bytes_processed.min(self.bytes_total);
        self.progress = if self.bytes_total > 0 {
            self.bytes_processed as f32 / self.bytes_total as f32
        } else {
            0.0
        };
        
        // Calculate speed and time remaining
        if let (Some(start), Some(_last_update)) = (self.start_time, self.last_update) {
            let elapsed = now.duration_since(start).as_secs_f64();
            
            if elapsed > 1.0 { // Only calculate after 1 second to avoid division issues
                // Calculate current speed (bytes per second)
                let bytes_per_second = self.bytes_processed as f64 / elapsed;
                
                // Format speed display
                self.speed = if bytes_per_second >= 1_000_000_000.0 {
                    format!("{:.1} GB/s", bytes_per_second / 1_000_000_000.0)
                } else if bytes_per_second >= 1_000_000.0 {
                    format!("{:.1} MB/s", bytes_per_second / 1_000_000.0)
                } else if bytes_per_second >= 1_000.0 {
                    format!("{:.1} KB/s", bytes_per_second / 1_000.0)
                } else {
                    format!("{:.0} B/s", bytes_per_second)
                };
                
                // Calculate time remaining
                let remaining_bytes = self.bytes_total - self.bytes_processed;
                if bytes_per_second > 0.0 && remaining_bytes > 0 {
                    let seconds_remaining = remaining_bytes as f64 / bytes_per_second;
                    self.time_left = format_duration(seconds_remaining);
                } else if remaining_bytes == 0 {
                    self.time_left = "Complete".to_string();
                    self.status = "Complete".to_string();
                } else {
                    self.time_left = "Calculating...".to_string();
                }
            } else {
                self.speed = "Calculating...".to_string();
                self.time_left = "Calculating...".to_string();
            }
        }
        
        self.last_update = Some(now);
    }
}

fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

pub struct ProgressWidget {
    pub progress: f32,
    pub status: String,
}

impl ProgressWidget {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            status: "Ready".to_string(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.status);
        ui.add(egui::ProgressBar::new(self.progress).show_percentage());
    }
}

pub struct TabWidget {
    pub active_tab: usize,
}

impl TabWidget {
    pub fn new() -> Self {
        Self { active_tab: 0 }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, tabs: &[&str]) -> usize {
        ui.horizontal(|ui| {
            for (i, tab_name) in tabs.iter().enumerate() {
                let button_color = if i == self.active_tab {
                    SecureTheme::PRIMARY_BLUE
                } else {
                    egui::Color32::from_rgb(45, 55, 72)
                };
                
                let button = egui::Button::new(*tab_name)
                    .fill(button_color)
                    .min_size(egui::vec2(80.0, 35.0));
                    
                if ui.add(button).clicked() {
                    self.active_tab = i;
                }
            }
        });
        self.active_tab
    }
}

pub struct DriveTableWidget {
    pub drives: Vec<DriveInfo>,
    pub select_all: bool,
}

impl DriveTableWidget {
    pub fn new() -> Self {
        Self {
            drives: Vec::new(),
            select_all: false,
        }
    }
    
    pub fn add_drive(&mut self, drive: DriveInfo) {
        self.drives.push(drive);
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Header
        ui.horizontal(|ui| {
            ui.label("DRIVES");
        });
        
        ui.add_space(10.0);
        
        // Define column widths for consistent alignment
        let col_widths = [60.0, 100.0, 80.0, 80.0, 80.0, 100.0, 80.0, 80.0];
        
        // Column headers with fixed widths
        ui.horizontal(|ui| {
            // Select column header
            ui.allocate_ui_with_layout(
                egui::vec2(col_widths[0], 20.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| { ui.label("Select"); }
            );
            
            // Drive name column header
            ui.allocate_ui_with_layout(
                egui::vec2(col_widths[1], 20.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| { ui.label("Drive name"); }
            );
            
            // Drive path column header
            ui.allocate_ui_with_layout(
                egui::vec2(col_widths[2], 20.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| { ui.label("Drive path"); }
            );
            
            // Size column header
            ui.allocate_ui_with_layout(
                egui::vec2(col_widths[3], 20.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| { ui.label("Size"); }
            );
            
            // Used column header
            ui.allocate_ui_with_layout(
                egui::vec2(col_widths[4], 20.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| { ui.label("Used"); }
            );
            
            // Progress column header
            ui.allocate_ui_with_layout(
                egui::vec2(col_widths[5], 20.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| { ui.label("Progress"); }
            );
            
            // Time left column header
            ui.allocate_ui_with_layout(
                egui::vec2(col_widths[6], 20.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| { ui.label("Time left"); }
            );
            
            // Speed column header
            ui.allocate_ui_with_layout(
                egui::vec2(col_widths[7], 20.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| { ui.label("Speed"); }
            );
        });
            
        ui.separator();
        
        // Drive rows
        let mut rows_to_update = Vec::new();
        for (i, drive) in self.drives.iter().enumerate() {
            let row_bg = if i % 2 == 0 { 
                SecureTheme::TABLE_ROW 
            } else { 
                SecureTheme::TABLE_ROW_ALT 
            };
            
            let response = ui.allocate_response(
                egui::vec2(ui.available_width(), 30.0),
                egui::Sense::hover()
            );
            
            if response.hovered() {
                ui.painter().rect_filled(
                    response.rect,
                    egui::Rounding::same(2.0),
                    SecureTheme::LIGHT_BLUE.gamma_multiply(0.3)
                );
            } else {
                ui.painter().rect_filled(
                    response.rect,
                    egui::Rounding::same(2.0),
                    row_bg
                );
            }
            
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(response.rect), |ui| {
                ui.set_clip_rect(response.rect);
                ui.horizontal(|ui| {
                    // Select column
                    ui.allocate_ui_with_layout(
                        egui::vec2(col_widths[0], 25.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            let mut selected = drive.selected;
                            if ui.checkbox(&mut selected, "").changed() {
                                rows_to_update.push((i, selected));
                            }
                        }
                    );
                    
                    // Drive name column
                    ui.allocate_ui_with_layout(
                        egui::vec2(col_widths[1], 25.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| { ui.label(&drive.name); }
                    );
                    
                    // Drive path column
                    ui.allocate_ui_with_layout(
                        egui::vec2(col_widths[2], 25.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| { ui.label(&drive.path); }
                    );
                    
                    // Size column
                    ui.allocate_ui_with_layout(
                        egui::vec2(col_widths[3], 25.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| { ui.label(&drive.size); }
                    );
                    
                    // Used column
                    ui.allocate_ui_with_layout(
                        egui::vec2(col_widths[4], 25.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| { ui.label(&drive.used); }
                    );
                    
                    // Progress column
                    ui.allocate_ui_with_layout(
                        egui::vec2(col_widths[5], 25.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            if drive.progress > 0.0 {
                                let percentage = (drive.progress * 100.0) as u8;
                                ui.vertical(|ui| {
                                    // Progress bar with percentage overlay
                                    let progress_bar = egui::ProgressBar::new(drive.progress)
                                        .desired_width(col_widths[5] - 20.0)
                                        .desired_height(12.0)
                                        .fill(SecureTheme::LIGHT_BLUE)
                                        .rounding(egui::Rounding::same(4.0));
                                    
                                    let progress_response = ui.add(progress_bar);
                                    
                                    // Overlay percentage text on progress bar
                                    let text = format!("{}%", percentage);
                                    let font_id = egui::FontId::monospace(9.0);
                                    let text_galley = ui.painter().layout_no_wrap(
                                        text,
                                        font_id,
                                        egui::Color32::WHITE
                                    );
                                    
                                    let text_pos = egui::Pos2::new(
                                        progress_response.rect.center().x - text_galley.size().x / 2.0,
                                        progress_response.rect.center().y - text_galley.size().y / 2.0
                                    );
                                    
                                    ui.painter().galley(text_pos, text_galley, egui::Color32::WHITE);
                                });
                            } else {
                                ui.label("-");
                            }
                        }
                    );
                    
                    // Time left column
                    ui.allocate_ui_with_layout(
                        egui::vec2(col_widths[6], 25.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| { ui.label(&drive.time_left); }
                    );
                    
                    // Speed column
                    ui.allocate_ui_with_layout(
                        egui::vec2(col_widths[7], 25.0),
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| { ui.label(&drive.speed); }
                    );
                });
            });
        }
        
        // Apply updates
        for (index, selected) in rows_to_update {
            if let Some(drive) = self.drives.get_mut(index) {
                drive.selected = selected;
            }
        }
        
        ui.add_space(10.0);
        
        // Select All button
        ui.horizontal(|ui| {
            if ui.button("✓ Select All").clicked() {
                let new_state = !self.select_all;
                self.select_all = new_state;
                for drive in &mut self.drives {
                    drive.selected = new_state;
                }
            }
        });
    }
}

pub struct AdvancedOptionsWidget {
    pub eraser_method: String,
    pub verification: String,
    pub confirm_erase: bool,
}

impl AdvancedOptionsWidget {
    pub fn new() -> Self {
        Self {
            eraser_method: "NIST SP 800-88 and DoD 5220.22-M".to_string(),
            verification: "json".to_string(),
            confirm_erase: false,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        self.show_with_permissions(ui, true, "Admin")
    }
    
    pub fn show_with_permissions(&mut self, ui: &mut egui::Ui, can_sanitize: bool, user_role: &str) -> bool {
        println!("🔐 AUTH STATUS: can_sanitize={}, user_role={}", can_sanitize, user_role);
        
        ui.horizontal(|ui| {
            ui.label("ADVANCE OPTIONS");
        });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            // Eraser method dropdown
            ui.label("Eraser method :");
            egui::ComboBox::from_id_salt("eraser_method")
                .selected_text(&self.eraser_method)
                .width(250.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.eraser_method, "NIST SP 800-88 and DoD 5220.22-M".to_string(), "NIST SP 800-88 and DoD 5220.22-M");
                    ui.selectable_value(&mut self.eraser_method, "NIST SP 800-88".to_string(), "NIST SP 800-88");
                    ui.selectable_value(&mut self.eraser_method, "DoD 5220.22-M".to_string(), "DoD 5220.22-M");
                    ui.selectable_value(&mut self.eraser_method, "DoD 5220.22-M ECE".to_string(), "DoD 5220.22-M ECE");
                    ui.selectable_value(&mut self.eraser_method, "Gutmann".to_string(), "Gutmann");
                    ui.selectable_value(&mut self.eraser_method, "Random".to_string(), "Random");
                    ui.selectable_value(&mut self.eraser_method, "ATA Secure Erase".to_string(), "ATA Secure Erase");
                    ui.selectable_value(&mut self.eraser_method, "Enhanced Secure Erase".to_string(), "Enhanced Secure Erase");
                });
            
            ui.add_space(50.0);
            
            // Verification dropdown
            ui.label("Verification :");
            egui::ComboBox::from_id_salt("verification")
                .selected_text(&self.verification)
                .width(100.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.verification, "json".to_string(), "json");
                    ui.selectable_value(&mut self.verification, "xml".to_string(), "xml");
                    ui.selectable_value(&mut self.verification, "pdf".to_string(), "pdf");
                });
        });
        
        ui.add_space(20.0);
        
        // Confirmation checkbox first, then erase button
        ui.vertical_centered(|ui| {
            ui.checkbox(&mut self.confirm_erase, "✅ Confirm to erase the data");
            
            ui.add_space(10.0);
            
            let can_erase = self.confirm_erase && can_sanitize;
            println!("🔧 ERASE STATUS: confirm_erase={}, can_sanitize={}, can_erase={}", 
                    self.confirm_erase, can_sanitize, can_erase);
            
            let erase_button = egui::Button::new("ERASE")
                .fill(if can_erase { SecureTheme::DANGER_RED } else { egui::Color32::GRAY })
                .min_size(egui::vec2(120.0, 40.0));
                
            let mut erase_clicked = false;
            if ui.add_enabled(can_erase, erase_button).clicked() {
                println!("🚨 ERASE BUTTON CLICKED!");
                erase_clicked = true;
            }
            
            if !self.confirm_erase {
                ui.label(egui::RichText::new("⚠ Please confirm to enable erase")
                    .color(egui::Color32::YELLOW)
                    .size(12.0));
            }
            
            erase_clicked
        }).inner
    }
}