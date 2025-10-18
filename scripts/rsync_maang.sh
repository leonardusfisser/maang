#!/usr/bin/env bash
set timestamp (date +%F_%H-%M)
set dest ~/maang/DevBackups/maang_backups/maang_$timestamp
rsync -av --progress ~/maang/ $dest --exclude 'target' --exclude '*.d'
echo "✅ Backup complete: $dest"