use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Operator,
    Viewer,
}

impl UserRole {
    pub fn can_sanitize(&self) -> bool {
        true // All users can sanitize now
    }
    
    pub fn can_manage_users(&self) -> bool {
        true // All users can manage users now
    }
    
    pub fn as_str(&self) -> &str {
        "User" // All users have the same role display
    }
}

#[derive(Debug, Clone)]
pub struct AuthSystem {
    users: HashMap<String, User>,
    current_user: Option<User>,
    users_file: String,
}

impl AuthSystem {
    pub fn new() -> Self {
        let mut auth = Self {
            users: HashMap::new(),
            current_user: None,
            users_file: "users.json".to_string(),
        };
        
        auth.load_users();
        
        // Create default admin user if no users exist
        if auth.users.is_empty() {
            auth.create_default_admin();
        }
        
        auth
    }
    
    fn create_default_admin(&mut self) {
        let admin_user = User {
            id: uuid::Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password_hash: Self::hash_password("admin123"),
            email: "admin@hddtool.local".to_string(),
            role: UserRole::Admin, // Still admin internally, but all roles have same permissions
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
        };
        
        self.users.insert("admin".to_string(), admin_user);
        self.save_users();
    }
    
    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<User, String> {
        if let Some(user) = self.users.get_mut(username) {
            if !user.is_active {
                return Err("Account is disabled".to_string());
            }
            
            let password_hash = Self::hash_password(password);
            if user.password_hash == password_hash {
                user.last_login = Some(Utc::now());
                let user_clone = user.clone();
                self.current_user = Some(user_clone.clone());
                self.save_users();
                Ok(user_clone)
            } else {
                Err("Invalid password".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }
    
    pub fn create_user(&mut self, username: &str, password: &str, email: &str, role: UserRole) -> Result<(), String> {
        if self.users.contains_key(username) {
            return Err("Username already exists".to_string());
        }
        
        if username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }
        
        if password.len() < 6 {
            return Err("Password must be at least 6 characters".to_string());
        }
        
        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: Self::hash_password(password),
            email: email.to_string(),
            role,
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
        };
        
        self.users.insert(username.to_string(), user);
        self.save_users();
        Ok(())
    }
    
    pub fn logout(&mut self) {
        self.current_user = None;
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.current_user.is_some()
    }
    
    pub fn current_user(&self) -> Option<&User> {
        self.current_user.as_ref()
    }
    
    pub fn get_all_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }
    
