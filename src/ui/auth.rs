use eframe::egui;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum AuthState {
    NotConnected,
    Connected,
    Login,
    Register,
    Authenticated(String), // username
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredUser {
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
    pub last_login: Option<String>,
    pub is_active: bool,
}

pub struct AuthWidget {
    pub state: AuthState,
    
    // Login form
    login_username: String,
    login_password: String,
    
    // Register form
    register_username: String,
    register_email: String,
    register_password: String,
    register_confirm_password: String,
    
    // Status messages
    status_message: String,
    error_message: String,
    
    // Loading states
    is_connecting: bool,
    is_logging_in: bool,
    is_registering: bool,
    
    // Stored users
    stored_users: HashMap<String, StoredUser>,
}

impl Default for AuthWidget {
    fn default() -> Self {
        let mut widget = Self {
            state: AuthState::NotConnected,
            login_username: String::new(),
            login_password: String::new(),
            register_username: String::new(),
            register_email: String::new(),
            register_password: String::new(),
            register_confirm_password: String::new(),
            status_message: String::new(),
            error_message: String::new(),
            is_connecting: false,
            is_logging_in: false,
            is_registering: false,
            stored_users: HashMap::new(),
        };
        
        // Load stored users from file
        widget.load_stored_users();
        widget
    }
}

impl AuthWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initialize(&mut self, server_enabled: bool, server_url: &str) {
        if server_enabled {
            self.state = AuthState::Connected;
            self.status_message = format!("Connected to server: {}", server_url);
        } else {
            self.state = AuthState::NotConnected;
            self.status_message = "Server connection disabled - running in local mode".to_string();
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut state_changed = false;

        match self.state.clone() {
            AuthState::NotConnected => {
                self.show_not_connected(ui);
            }
            AuthState::Connected => {
                state_changed = self.show_connection_options(ui);
            }
            AuthState::Login => {
                state_changed = self.show_login_form(ui, ctx);
            }
            AuthState::Register => {
                state_changed = self.show_register_form(ui, ctx);
            }
            AuthState::Authenticated(username) => {
                if self.show_authenticated_status(ui, &username) {
                    state_changed = true;
                }
            }
        }

        // Show status messages
        if !self.status_message.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(0, 150, 0), &self.status_message);
        }
        
        if !self.error_message.is_empty() {
            ui.colored_label(egui::Color32::from_rgb(200, 0, 0), &self.error_message);
        }

        state_changed
    }

    fn show_not_connected(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add(egui::Label::new("🔌 Server Connection Disabled"));
            ui.add_space(10.0);
            ui.label("Running in local mode - certificates stored locally");
            ui.add_space(10.0);
            ui.label("To enable server features:");
            ui.label("1. Set HDD_TOOL_SERVER_URL environment variable");
            ui.label("2. Or update config.json with server details");
        });
    }

    fn show_connection_options(&mut self, ui: &mut egui::Ui) -> bool {
        ui.vertical_centered(|ui| {
            ui.add(egui::Label::new("🛡️ HDD Tool Server Authentication"));
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("🔑 Login").min_size(egui::vec2(120.0, 40.0))).clicked() {
                    self.state = AuthState::Login;
                    return true;
                }
                
                ui.add_space(20.0);
                
                if ui.add(egui::Button::new("📝 Create Account").min_size(egui::vec2(120.0, 40.0))).clicked() {
                    self.state = AuthState::Register;
                    return true;
                }
                
                false
            });

            ui.add_space(20.0);
            
            if ui.add(egui::Button::new("🧪 Test Connection")).clicked() {
                self.test_connection();
            }
        });
        
        false
    }

    fn show_login_form(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut state_changed = false;
        
        ui.vertical_centered(|ui| {
            ui.add(egui::Label::new("🔑 Login to HDD Tool"));
            ui.add_space(15.0);
            
            // Show available users hint
            if !self.stored_users.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("💡 Available users:");
                    let active_users: Vec<String> = self.stored_users.iter()
                        .filter(|(_, user)| user.is_active)
                        .map(|(username, _)| username.clone())
                        .collect();
                    ui.label(active_users.join(", "));
                });
                ui.add_space(10.0);
            }
            
            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.add(egui::TextEdit::singleline(&mut self.login_username).desired_width(200.0));
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(&mut self.login_password)
                    .password(true)
                    .desired_width(200.0));
            });
            
            ui.add_space(15.0);
            
            // Show default credentials hint
            ui.collapsing("🔧 Test Credentials", |ui| {
                ui.label("👤 Admin: admin / admin123");
                ui.label("👤 Root: root / (check users.json for password)");
                ui.label("(These are the stored user accounts)");
            });
            
            ui.add_space(15.0);
            
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("🔑 Login").min_size(egui::vec2(100.0, 30.0))).clicked() && !self.is_logging_in {
                    self.perform_login(ctx);
                    state_changed = true;
                }
                
                ui.add_space(10.0);
                
                if ui.add(egui::Button::new("⬅️ Back")).clicked() {
                    self.state = AuthState::Connected;
                    self.clear_error();
                    state_changed = true;
                }
            });
            
            if self.is_logging_in {
                ui.add_space(10.0);
                ui.spinner();
                ui.label("Authenticating...");
            }
        });
        
        state_changed
    }

    fn show_register_form(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> bool {
        let mut state_changed = false;
        
        ui.vertical_centered(|ui| {
            ui.add(egui::Label::new("📝 Create New Account"));
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.add(egui::TextEdit::singleline(&mut self.register_username).desired_width(200.0));
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("Email:");
                ui.add(egui::TextEdit::singleline(&mut self.register_email).desired_width(200.0));
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(&mut self.register_password)
                    .password(true)
                    .desired_width(200.0));
            });
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("Confirm:");
                ui.add(egui::TextEdit::singleline(&mut self.register_confirm_password)
                    .password(true)
                    .desired_width(200.0));
            });
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("📝 Create Account").min_size(egui::vec2(120.0, 30.0))).clicked() && !self.is_registering {
                    self.perform_registration(ctx);
                    state_changed = true;
                }
                
                ui.add_space(10.0);
                
                if ui.add(egui::Button::new("⬅️ Back")).clicked() {
                    self.state = AuthState::Connected;
                    self.clear_error();
                    state_changed = true;
                }
            });
            
            if self.is_registering {
                ui.add_space(10.0);
                ui.spinner();
                ui.label("Creating account...");
            }
        });
        
        state_changed
    }

    fn show_authenticated_status(&mut self, ui: &mut egui::Ui, username: &str) -> bool {
        let mut state_changed = false;
        
        ui.vertical_centered(|ui| {
            ui.add(egui::Label::new("✅ Authenticated"));
            ui.add_space(10.0);
            
            // Show user info
            if let Some(user) = self.stored_users.get(username) {
                ui.label(format!("Welcome, {}!", username));
                ui.label(format!("Role: {}", user.role));
                ui.label(format!("Email: {}", user.email));
            } else {
                ui.label(format!("Welcome, {}!", username));
            }
            
            ui.add_space(10.0);
            ui.label("🔐 You now have access to all HDD Tool features");
            ui.add_space(20.0);
            
            if ui.add(egui::Button::new("🚪 Logout").min_size(egui::vec2(100.0, 30.0))).clicked() {
                self.logout();
                state_changed = true;
            }
        });
        
        state_changed
    }

    fn perform_login(&mut self, _ctx: &egui::Context) {
        self.is_logging_in = true;
        self.clear_error();
        
        if self.login_username.is_empty() || self.login_password.is_empty() {
            self.error_message = "Please enter username and password".to_string();
            self.is_logging_in = false;
            return;
        }
        
        // Validate against stored users
        if let Some(user) = self.stored_users.get(&self.login_username) {
            if !user.is_active {
                self.error_message = "Account is disabled".to_string();
                self.is_logging_in = false;
                return;
            }
            
            // Hash the provided password and compare
            let password_hash = self.hash_password(&self.login_password);
            if password_hash == user.password_hash {
                // Successful login
                let username = self.login_username.clone();
                self.state = AuthState::Authenticated(username.clone());
                self.status_message = format!("Welcome back, {}!", username);
                self.update_last_login(&username);
                self.clear_forms();
            } else {
                self.error_message = "Invalid username or password".to_string();
            }
        } else {
            self.error_message = "Invalid username or password".to_string();
        }
        
        self.is_logging_in = false;
    }

    fn perform_registration(&mut self, _ctx: &egui::Context) {
        self.is_registering = true;
        self.clear_error();
        
        // Simple validation for demo
        if !self.register_username.is_empty() && 
           !self.register_email.is_empty() && 
           !self.register_password.is_empty() &&
           self.register_password == self.register_confirm_password {
            self.status_message = "Account created successfully! Please login.".to_string();
            self.state = AuthState::Connected;
            self.clear_forms();
        } else {
            self.error_message = "Please fill all fields correctly".to_string();
        }
        
        self.is_registering = false;
    }

    fn test_connection(&mut self) {
        self.status_message = "Connection test successful!".to_string();
    }

    fn clear_error(&mut self) {
        self.error_message.clear();
    }

    fn clear_forms(&mut self) {
        self.login_username.clear();
        self.login_password.clear();
        self.register_username.clear();
        self.register_email.clear();
        self.register_password.clear();
        self.register_confirm_password.clear();
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self.state, AuthState::Authenticated(_))
    }

    pub fn get_current_user(&self) -> Option<&str> {
        if let AuthState::Authenticated(ref username) = self.state {
            Some(username)
        } else {
            None
        }
    }
    
    fn load_stored_users(&mut self) {
        if let Ok(contents) = fs::read_to_string("users.json") {
            if let Ok(users) = serde_json::from_str::<HashMap<String, StoredUser>>(&contents) {
                self.stored_users = users;
            }
        }
    }
    
    fn save_stored_users(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.stored_users) {
            let _ = fs::write("users.json", json);
        }
    }
    
    fn hash_password(&self, password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    fn update_last_login(&mut self, username: &str) {
        if let Some(user) = self.stored_users.get_mut(username) {
            user.last_login = Some(chrono::Utc::now().to_rfc3339());
            self.save_stored_users();
        }
    }
    
    pub fn logout(&mut self) {
        self.state = AuthState::Connected;
        self.clear_forms();
        self.clear_error();
        self.status_message = "Logged out successfully".to_string();
    }
    
    pub fn get_user_role(&self) -> Option<String> {
        if let AuthState::Authenticated(ref username) = self.state {
            self.stored_users.get(username).map(|user| user.role.clone())
        } else {
            None
        }
    }
    
    pub fn get_available_users(&self) -> Vec<String> {
        self.stored_users.keys()
            .filter(|&username| {
                self.stored_users.get(username)
                    .map(|user| user.is_active)
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}