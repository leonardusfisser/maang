#!/usr/bin/env bash
set -l ts (date +%F_%H-%M)
       set -l dest ~/DevBackups/maang_backups/maang_$ts
       mkdir -p $dest
       cp -a --reflink=auto ~/maang/. $dest/
       find $dest -type d -name target -prune -exec rm -rf {} +; and find $dest -type f -name '*.d' -delete

       echo "✅ Backup complete: $dest"