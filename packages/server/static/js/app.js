// HDD Tool Server Web Interface

let authToken = null;
let currentUser = null;
let selectedDrives = new Set();
let loginModal = null;

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    loginModal = new bootstrap.Modal(document.getElementById('loginModal'));
    
    // Check for existing authentication
    authToken = localStorage.getItem('authToken');
    if (authToken) {
        showDashboard();
        loadDrives();
        loadJobs();
    } else {
        showLoginScreen();
    }
    
    // Setup event listeners
    setupEventListeners();
});

function setupEventListeners() {
    // Login forms
    document.getElementById('login-form').addEventListener('submit', handleLogin);
    document.getElementById('modal-login-form').addEventListener('submit', handleModalLogin);
    
    // Sanitization form
    document.getElementById('sanitization-form').addEventListener('submit', handleSanitization);
    
    // Confirmation checkbox
    document.getElementById('confirm-erase').addEventListener('change', function() {
        const startButton = document.getElementById('start-sanitization');
        const hasSelectedDrives = selectedDrives.size > 0;
        startButton.disabled = !(this.checked && hasSelectedDrives);
    });
}

async function handleLogin(event) {
    event.preventDefault();
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    await login(username, password);
}

async function handleModalLogin(event) {
    event.preventDefault();
    const username = document.getElementById('modal-username').value;
    const password = document.getElementById('modal-password').value;
    await login(username, password);
    loginModal.hide();
}

async function login(username, password) {
    try {
        const response = await fetch('/api/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ username, password })
        });
        
        const data = await response.json();
        
        if (data.success) {
            authToken = data.data.token;
            currentUser = data.data.user;
            localStorage.setItem('authToken', authToken);
            
            showDashboard();
            loadDrives();
            loadJobs();
            showNotification('Login successful!', 'success');
        } else {
            showNotification(data.message || 'Login failed', 'error');
        }
    } catch (error) {
        console.error('Login error:', error);
        showNotification('Login failed. Please try again.', 'error');
    }
}

function logout() {
    authToken = null;
    currentUser = null;
    selectedDrives.clear();
    localStorage.removeItem('authToken');
    showLoginScreen();
    showNotification('Logged out successfully', 'info');
}

function showLoginScreen() {
    document.getElementById('login-screen').style.display = 'block';
    document.getElementById('dashboard').style.display = 'none';
    updateAuthSection();
}

function showDashboard() {
    document.getElementById('login-screen').style.display = 'none';
    document.getElementById('dashboard').style.display = 'block';
    updateAuthSection();
}

function showLoginModal() {
    loginModal.show();
}

function updateAuthSection() {
    const authSection = document.getElementById('auth-section');
    
    if (authToken && currentUser) {
        authSection.innerHTML = `
            <div class="dropdown">
                <button class="btn btn-outline-light dropdown-toggle" type="button" data-bs-toggle="dropdown">
                    <i class="fas fa-user"></i> ${currentUser.username} (${currentUser.role})
                </button>
                <ul class="dropdown-menu">
                    <li><a class="dropdown-item" href="#" onclick="logout()">
                        <i class="fas fa-sign-out-alt"></i> Logout
                    </a></li>
                </ul>
            </div>
        `;
    } else {
        authSection.innerHTML = `
            <button class="btn btn-outline-light" onclick="showLoginModal()">
                <i class="fas fa-sign-in-alt"></i> Login
            </button>
        `;
    }
}

async function loadDrives() {
    const drivesList = document.getElementById('drives-list');
    drivesList.innerHTML = '<div class="text-center"><div class="spinner-border" role="status"></div><p>Loading drives...</p></div>';
    
    try {
        const response = await fetch('/api/drives', {
            headers: {
                'Authorization': `Bearer ${authToken}`
            }
        });
        
        const data = await response.json();
        
        if (data.success) {
            displayDrives(data.data);
        } else {
            drivesList.innerHTML = '<p class="text-danger">Failed to load drives</p>';
        }
    } catch (error) {
        console.error('Error loading drives:', error);
        drivesList.innerHTML = '<p class="text-danger">Error loading drives</p>';
    }
}

