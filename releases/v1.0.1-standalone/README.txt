ShredX Standalone v1.0.1 - Professional Disk Sanitization Tool
====================================================================

QUICK START GUIDE
==================

1. INSTALLATION (Administrator Required)
   - Right-click "install-standalone.bat"
   - Select "Run as administrator"
   - Follow the on-screen prompts
   - Choose installation location

2. LAUNCHING SHREDX
   - Double-click the desktop shortcut "ShredX"
   - OR use Start Menu: All Programs > ShredX
   - OR run directly: [Install Directory]\Launch ShredX.bat
   
3. FIRST RUN SETUP
   - ShredX will create default user configuration
   - Set up Admin user with strong password
   - Configure security preferences

SYSTEM REQUIREMENTS
===================
- Windows 10/11 (64-bit)
- Administrator privileges (for disk operations)
- 100MB free disk space
- 4GB+ RAM recommended

KEY FEATURES
============
✓ NIST SP 800-88 Compliant Sanitization
✓ Multiple Wiping Algorithms (DoD, Gutmann, Custom)
✓ Real-time Progress Monitoring
✓ Comprehensive Audit Reports
✓ Certificate Generation
✓ SSD-Optimized Operations
✓ Role-Based Access Control
✓ Professional UI with Dark Theme

SECURITY NOTES
===============
⚠️  ADMINISTRATOR PRIVILEGES: ShredX requires Administrator privileges
    for low-level disk access. You will see UAC prompts during operation.

⚠️  DATA DESTRUCTION: Sanitization operations are IRREVERSIBLE.
    Always verify target devices before proceeding.

⚠️  POWER REQUIREMENTS: Ensure stable power during long operations.
    Use UPS for critical sanitization tasks.

USAGE GUIDELINES
================

Basic Operation:
1. Launch ShredX as Administrator
2. Select target device from the list
3. Choose sanitization method:
   - NIST Clear: Single secure overwrite (fast)
   - NIST Purge: Multiple pass overwrite (secure)
   - DoD 5220.22-M: Military standard (7-pass)
   - Custom: User-defined patterns

4. Configure options:
   - Verification: Verify overwrite completion
   - Reporting: Generate compliance certificates
   - Performance: Adjust thread count and buffer size

5. Start sanitization and monitor progress
6. Review completion report and certificates

Advanced Features:
- Device Analysis: View detailed device information
- Bulk Operations: Process multiple devices
- Scheduled Tasks: Automate recurring sanitization
- Compliance Reporting: Generate audit-ready certificates
- User Management: Control access with role-based permissions

COMPLIANCE STANDARDS
====================
ShredX meets the following industry standards:

• NIST SP 800-88 Revision 1
  - Clear: Single overwrite pass
  - Purge: Multiple cryptographically strong passes
  - Destroy: Physical destruction guidance

• DoD 5220.22-M
  - Standard 3-pass overwrite
  - Enhanced 7-pass for classified data

• Common Criteria Evaluation
• HIPAA Compliance Ready
• GDPR Data Protection Requirements

TROUBLESHOOTING
===============

Access Denied Errors:
- Ensure running as Administrator
- Close other applications using the target device
- Disable antivirus real-time protection temporarily
- Check device is not system/boot drive

Performance Issues:
- Reduce thread count in settings
- Decrease buffer size for older systems
- Enable SSD optimization for solid-state drives
- Ensure adequate free RAM

Cannot Detect Device:
- Refresh device list (F5)
- Check device connections
- Update device drivers
- Verify device is not in use by other applications

Application Won't Start:
- Check Windows compatibility
- Verify all files extracted properly
- Run Windows compatibility troubleshooter
- Check antivirus quarantine

SUPPORT & RESOURCES
===================

Documentation:
- Complete user manual: [Installation Directory]\docs\
- Video tutorials: Available on project website
- FAQ and knowledge base: Online support portal

Technical Support:
- GitHub Issues: Report bugs and feature requests
- Community Forum: User discussions and tips
- Professional Support: Enterprise licensing available

File Locations:
- Application: [Installation Directory]\ShredX.exe
- Configuration: [Installation Directory]\config.json
- Reports: [Installation Directory]\reports\
- Certificates: [Installation Directory]\certificates\
- Logs: [Installation Directory]\logs\

LEGAL & COMPLIANCE
==================

Data Protection:
ShredX is designed to permanently destroy data according to international
standards. Users are responsible for:
- Verifying data backup requirements before sanitization
- Compliance with local data protection laws
- Proper handling of classified or sensitive information
- Maintaining audit trails for regulatory compliance

Liability:
- Data loss from sanitization operations is irreversible
- Users assume full responsibility for data destruction
- Verify target devices and data before proceeding
- Test sanitization procedures in non-production environments

Export Controls:
Cryptographic sanitization methods may be subject to export controls
in some jurisdictions. Verify local regulations before international use.

WARRANTY & DISCLAIMER
======================

ShredX is provided "as is" without warranty of any kind. While designed
to meet industry standards, users should validate sanitization effectiveness
for their specific security requirements.

For enterprise deployments, professional validation and certification
services are available.

Version: 1.0.1-Standalone
Build Date: 2024
License: MIT (See LICENSE file)
Website: https://github.com/riteshvijaykumar/HDD-Tool

====================================================================
© 2024 ShredX Project. Professional Data Sanitization Solution.
====================================================================