// Upload Area
const uploadArea = document.getElementById('uploadArea');
const fileInput = document.getElementById('fileInput');
const uploadProgress = document.getElementById('uploadProgress');
const progressList = document.getElementById('progressList');
const mediaGrid = document.getElementById('mediaGrid');

// Click to upload
uploadArea.addEventListener('click', () => {
    fileInput.click();
});

// Drag and drop
uploadArea.addEventListener('dragover', (e) => {
    e.preventDefault();
    uploadArea.classList.add('dragging');
});

uploadArea.addEventListener('dragleave', () => {
    uploadArea.classList.remove('dragging');
});

uploadArea.addEventListener('drop', (e) => {
    e.preventDefault();
    uploadArea.classList.remove('dragging');

    const files = Array.from(e.dataTransfer.files);
    uploadFiles(files);
});

// File input change
fileInput.addEventListener('change', (e) => {
    const files = Array.from(e.target.files);
    uploadFiles(files);
    fileInput.value = ''; // Reset input
});

// Upload files
async function uploadFiles(files) {
    uploadProgress.style.display = 'block';
    progressList.innerHTML = '';

    for (const file of files) {
        await uploadFile(file);
    }

    setTimeout(() => {
        uploadProgress.style.display = 'none';
        location.reload(); // Reload to show new files
    }, 1000);
}

async function uploadFile(file) {
    const progressItem = document.createElement('div');
    progressItem.className = 'progress-item';
    progressItem.innerHTML = `
            <div class="progress-filename">${file.name}</div>
            <div class="progress-bar">
                <div class="progress-fill" style="width: 0%" id="progress-${file.name}"></div>
            </div>
        `;
    progressList.appendChild(progressItem);

    const formData = new FormData();
    formData.append('file', file);

    try {
        const response = await fetch('/dashboard/upload', {
            method: 'POST',
            body: formData
        });

        if (response.ok) {
            document.getElementById(`progress-${file.name}`).style.width = '100%';
            showToast('File uploaded successfully!', 'success');
        } else {
            showToast('Upload failed: ' + file.name, 'error');
        }
    } catch (error) {
        showToast('Upload error: ' + error.message, 'error');
    }
}

// Copy URL to clipboard
function copyUrl(url) {
    const fullUrl = window.location.origin + url;
    navigator.clipboard.writeText(fullUrl).then(() => {
        showToast('URL copied to clipboard!', 'success');
    });
}

// Delete file
async function deleteFile(id) {
    if (!confirm('Are you sure you want to delete this file?')) {
        return;
    }

    try {
        const response = await fetch(`/dashboard/media/${id}`, {
            method: 'DELETE'
        });

        if (response.ok) {
            showToast('File deleted successfully!', 'success');
            setTimeout(() => location.reload(), 1000);
        } else {
            showToast('Delete failed', 'error');
        }
    } catch (error) {
        showToast('Error: ' + error.message, 'error');
    }
}

// Filter tabs
document.querySelectorAll('.filter-tab').forEach(tab => {
    tab.addEventListener('click', () => {
        // Update active tab
        document.querySelectorAll('.filter-tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');

        // Filter media items
        const filter = tab.dataset.filter;
        document.querySelectorAll('.media-item').forEach(item => {
            if (filter === 'all' || item.dataset.type === filter) {
                item.style.display = 'block';
            } else {
                item.style.display = 'none';
            }
        });
    });
});

// Show toast notification
function showToast(message, type) {
    const toast = document.getElementById('toast');
    const toastMessage = document.getElementById('toastMessage');

    toastMessage.textContent = message;
    toast.className = `toast show ${type}`;

    setTimeout(() => {
        toast.classList.remove('show');
    }, 3000);
}