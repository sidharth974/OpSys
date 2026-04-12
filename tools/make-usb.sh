#!/bin/bash
# OpSys - Create Bootable USB Drive
# WARNING: This will ERASE the target USB drive!
#
# Usage: sudo ./tools/make-usb.sh /dev/sdX
#

set -e

ISO="build/opsys.iso"
DEVICE="$1"

if [ -z "$DEVICE" ]; then
    echo "OpSys Bootable USB Creator"
    echo ""
    echo "Usage: sudo $0 /dev/sdX"
    echo ""
    echo "Available drives:"
    lsblk -d -o NAME,SIZE,TYPE,MODEL | grep disk
    echo ""
    echo "WARNING: This will ERASE the selected drive!"
    exit 1
fi

if [ ! -f "$ISO" ]; then
    echo "Error: $ISO not found. Run 'make iso-release' first."
    exit 1
fi

if [ "$(id -u)" -ne 0 ]; then
    echo "Error: Must run as root (sudo)."
    exit 1
fi

# Safety check
if mount | grep -q "$DEVICE"; then
    echo "Error: $DEVICE has mounted partitions. Unmount them first:"
    mount | grep "$DEVICE"
    exit 1
fi

echo "========================================"
echo "  OpSys Bootable USB Creator"
echo "========================================"
echo ""
echo "  ISO:    $ISO ($(du -h "$ISO" | cut -f1))"
echo "  Target: $DEVICE"
echo ""
read -p "  This will ERASE $DEVICE. Continue? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo "Writing ISO to $DEVICE..."
dd if="$ISO" of="$DEVICE" bs=4M status=progress conv=fsync
sync

echo ""
echo "Done! OpSys is now bootable from $DEVICE"
echo ""
echo "To boot:"
echo "  1. Plug the USB into your target machine"
echo "  2. Enter BIOS/UEFI boot menu (usually F12, F2, or Del)"
echo "  3. Select the USB drive"
echo "  4. OpSys will boot with the graphical desktop"
echo ""
echo "Note: Serial console output goes to COM1 (115200 baud)"