    pub fn delete_user(&mut self, username: &str) -> Result<(), String> {
        if username == "admin" {
            return Err("Cannot delete admin user".to_string());
        }
        
        if self.users.remove(username).is_some() {
            self.save_users();
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
    
    pub fn toggle_user_status(&mut self, username: &str) -> Result<(), String> {
        if let Some(user) = self.users.get_mut(username) {
            if username == "admin" {
                return Err("Cannot disable admin user".to_string());
            }
            user.is_active = !user.is_active;
            self.save_users();
            Ok(())
        } else {
            Err("User not found".to_string())
        }
    }
    
    fn load_users(&mut self) {
        if Path::new(&self.users_file).exists() {
            if let Ok(content) = fs::read_to_string(&self.users_file) {
                if let Ok(users) = serde_json::from_str::<HashMap<String, User>>(&content) {
                    self.users = users;
                }
            }
        }
    }
    
    fn save_users(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.users) {
            let _ = fs::write(&self.users_file, content);
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthPage {
    Login,
    CreateUser,
    UserManagement,
}

pub struct AuthUI {
    pub current_page: AuthPage,
    pub login_username: String,
    pub login_password: String,
    pub create_username: String,
    pub create_password: String,
    pub create_email: String,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub show_password: bool,
}

impl Default for AuthUI {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthUI {
    pub fn new() -> Self {
        Self {
            current_page: AuthPage::Login,
            login_username: String::new(),
            login_password: String::new(),
            create_username: String::new(),
            create_password: String::new(),
            create_email: String::new(),
            error_message: None,
            success_message: None,
            show_password: false,
        }
    }
    
    pub fn show_login(&mut self, ui: &mut egui::Ui, auth_system: &mut AuthSystem) -> bool {
        let mut login_attempted = false;
        
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            // Logo and title
            ui.heading(egui::RichText::new("🛡️ SHREDX Authentication")
                .size(28.0)
                .color(egui::Color32::WHITE));
            
            ui.add_space(30.0);
            
            // Login form
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(30, 41, 59, 200))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(30.0))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Login to Continue");
                        ui.add_space(20.0);
                        
                        // Login form with centered alignment
                        ui.vertical_centered(|ui| {
                            // Username field
                            ui.horizontal(|ui| {
                                ui.label("👤 Username:");
                                ui.add_space(10.0);
                                ui.add(egui::TextEdit::singleline(&mut self.login_username)
                                    .desired_width(200.0)
                                    .hint_text("Enter username"));
                            });
                            
                            ui.add_space(10.0);
                            
                            // Password field
                            ui.horizontal(|ui| {
                                ui.label("🔒 Password:");
                                ui.add_space(10.0);
                                ui.add(egui::TextEdit::singleline(&mut self.login_password)
                                    .password(!self.show_password)
                                    .desired_width(200.0)
                                    .hint_text("Enter password"));
                                
                                let eye_icon = if self.show_password { "👁" } else { "👁‍🗨" };
                                if ui.small_button(eye_icon).clicked() {
                                    self.show_password = !self.show_password;
                                }
                            });
                        });
                        
                        ui.add_space(20.0);
                        
                        // Login button (already centered)
                        let login_button = egui::Button::new("🚀 Login")
                            .fill(egui::Color32::from_rgb(37, 99, 235))
                            .min_size(egui::vec2(150.0, 40.0));
                            
                        if ui.add(login_button).clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            login_attempted = true;
                        }
                        
                        ui.add_space(20.0);
                        
                        // Create user link (only for admin)
                        ui.horizontal(|ui| {
                            ui.label("Need to create users?");
                            if ui.link("Create User").clicked() {
                                self.current_page = AuthPage::CreateUser;
                                self.clear_messages();
                            }
                        });
                        
                        // Default credentials info
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Default: admin / admin123")
                            .size(12.0)
                            .color(egui::Color32::GRAY));
                    });
                });
            
            ui.add_space(20.0);
            
            // Error/Success messages
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::from_rgb(239, 68, 68), format!("❌ {}", error));
            }
            
            if let Some(success) = &self.success_message {
                ui.colored_label(egui::Color32::from_rgb(34, 197, 94), format!("✅ {}", success));
            }
        });
        
        // Handle login attempt
        if login_attempted {
            match auth_system.authenticate(&self.login_username, &self.login_password) {
                Ok(user) => {
                    self.success_message = Some(format!("Welcome back, {}!", user.username));
                    self.error_message = None;
                    self.login_username.clear();
                    self.login_password.clear();
                    return true; // Login successful
                }
                Err(error) => {
                    self.error_message = Some(error);
                    self.success_message = None;
                }
            }
        }
        
        false // Login not successful
    }
    
    pub fn show_create_user(&mut self, ui: &mut egui::Ui, auth_system: &mut AuthSystem) {
        ui.vertical_centered(|ui| {
            ui.add_space(30.0);
            
            ui.heading(egui::RichText::new("👥 Create New User")
                .size(24.0)
                .color(egui::Color32::WHITE));
            
            ui.add_space(20.0);
            
            // Create user form
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(30, 41, 59, 200))
                .rounding(egui::Rounding::same(10.0))
                .inner_margin(egui::Margin::same(30.0))
                .show(ui, |ui| {
                    // Create user form with consistent spacing
                    egui::Grid::new("create_user_grid")
                        .num_columns(2)
                        .spacing([20.0, 15.0])
                        .show(ui, |ui| {
                            // Username row
                            ui.label("� Username:");
                            ui.add(egui::TextEdit::singleline(&mut self.create_username)
                                .desired_width(250.0)
                                .hint_text("Min 3 characters"));
                            ui.end_row();
                            
                            // Email row
                            ui.label("� Email:");
                            ui.add(egui::TextEdit::singleline(&mut self.create_email)
                                .desired_width(250.0)
                                .hint_text("user@domain.com"));
                            ui.end_row();
                            
                            // Password row
                            ui.label("🔒 Password:");
                            ui.add(egui::TextEdit::singleline(&mut self.create_password)
                                .password(true)
                                .desired_width(250.0)
                                .hint_text("Min 6 characters"));
                            ui.end_row();
                        });
                    
                    ui.add_space(20.0);
                    
                    // Buttons
                    ui.horizontal(|ui| {
                        let create_button = egui::Button::new("✅ Create User")
                            .fill(egui::Color32::from_rgb(34, 197, 94))
                            .min_size(egui::vec2(120.0, 35.0));
                            
                        if ui.add(create_button).clicked() {
                            match auth_system.create_user(
                                &self.create_username,
                                &self.create_password,
                                &self.create_email,
                                UserRole::Admin // All users get same permissions anyway
                            ) {
                                Ok(()) => {
                                    self.success_message = Some(format!("User '{}' created successfully!", self.create_username));
                                    self.error_message = None;
                                    self.clear_create_form();
                                }
                                Err(error) => {
                                    self.error_message = Some(error);
                                    self.success_message = None;
                                }
                            }
                        }
                        
                        ui.add_space(10.0);
                        
                        let back_button = egui::Button::new("🔙 Back to Login")
                            .fill(egui::Color32::from_rgb(107, 114, 128))
                            .min_size(egui::vec2(120.0, 35.0));
                            
                        if ui.add(back_button).clicked() {
                            self.current_page = AuthPage::Login;
                            self.clear_messages();
                        }
                    });
                });
            
            ui.add_space(20.0);
            
            // Error/Success messages
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::from_rgb(239, 68, 68), format!("❌ {}", error));
            }
            
            if let Some(success) = &self.success_message {
                ui.colored_label(egui::Color32::from_rgb(34, 197, 94), format!("✅ {}", success));
            }
        });
    }
    
    pub fn show_user_management(&mut self, ui: &mut egui::Ui, auth_system: &mut AuthSystem) {
        ui.vertical(|ui| {
            ui.heading("👥 User Management");
            ui.add_space(10.0);
            
            // User table
            let users: Vec<_> = auth_system.get_all_users().into_iter().cloned().collect();
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("user_grid")
                    .num_columns(5)
                    .spacing([10.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        // Header (removed Role column since all users are equal)
                        ui.strong("Username");
                        ui.strong("Email");
                        ui.strong("Status");
                        ui.strong("Last Login");
                        ui.strong("Actions");
                        ui.end_row();
                        
                        // User rows
                        for user in &users {
                            ui.label(&user.username);
                            ui.label(&user.email);
                            
                            let status_color = if user.is_active {
                                egui::Color32::from_rgb(34, 197, 94)
                            } else {
                                egui::Color32::from_rgb(239, 68, 68)
                            };
                            ui.colored_label(status_color, if user.is_active { "Active" } else { "Disabled" });
                            
                            let last_login = user.last_login
                                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                                .unwrap_or_else(|| "Never".to_string());
                            ui.label(last_login);
                            
                            // Action buttons
                            ui.horizontal(|ui| {
                                if user.username != "admin" {
                                    let toggle_text = if user.is_active { "Disable" } else { "Enable" };
                                    if ui.small_button(toggle_text).clicked() {
                                        let _ = auth_system.toggle_user_status(&user.username);
                                    }
                                    
                                    if ui.small_button("Delete").clicked() {
                                        match auth_system.delete_user(&user.username) {
                                            Ok(()) => {
                                                self.success_message = Some(format!("User '{}' deleted", user.username));
                                            }
                                            Err(e) => {
                                                self.error_message = Some(e);
                                            }
                                        }
                                    }
                                }
                            });
                            
                            ui.end_row();
                        }
                    });
            });
        });
    }
    
    fn clear_messages(&mut self) {
        self.error_message = None;
        self.success_message = None;
    }
    
    fn clear_create_form(&mut self) {
        self.create_username.clear();
        self.create_password.clear();
        self.create_email.clear();
    }
}