function displayDrives(drives) {
    const drivesList = document.getElementById('drives-list');
    
    if (drives.length === 0) {
        drivesList.innerHTML = '<p class="text-muted">No drives found</p>';
        return;
    }
    
    const drivesHtml = drives.map(drive => `
        <div class="drive-item" onclick="toggleDriveSelection('${drive.id}')">
            <div class="drive-info">
                <div class="form-check">
                    <input class="form-check-input" type="checkbox" id="drive-${drive.id}" 
                           onchange="handleDriveSelection('${drive.id}', this.checked)">
                </div>
                <div class="drive-details">
                    <h6 class="mb-1">
                        <i class="fas fa-hdd"></i> ${drive.name}
                        <span class="drive-type ${drive.drive_type.toLowerCase()}">${drive.drive_type}</span>
                    </h6>
                    <p class="mb-1">${drive.model || 'Unknown Model'}</p>
                    <small class="drive-size">${formatBytes(drive.size)} • Serial: ${drive.serial || 'N/A'}</small>
                </div>
                <div class="drive-status">
                    ${drive.is_connected ? 
                        '<span class="badge bg-success">Connected</span>' : 
                        '<span class="badge bg-danger">Disconnected</span>'
                    }
                </div>
            </div>
        </div>
    `).join('');
    
    drivesList.innerHTML = drivesHtml;
}

function toggleDriveSelection(driveId) {
    const checkbox = document.getElementById(`drive-${driveId}`);
    checkbox.checked = !checkbox.checked;
    handleDriveSelection(driveId, checkbox.checked);
}

function handleDriveSelection(driveId, isSelected) {
    const driveItem = document.getElementById(`drive-${driveId}`).closest('.drive-item');
    
    if (isSelected) {
        selectedDrives.add(driveId);
        driveItem.classList.add('selected');
    } else {
        selectedDrives.delete(driveId);
        driveItem.classList.remove('selected');
    }
    
    // Update sanitization button state
    const confirmCheckbox = document.getElementById('confirm-erase');
    const startButton = document.getElementById('start-sanitization');
    startButton.disabled = !(confirmCheckbox.checked && selectedDrives.size > 0);
}

