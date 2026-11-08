#!/bin/bash
# prepare sd card
set -euo pipefail
cd "$(dirname "$0")/.."

disk="${1:-}"
if [[ -z "$disk" ]]; then
    diskutil list external physical
    echo
    echo "usage: $0 /dev/diskN    (pick the microSD card above - this erases it)"
    exit 1
fi
[[ -d build/DWIN_SET ]] || { echo "build/DWIN_SET is missing - run: make dwin"; exit 1; }

info="$(diskutil info "$disk")"
echo "$info" | grep -E "Device / Media Name|Disk Size|Device Location|Removable Media|Protocol" || true
bytes="$(echo "$info" | sed -n 's/.*Disk Size:.*(\([0-9]*\) Bytes).*/\1/p')"
# removable media limit
if ! echo "$info" | grep -qE "Removable Media: +Removable" || [[ -z "$bytes" || "$bytes" -gt 64000000000 ]]; then
    echo "Refusing: $disk doesn't look like a microSD card (must be removable media, 64 GB or less)."
    exit 1
fi

read -r -p "ERASE everything on $disk and write DWIN_SET? Type 'erase' to continue: " ok
[[ "$ok" == "erase" ]] || { echo "Cancelled."; exit 1; }

if [[ -n "$bytes" && "$bytes" -gt 9000000000 ]]; then
    # partition size limit
    diskutil partitionDisk "$disk" MBR FAT32 DWIN 8G "Free Space" EMPTY R
else
    diskutil eraseDisk FAT32 DWIN MBRFormat "$disk"
fi
part="${disk}s1"
diskutil unmount "$part"
echo "Reformatting with 4 KB clusters (needs your password)..."
sudo newfs_msdos -F 32 -c 8 -v DWIN "/dev/r$(basename "$part")"
diskutil mount "$part"
vol="/Volumes/DWIN"

cp -R build/DWIN_SET "$vol/"
# clean macos metadata
dot_clean -m "$vol" 2>/dev/null || true
find "$vol" -name '._*' -delete 2>/dev/null || true
mdutil -i off "$vol" >/dev/null 2>&1 || true
rm -rf "$vol/.Spotlight-V100" "$vol/.fseventsd" "$vol/.Trashes" 2>/dev/null || true
sync

echo
ls -l "$vol/DWIN_SET"
diskutil eject "$disk"
echo
echo "Done. With the panel powered OFF, insert the card, then power it on."
echo "Wait for the blue download screen to say it has finished, power off, remove the card,"
echo "and power on again: you should see the IntelliThings orb and 'Waiting for the controller...'."
