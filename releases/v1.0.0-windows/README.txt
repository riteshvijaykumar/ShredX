# ShredX v1.0.0 - Windows Release

## Overview
ShredX is a cross-platform disk sanitization utility that provides NIST SP 800-88 compliant data erasure for various storage devices including HDDs, SSDs, NVMe drives, USB storage, and SD cards.

## System Requirements
- **Operating System**: Windows 10/11 (x64 or x86)
- **Privileges**: Administrator access required
- **Memory**: 4GB RAM minimum
- **Storage**: 100MB free disk space
- **Hardware**: Compatible storage devices for sanitization

## Installation

### Quick Install
1. Extract the ZIP file to a temporary folder
2. Right-click `install.bat` and select "Run as administrator"
3. Follow the installation prompts
4. Launch ShredX from Desktop shortcut or Start Menu

### Manual Install
1. Extract files to desired location (e.g., `C:\Program Files\ShredX\`)
2. Create shortcuts as needed
3. Run `ShredX.exe` as Administrator

## Features

### Core Sanitization
- **NIST SP 800-88 Compliance**: Clear, Purge, and Destroy methods
- **DoD 5220.22-M**: Department of Defense approved algorithms
- **Gutmann Method**: 35-pass military-grade erasure
- **Custom Patterns**: User-defined overwrite patterns

### Device Support
- **Hard Disk Drives (HDDs)**: Traditional magnetic storage
- **Solid State Drives (SSDs)**: TRIM and Secure Erase support
- **NVMe Drives**: High-performance PCIe storage
- **USB Storage**: Flash drives and external storage
- **SD Cards**: Memory cards and multimedia storage

### Security Features
- **User Authentication**: Role-based access control (Admin, Operator, Viewer)
- **Certificate Generation**: Cryptographically signed erasure certificates
- **Audit Trails**: Comprehensive logging and compliance reports
- **Tamper Protection**: Secure operation validation

### User Interface
- **Modern GUI**: Clean, intuitive desktop application
- **Real-time Progress**: Live monitoring of sanitization operations
- **Device Detection**: Automatic hardware identification
- **Report Generation**: PDF certificates and compliance documentation

## Quick Start

1. **Launch Application**
   - Double-click Desktop shortcut
   - Or: Start Menu → ShredX → ShredX
   - **Important**: Always run as Administrator

2. **Authentication**
   - First run: Create admin account
   - Login with your credentials
   - Manage user accounts as needed

3. **Select Device**
   - View detected storage devices
   - Select target device for sanitization
   - Review device information

4. **Choose Method**
   - Select sanitization algorithm
   - Configure operation parameters
   - Set verification options

5. **Execute Sanitization**
   - Start the erasure process
   - Monitor real-time progress
   - Wait for completion

6. **Generate Certificate**
   - Download compliance certificate
   - Save audit logs
   - Archive documentation

## Safety & Compliance

### ⚠️ CRITICAL WARNING
- **Data Destruction**: This tool permanently destroys data
- **No Recovery**: Sanitized data cannot be recovered
- **Backup First**: Always backup important data before use
- **Verify Target**: Double-check selected devices

### NIST SP 800-88 Compliance
- Implements approved sanitization methods
- Generates compliant documentation
- Maintains audit trails
- Supports regulatory requirements

### Industry Standards
- DoD 5220.22-M compliance
- Common Criteria compatibility
- FIPS 140-2 approved algorithms
- International standards support

## Configuration

### Default Settings
```json
{
  "theme": "Dark",
  "auto_save_reports": true,
  "verification_enabled": true,
  "buffer_size": "16MB",
  "thread_count": 4
}
```

### Environment Variables
- `SHREDX_CONFIG_PATH`: Custom configuration file location
- `SHREDX_REPORTS_PATH`: Custom reports directory
- `SHREDX_LOG_LEVEL`: Logging verbosity (debug, info, warn, error)

## Troubleshooting

### Common Issues

1. **Access Denied**
   - Solution: Run as Administrator
   - Ensure user has necessary privileges

2. **Device Not Detected**
   - Solution: Check device connections
   - Verify driver installation
   - Try different USB ports

3. **Operation Failed**
   - Solution: Check device health
   - Verify sufficient system resources
   - Review error logs

4. **Certificate Generation Failed**
   - Solution: Check disk space
   - Verify write permissions
   - Review report settings

### Log Files
- **Application Logs**: `%APPDATA%\ShredX\logs\`
- **Operation Logs**: `%APPDATA%\ShredX\operations\`
- **Error Logs**: Windows Event Viewer → Application Logs

## Support

### Documentation
- User Guide: `docs\USER_GUIDE.md`
- Technical Guide: `docs\TECHNICAL_GUIDE.md`
- Compliance Guide: `docs\COMPLIANCE.md`

### Online Resources
- GitHub Repository: https://github.com/riteshvijaykumar/ShredX
- Issue Tracker: Report bugs and feature requests
- Documentation: Latest guides and tutorials

### Professional Support
- For enterprise deployments
- Custom compliance requirements
- Integration assistance
- Training and certification

## Uninstallation

### Automatic Uninstall
1. Navigate to installation directory
2. Right-click `uninstall.bat`
3. Select "Run as administrator"
4. Follow prompts to complete removal

### Manual Uninstall
1. Delete installation directory
2. Remove Desktop and Start Menu shortcuts
3. Clean registry entries (if needed)
4. Remove user data (optional)

## License
This software is licensed under the MIT License. See LICENSE file for full terms.

## Disclaimer
This tool is provided "as-is" without warranty. Users are responsible for proper backup and data management. Always verify compliance requirements for your specific use case.

---

**ShredX v1.0.0** - Professional Disk Sanitization for Windows
Copyright © 2025 SIH Team