async function scanDrives() {
    const button = event.target;
    const originalText = button.innerHTML;
    button.innerHTML = '<i class="fas fa-spinner fa-spin"></i> Scanning...';
    button.disabled = true;
    
    try {
        const response = await fetch('/api/drives/scan', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${authToken}`
            }
        });
        
        const data = await response.json();
        
        if (data.success) {
            showNotification('Drive scan completed', 'success');
            await loadDrives();
        } else {
            showNotification('Drive scan failed', 'error');
        }
    } catch (error) {
        console.error('Error scanning drives:', error);
        showNotification('Drive scan failed', 'error');
    } finally {
        button.innerHTML = originalText;
        button.disabled = false;
    }
}

async function handleSanitization(event) {
    event.preventDefault();
    
    if (selectedDrives.size === 0) {
        showNotification('Please select at least one drive', 'warning');
        return;
    }
    
    const method = document.getElementById('sanitization-method').value;
    const passes = parseInt(document.getElementById('passes').value);
    const verify = document.getElementById('verify').checked;
    
    const confirmResult = confirm(
        `Are you sure you want to sanitize ${selectedDrives.size} drive(s) using ${method}?\n\n` +
        `⚠️ THIS WILL PERMANENTLY DESTROY ALL DATA!\n\n` +
        `Selected drives: ${Array.from(selectedDrives).join(', ')}\n` +
        `Method: ${method}\n` +
        `Passes: ${passes}\n` +
        `Verification: ${verify ? 'Yes' : 'No'}`
    );
    
    if (!confirmResult) {
        return;
    }
    
    try {
        const response = await fetch('/api/sanitization/start', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify({
                drive_ids: Array.from(selectedDrives),
                method: method,
                passes: passes,
                verify: verify
            })
        });
        
        const data = await response.json();
        
        if (data.success) {
            showNotification('Sanitization job started successfully', 'success');
            // Reset form
            selectedDrives.clear();
            document.getElementById('sanitization-form').reset();
            document.querySelectorAll('.drive-item').forEach(item => {
                item.classList.remove('selected');
                item.querySelector('input[type="checkbox"]').checked = false;
            });
            document.getElementById('start-sanitization').disabled = true;
            
            // Reload jobs
            await loadJobs();
        } else {
            showNotification(data.message || 'Failed to start sanitization', 'error');
        }
    } catch (error) {
        console.error('Error starting sanitization:', error);
        showNotification('Failed to start sanitization', 'error');
    }
}

async function loadJobs() {
    const jobsList = document.getElementById('jobs-list');
    
    try {
        const response = await fetch('/api/sanitization/jobs', {
            headers: {
                'Authorization': `Bearer ${authToken}`
            }
        });
        
        const data = await response.json();
        
        if (data.success) {
            displayJobs(data.data);
        } else {
            jobsList.innerHTML = '<p class="text-danger">Failed to load jobs</p>';
        }
    } catch (error) {
        console.error('Error loading jobs:', error);
        jobsList.innerHTML = '<p class="text-danger">Error loading jobs</p>';
    }
}

function displayJobs(jobs) {
    const jobsList = document.getElementById('jobs-list');
    
    if (jobs.length === 0) {
        jobsList.innerHTML = '<p class="text-muted">No sanitization jobs</p>';
        return;
    }
    
    const jobsHtml = jobs.map(job => `
        <div class="job-item">
            <div class="d-flex justify-content-between align-items-center">
                <div>
                    <h6 class="mb-1">
                        <i class="fas fa-tasks"></i> Job ${job.id.substring(0, 8)}
                        <span class="job-status ${job.status}">${job.status}</span>
                    </h6>
                    <p class="mb-1">
                        Drives: ${job.drive_ids.join(', ')} • 
                        Method: ${job.method} • 
                        Passes: ${job.passes}
                    </p>
                    <small class="text-muted">
                        ${job.started_at ? `Started: ${new Date(job.started_at).toLocaleString()}` : 'Not started'}
                        ${job.completed_at ? ` • Completed: ${new Date(job.completed_at).toLocaleString()}` : ''}
                    </small>
                </div>
                <div class="text-end">
                    <div class="h6 mb-0">${job.progress.toFixed(1)}%</div>
                    ${job.status === 'running' ? '<i class="fas fa-spinner fa-spin text-primary"></i>' : ''}
                </div>
            </div>
            ${job.status === 'running' || job.progress > 0 ? `
                <div class="progress-container">
                    <div class="progress" style="height: 8px;">
                        <div class="progress-bar ${job.status === 'completed' ? 'bg-success' : 'bg-primary'}" 
                             style="width: ${job.progress}%"></div>
                    </div>
                </div>
            ` : ''}
            ${job.error_message ? `
                <div class="alert alert-danger mt-2 mb-0" style="padding: 0.5rem;">
                    <small><i class="fas fa-exclamation-triangle"></i> ${job.error_message}</small>
                </div>
            ` : ''}
        </div>
    `).join('');
    
    jobsList.innerHTML = jobsHtml;
}

function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function showNotification(message, type = 'info') {
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `alert alert-${type === 'error' ? 'danger' : type} alert-dismissible fade show position-fixed`;
    notification.style.cssText = 'top: 20px; right: 20px; z-index: 9999; min-width: 300px;';
    
    notification.innerHTML = `
        ${message}
        <button type="button" class="btn-close" data-bs-dismiss="alert"></button>
    `;
    
    document.body.appendChild(notification);
    
    // Auto remove after 5 seconds
    setTimeout(() => {
        if (document.body.contains(notification)) {
            notification.remove();
        }
    }, 5000);
}

// Auto refresh jobs every 5 seconds if authenticated
setInterval(() => {
    if (authToken && document.getElementById('dashboard').style.display !== 'none') {
        loadJobs();
    }
}, 5000);