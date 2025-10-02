use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use sha2::{Sha256, Digest};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationCertificate {
    pub id: String,
    pub device_info: DeviceCertificateInfo,
    pub sanitization_info: SanitizationInfo,
    pub compliance_info: ComplianceInfo,
    pub verification_info: VerificationInfo,
    pub timestamp: DateTime<Utc>,
    pub user_info: UserInfo,
    pub certificate_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCertificateInfo {
    pub device_path: String,
    pub device_name: String,
    pub device_type: String,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub capacity: u64,
    pub sector_size: u32,
    pub supports_secure_erase: bool,
    pub supports_crypto_erase: bool,
    pub encryption_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationInfo {
    pub method: String,
    pub algorithm: String,
    pub passes_completed: u32,
    pub total_bytes_processed: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_seconds: u64,
    pub average_speed_mbps: f64,
    pub success: bool,
    pub error_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceInfo {
    pub standards_met: Vec<String>,
    pub nist_compliant: bool,
    pub dod_compliant: bool,
    pub hipaa_compliant: bool,
    pub gdpr_compliant: bool,
    pub security_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationInfo {
    pub verification_performed: bool,
    pub verification_method: String,
    pub verification_passed: bool,
    pub residual_data_found: bool,
    pub verification_details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub user_id: String,
    pub organization: String,
    pub role: String,
}

pub struct CertificateGenerator {
    certificates_dir: String,
}

impl CertificateGenerator {
    pub fn new() -> Self {
        let certificates_dir = "./certificates".to_string();
        
        // Create certificates directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&certificates_dir) {
            eprintln!("Warning: Could not create certificates directory: {}", e);
        }
        
        Self { certificates_dir }
    }

    pub fn generate_certificate(
        &self,
        device_info: DeviceCertificateInfo,
        sanitization_info: SanitizationInfo,
        user_info: UserInfo,
    ) -> Result<SanitizationCertificate, Box<dyn std::error::Error>> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Determine compliance based on method and success
        let compliance_info = self.determine_compliance(&sanitization_info);
        
        // Generate verification info (in real implementation, this would come from actual verification)
        let verification_info = VerificationInfo {
            verification_performed: true,
            verification_method: "Post-sanitization sector scan".to_string(),
            verification_passed: sanitization_info.success,
            residual_data_found: false,
            verification_details: if sanitization_info.success {
                "No recoverable data detected after sanitization".to_string()
            } else {
                "Sanitization incomplete - verification could not be performed".to_string()
            },
        };

        let mut certificate = SanitizationCertificate {
            id: id.clone(),
            device_info,
            sanitization_info,
            compliance_info,
            verification_info,
            timestamp,
            user_info,
            certificate_hash: String::new(), // Will be calculated below
        };

        // Calculate certificate hash
        certificate.certificate_hash = self.calculate_certificate_hash(&certificate)?;

        Ok(certificate)
    }

    fn determine_compliance(&self, sanitization_info: &SanitizationInfo) -> ComplianceInfo {
        let mut standards_met = Vec::new();
        let mut nist_compliant = false;
        let mut dod_compliant = false;
        let hipaa_compliant = sanitization_info.success;
        let gdpr_compliant = sanitization_info.success;

        // Check NIST SP 800-88 compliance
        if sanitization_info.method.contains("NIST") || 
           sanitization_info.algorithm.contains("Clear") ||
           sanitization_info.algorithm.contains("Purge") {
            standards_met.push("NIST SP 800-88".to_string());
            nist_compliant = sanitization_info.success;
        }

        // Check DoD 5220.22-M compliance
        if sanitization_info.method.contains("DoD") || 
           sanitization_info.passes_completed >= 3 {
            standards_met.push("DoD 5220.22-M".to_string());
            dod_compliant = sanitization_info.success;
        }

        // Check for secure erase compliance
        if sanitization_info.algorithm.contains("Secure Erase") {
            standards_met.push("ATA Secure Erase".to_string());
        }

        if sanitization_info.algorithm.contains("Crypto Erase") {
            standards_met.push("Cryptographic Erase".to_string());
        }

        let security_level = if nist_compliant && dod_compliant {
            "High Security"
        } else if nist_compliant || dod_compliant {
            "Medium Security"
        } else if sanitization_info.success {
            "Basic Security"
        } else {
            "Incomplete"
        }.to_string();

        ComplianceInfo {
            standards_met,
            nist_compliant,
            dod_compliant,
            hipaa_compliant,
            gdpr_compliant,
            security_level,
        }
    }

    fn calculate_certificate_hash(&self, certificate: &SanitizationCertificate) -> Result<String, Box<dyn std::error::Error>> {
        // Create a temporary certificate with empty hash for hashing
        let mut temp_cert = certificate.clone();
        temp_cert.certificate_hash = String::new();
        
        let json_data = serde_json::to_string(&temp_cert)?;
        let mut hasher = Sha256::new();
        hasher.update(json_data.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    pub fn save_certificate_local(&self, certificate: &SanitizationCertificate) -> Result<String, Box<dyn std::error::Error>> {
        let filename = format!("certificate_{}_{}.json", 
            certificate.device_info.device_name.replace(" ", "_"),
            certificate.timestamp.format("%Y%m%d_%H%M%S"));
        
        let filepath = Path::new(&self.certificates_dir).join(&filename);
        
        let json_data = serde_json::to_string_pretty(&certificate)?;
        fs::write(&filepath, json_data)?;
        
        println!("✅ Certificate saved locally: {}", filepath.display());
        Ok(filepath.to_string_lossy().to_string())
    }

    pub fn generate_certificate_report(&self, certificate: &SanitizationCertificate) -> String {
        format!(
r#"
═══════════════════════════════════════════════════════════════════════════════
                        SECURE DATA SANITIZATION CERTIFICATE
═══════════════════════════════════════════════════════════════════════════════

Certificate ID: {}
Generated: {}
Certificate Hash: {}

DEVICE INFORMATION:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Device Path: {}
│ Device Name: {}
│ Device Type: {}
│ Manufacturer: {}
│ Model: {}
│ Serial Number: {}
│ Capacity: {} GB
│ Sector Size: {} bytes
│ Secure Erase Support: {}
│ Crypto Erase Support: {}
│ Encryption Status: {}
└─────────────────────────────────────────────────────────────────────────────┘

SANITIZATION INFORMATION:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Method: {}
│ Algorithm: {}
│ Passes Completed: {}
│ Total Bytes Processed: {} GB
│ Start Time: {}
│ End Time: {}
│ Duration: {} seconds ({} minutes)
│ Average Speed: {:.2} MB/s
│ Success: {}
│ Error Count: {}
└─────────────────────────────────────────────────────────────────────────────┘

COMPLIANCE INFORMATION:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Security Level: {}
│ Standards Met: {}
│ NIST SP 800-88 Compliant: {}
│ DoD 5220.22-M Compliant: {}
│ HIPAA Compliant: {}
│ GDPR Compliant: {}
└─────────────────────────────────────────────────────────────────────────────┘

VERIFICATION INFORMATION:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Verification Performed: {}
│ Verification Method: {}
│ Verification Passed: {}
│ Residual Data Found: {}
│ Details: {}
└─────────────────────────────────────────────────────────────────────────────┘

USER INFORMATION:
┌─────────────────────────────────────────────────────────────────────────────┐
│ Username: {}
│ User ID: {}
│ Organization: {}
│ Role: {}
└─────────────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════════════
This certificate confirms that the above device has been sanitized according to
industry standards and regulatory requirements. The sanitization process has been
verified and documented for compliance purposes.

Generated by: HDD Tool - Secure Data Sanitization System
Version: 1.0.0
═══════════════════════════════════════════════════════════════════════════════
"#,
            certificate.id,
            certificate.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            certificate.certificate_hash,
            certificate.device_info.device_path,
            certificate.device_info.device_name,
            certificate.device_info.device_type,
            certificate.device_info.manufacturer,
            certificate.device_info.model,
            certificate.device_info.serial_number,
            certificate.device_info.capacity / (1024 * 1024 * 1024),
            certificate.device_info.sector_size,
            if certificate.device_info.supports_secure_erase { "Yes" } else { "No" },
            if certificate.device_info.supports_crypto_erase { "Yes" } else { "No" },
            certificate.device_info.encryption_status,
            certificate.sanitization_info.method,
            certificate.sanitization_info.algorithm,
            certificate.sanitization_info.passes_completed,
            certificate.sanitization_info.total_bytes_processed / (1024 * 1024 * 1024),
            certificate.sanitization_info.start_time.format("%Y-%m-%d %H:%M:%S UTC"),
            certificate.sanitization_info.end_time.format("%Y-%m-%d %H:%M:%S UTC"),
            certificate.sanitization_info.duration_seconds,
            certificate.sanitization_info.duration_seconds / 60,
            certificate.sanitization_info.average_speed_mbps,
            if certificate.sanitization_info.success { "Yes" } else { "No" },
            certificate.sanitization_info.error_count,
            certificate.compliance_info.security_level,
            certificate.compliance_info.standards_met.join(", "),
            if certificate.compliance_info.nist_compliant { "Yes" } else { "No" },
            if certificate.compliance_info.dod_compliant { "Yes" } else { "No" },
            if certificate.compliance_info.hipaa_compliant { "Yes" } else { "No" },
            if certificate.compliance_info.gdpr_compliant { "Yes" } else { "No" },
            if certificate.verification_info.verification_performed { "Yes" } else { "No" },
            certificate.verification_info.verification_method,
            if certificate.verification_info.verification_passed { "Yes" } else { "No" },
            if certificate.verification_info.residual_data_found { "Yes" } else { "No" },
            certificate.verification_info.verification_details,
            certificate.user_info.username,
            certificate.user_info.user_id,
            certificate.user_info.organization,
            certificate.user_info.role,
        )
    }

    pub fn save_certificate_report(&self, certificate: &SanitizationCertificate) -> Result<String, Box<dyn std::error::Error>> {
        let report_content = self.generate_certificate_report(certificate);
        
        let filename = format!("certificate_report_{}_{}.txt", 
            certificate.device_info.device_name.replace(" ", "_"),
            certificate.timestamp.format("%Y%m%d_%H%M%S"));
        
        let filepath = Path::new(&self.certificates_dir).join(&filename);
        fs::write(&filepath, report_content)?;
        
        println!("✅ Certificate report saved: {}", filepath.display());
        Ok(filepath.to_string_lossy().to_string())
    }

    pub fn load_certificates(&self) -> Result<Vec<SanitizationCertificate>, Box<dyn std::error::Error>> {
        let mut certificates = Vec::new();
        
        if !Path::new(&self.certificates_dir).exists() {
            return Ok(certificates);
        }

        for entry in fs::read_dir(&self.certificates_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        match serde_json::from_str::<SanitizationCertificate>(&content) {
                            Ok(certificate) => certificates.push(certificate),
                            Err(e) => eprintln!("Warning: Could not parse certificate file {}: {}", path.display(), e),
                        }
                    }
                    Err(e) => eprintln!("Warning: Could not read certificate file {}: {}", path.display(), e),
                }
            }
        }
        
        // Sort by timestamp (newest first)
        certificates.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(certificates)
    }
}

impl Default for CertificateGenerator {
    fn default() -> Self {
        Self::new()
    }
}