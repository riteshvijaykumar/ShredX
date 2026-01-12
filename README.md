# 🔒 HDD Tool - Cross-Platform NIST SP 800-88 Compliant Data Sanitization

[![Build Status](https://github.com/riteshvijaykumar/HDD-Tool/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/riteshvijaykumar/HDD-Tool/actions)
[![Release](https://img.shields.io/github/v/release/riteshvijaykumar/HDD-Tool)](https://github.com/riteshvijaykumar/HDD-Tool/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)](https://github.com/riteshvijaykumar/HDD-Tool/releases)

## 🎯 Overview

HDD Tool is a **cross-platform** enterprise-grade data sanitization tool that fully complies with **NIST SP 800-88 Rev. 1 Guidelines for Media Sanitization**. Built with Rust for maximum performance and security, it provides comprehensive data destruction capabilities for all types of storage devices across Windows, Linux, and macOS platforms.

### 🌍 Cross-Platform Support

| Platform | Desktop App | Server | Status |
|----------|-------------|---------|---------|
| **Windows** | ✅ x64, x86 | ✅ x64 | Full Support |
| **Linux** | ✅ x64, ARM64 | ✅ x64, ARM64 | Full Support |
| **macOS** | ✅ Intel, Apple Silicon | ✅ Intel, Apple Silicon | Full Support |

## 📦 Download Latest Release

### 🪟 Windows (v1.0.0) - **Available Now!**
[![Download Windows](https://img.shields.io/badge/Download-Windows%20x64-blue?style=for-the-badge&logo=windows)](https://github.com/riteshvijaykumar/HDD-Tool/releases/latest/download/ShredX-v1.0.0-Windows-x64.zip)

- **Requirements**: Windows 10/11, Administrator privileges
- **Size**: 3.46 MB
- **Installation**: Extract and run `install.bat` as Administrator
- **Features**: Complete NIST SP 800-88 compliance, modern UI, certificate generation

### 🐧 Linux (Coming Soon)
- Ubuntu, Debian, CentOS packages
- AppImage for universal compatibility
- Complete server deployment scripts

### 🍎 macOS (Coming Soon)
- Intel and Apple Silicon support
- DMG installer with code signing
- Full macOS integration

## 🏗️ Quick Workflow Summary

```
🚀 Start → 🔍 Detection → 💻 UI → 👤 Mode Selection → 🛡️ NIST Methods → ✅ Validation → 
🔄 Sanitization → 📈 Monitoring → 🔍 Verification → 📄 Reports → 🏆 Certificates → ✅ Complete
```

## 🛡️ NIST 800-88 Compliance Levels

| Level | Security | Method | Use Case |
|-------|----------|---------|----------|
| **CLEAR** | Confidential | Single Pass Crypto Random | Software recovery protection |
| **PURGE** | Secret/Top Secret | 7-Pass Multi-Pattern | Laboratory recovery protection |
| **DESTROY** | Highest Security | Physical destruction guidance | Complete assurance |

## 📋 Key Features

- ✅ **NIST SP 800-88 Rev. 1 Compliant**
- ✅ **Hardware-based sanitization** (ATA/NVMe Secure Erase)
- ✅ **Multi-pass software sanitization** (DoD, Gutmann, Custom)
- ✅ **Real-time progress monitoring**
- ✅ **Comprehensive verification**
- ✅ **Digital certificate generation**
- ✅ **Audit trail logging**
- ✅ **Professional reporting**

## 📚 Project Report Sections

### 2. Base Paper
- [A Survey of Data Sanitization Techniques for Secure Storage Media Disposal](https://ieeexplore.ieee.org/document/XXXXXXX)
- [NIST Special Publication 800-88 Revision 1: Guidelines for Media Sanitization](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-88r1.pdf)

### 3. Existing System
- Most current data sanitization tools are platform-specific, lack compliance with NIST SP 800-88, or do not provide certificate-based audit trails.
- Existing open-source tools (e.g., DBAN, nwipe) offer limited reporting and lack modern UI or server integration.
- Enterprise solutions are often proprietary and expensive, with limited transparency.

### 4. Objectives of the Proposed System
- Develop a cross-platform, open-source data sanitization tool compliant with NIST SP 800-88 Rev. 1.
- Support both hardware-based (ATA/NVMe) and software-based (multi-pass) sanitization methods.
- Provide real-time monitoring, certificate generation, and audit trails.
- Enable server-based management and reporting for enterprise use.
- Deliver a modern, user-friendly interface and robust security.

### 5. Proposed Scopus Indexed Journal for Publication
- *Journal of Information Security and Applications* (Elsevier, Scopus Indexed)
- *IEEE Access* (Scopus Indexed)
- *International Journal of Information Security* (Springer, Scopus Indexed)

### 6. Timeline of Project Phases (Elaborated)

| Phase   | Description                                                      | Timeline         | Key Modules/Tasks                                                                                 |
|---------|------------------------------------------------------------------|------------------|--------------------------------------------------------------------------------------------------|
| Phase 1 | Requirement Analysis, Literature Survey, System Design           | Jan 2026 - Feb 2026 | - Literature review<br>- Requirement gathering<br>- System architecture design<br>- Technology stack selection<br>- UI/UX wireframes<br>- Database schema design<br>- Project planning & task allocation |
| Phase 2 | Core Implementation (Desktop App, Sanitization Engine, UI)       | Mar 2026 - Apr 2026 | - Rust project setup<br>- Core sanitization engine (NIST, DoD, Gutmann, ATA/NVMe)<br>- Device detection module<br>- User authentication module<br>- Desktop GUI (egui/eframe)<br>- Certificate/report generation<br>- Local data storage<br>- Unit testing for core modules |
| Phase 3 | Server Integration, Testing, Documentation, Paper Preparation    | May 2026 - Jun 2026 | - Server backend (Rust + Warp)<br>- REST API development<br>- PostgreSQL integration<br>- Web dashboard (optional)<br>- Integration testing<br>- Deployment scripts<br>- User manual & documentation<br>- Research paper writing<br>- Final review & submission |

---

## 🚀 Installation & Quick Start

### 📦 Download Release

Visit our [Releases Page](https://github.com/riteshvijaykumar/HDD-Tool/releases) and download the appropriate package:

#### Windows
- **x64**: `hdd-tool-windows-x64.zip` (Windows 10/11 64-bit)
- **x86**: `hdd-tool-windows-x86.zip` (Windows 10/11 32-bit)

#### Linux
- **x64**: `hdd-tool-linux-x64.tar.gz` (Intel/AMD 64-bit)
- **ARM64**: `hdd-tool-linux-arm64.tar.gz` (ARM 64-bit)

#### macOS
- **Intel**: `hdd-tool-macos-x64.tar.gz` (Intel Macs)
- **Apple Silicon**: `hdd-tool-macos-arm64.tar.gz` (M1/M2/M3 Macs)

### 🛠️ Installation

#### Windows
1. Extract `hdd-tool-windows-x64.zip`
2. **Run as Administrator**: `install.bat`
3. Launch from Start Menu or run `hdd-tool` in Command Prompt

#### Linux
```bash
tar -xzf hdd-tool-linux-x64.tar.gz
cd hdd-tool-linux-x64
sudo bash install.sh
hdd-tool
```

#### macOS
```bash
tar -xzf hdd-tool-macos-x64.tar.gz
cd hdd-tool-macos-x64
sudo bash install.sh
hdd-tool
```

### 🚀 Quick Start Guide

1. **Launch Application**: Run with elevated privileges
2. **Authentication**: Create account or login
3. **Select Device**: Choose target storage device
4. **Choose Method**: 
   - **Standard Mode**: NIST Clear/Purge buttons
   - **Advanced Mode**: Full algorithm selection
5. **Execute**: Monitor real-time progress
6. **Server Sync**: View results in web dashboard (optional)

## 📊 Supported Algorithms

### 🔧 Hardware-Based (Recommended)
- ATA Secure Erase (Standard)
- ATA Enhanced Secure Erase  
- NVMe Secure Erase
- NVMe Cryptographic Erase

### 📋 NIST 800-88 Methods
- **NIST Clear**: Single pass cryptographic random
- **NIST Purge**: 7-pass enhanced destruction

### 🛡️ Additional Standards
- DoD 5220.22-M (3-pass)
- DoD 5220.22-M ECE (7-pass)
- Gutmann Method (35-pass)
- Custom patterns

## 🏗️ Architecture

### Desktop Application (Rust + egui)
- **Cross-platform GUI**: Native performance on all platforms
- **Hardware Interface**: Direct drive access with elevated privileges
- **Real-time Monitoring**: Progress tracking and validation
- **Local Storage**: JSON-based user and certificate storage

### Server Component (PostgreSQL + Warp)
- **Database Backend**: PostgreSQL for scalable data storage
- **REST API**: Secure authentication and certificate management
- **Web Dashboard**: Browser-based management interface
- **Ubuntu Integration**: Automated deployment scripts

## 🔧 Technical Architecture

- **Core Engine**: `src/core/engine.rs` - Cross-platform sanitization engine
- **Platform Specific**: Windows (`windows` crate), Linux/macOS (`nix`, `libc`)
- **Hardware Interface**: `src/ata_commands.rs` - ATA/NVMe command implementation
- **Authentication**: `src/auth.rs` - User management and session handling
- **Server**: `src/server/` - PostgreSQL backend with REST API
- **Security**: `src/security/` - Certificate generation and validation
- **Reporting**: `src/reporting/` - Audit trails and compliance reports

## 🛠️ Development & Building

### Prerequisites
- **Rust**: 2024 Edition (1.75+)
- **Platform Dependencies**:
  - **Windows**: Visual Studio Build Tools, Windows SDK
  - **Linux**: `build-essential`, `libssl-dev`, `libgtk-3-dev`, `hdparm`
  - **macOS**: Xcode Command Line Tools, Homebrew

### Build from Source

#### Clone Repository
```bash
git clone https://github.com/riteshvijaykumar/HDD-Tool.git
cd HDD-Tool
```

#### Desktop Application
```bash
# Build desktop app
cargo build --release --bin hdd-tool

# Run locally
cargo run --bin hdd-tool
```

#### Server Component
```bash
# Build with server features
cargo build --release --bin hdd-tool-server --features server

# Run server (requires PostgreSQL)
export DATABASE_URL="postgresql://user:pass@localhost/hdd_tool_db"
cargo run --bin hdd-tool-server --features server
```

#### Cross-Platform Releases
```bash
# Linux/macOS
chmod +x scripts/build-release.sh
./scripts/build-release.sh v1.0.0

# Windows
scripts\build-release.bat v1.0.0

# Quick deployment
chmod +x deploy.sh
./deploy.sh local    # Local development
./deploy.sh build    # Build releases
./deploy.sh server   # Deploy server
./deploy.sh docker   # Docker deployment
```

### GitHub Actions
Automated CI/CD builds releases for all platforms:
- **Triggered by**: Git tags (`v*`) or manual workflow dispatch
- **Platforms**: Windows (x64, x86), Linux (x64, ARM64), macOS (Intel, Apple Silicon)
- **Artifacts**: ZIP/tar.gz packages with installers

## 🌐 Server Deployment

### Ubuntu Server (Recommended)
```bash
# Use automated setup script
chmod +x ubuntu-setup.sh
sudo ./ubuntu-setup.sh

# Manual deployment
./deploy.sh server
```

### Docker Deployment
```bash
# Quick start with Docker Compose
./deploy.sh docker

# Manual Docker
docker build -t hdd-tool-server .
docker run -d -p 3030:3030 --env-file .env hdd-tool-server
```

### Configuration
- **Environment Variables**: `DATABASE_URL`, `SERVER_PORT`, `SERVER_HOST`
- **Config File**: `config.json` for desktop app settings
- **SSL**: Automatic Let's Encrypt integration for production

## 📋 Documentation

- [📋 Complete Workflow Chart](NIST_800-88_WORKFLOW.md)
- [🔧 Advanced Features](ADVANCED_FEATURES.md)
- [🌐 Interactive Workflow](workflow_chart.html)
- [🐧 Ubuntu Server Setup](UBUNTU_SERVER_SETUP.md)
- [📖 Complete Guide](COMPLETE_GUIDE.md)
- [🚀 Release Scripts](scripts/README.md)

## ⚡ Performance Optimizations

- 16MB optimized buffer sizes
- Multi-threaded processing (4 threads)
- Sector-aligned operations
- Hardware acceleration when available

---

**⚠️ Security Notice**: This tool permanently destroys data. Ensure proper backups before use. Always verify compliance requirements for your specific use case.

### 🌍 Relevant United Nations Sustainable Development Goals (SDGs)

This project aligns with the following UN SDGs:

- **Goal 9: Industry, Innovation and Infrastructure**  
  Promotes innovation in secure data destruction and digital infrastructure.
- **Goal 12: Responsible Consumption and Production**  
  Encourages responsible disposal and reuse of storage devices by ensuring secure data sanitization.
- **Goal 16: Peace, Justice and Strong Institutions**  
  Supports strong institutions by enabling secure data handling, privacy, and compliance with regulations.
- **Goal 13: Climate Action**  
  Indirectly supports climate action by enabling safe reuse and recycling of hardware, reducing e-waste.

---