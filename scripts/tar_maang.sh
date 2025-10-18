#!/usr/bin/env fish
set -l ts (date +%F_%H-%M)
set -l dest ~/DevBackups/maang_backups/maang_$ts

# make backup folder
mkdir -p $dest

# fast local copy
cp -a --reflink=auto ~/maang/. $dest/

# remove build artifacts
find $dest -type d -name target -prune -exec rm -rf {} +; and find $dest -type f -name '*.d' -delete

# create compressed tar.gz archive
set -l archive ~/DevBackups/maang_backups/maang_$ts.tar.gz
tar -czf $archive -C (dirname $dest) (basename $dest)

echo "✅ Backup complete: $dest"
echo "📦 Archive created: $archive"
