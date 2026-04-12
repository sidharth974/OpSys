#!/bin/bash
# Record an OpSys demo video by scripting QEMU keypresses and taking screenshots.
set -e

ISO="build/opsys.iso"
OUTDIR="/tmp/opsys_frames"
rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"

FIFO="/tmp/opsys_monitor"
rm -f "$FIFO"
mkfifo "$FIFO"

echo "Starting QEMU..."
qemu-system-x86_64 \
    -machine q35,usb=off \
    -m 256M \
    -serial none \
    -cdrom "$ISO" \
    -display none \
    -no-reboot -no-shutdown \
    -monitor pipe:"$FIFO" \
    -vga std &
QEMU_PID=$!

# Helper: send a command to QEMU monitor
qcmd() {
    echo "$1" > "${FIFO}.in"
    sleep 0.3
}

# Helper: take a screenshot
frame=0
snap() {
    local f=$(printf "%s/frame_%04d.ppm" "$OUTDIR" "$frame")
    qcmd "screendump $f"
    frame=$((frame + 1))
    sleep 0.5
}

# Helper: type a string via sendkey (alphanumeric + common symbols)
typestr() {
    local str="$1"
    for ((i=0; i<${#str}; i++)); do
        local ch="${str:$i:1}"
        case "$ch" in
            [a-z]) qcmd "sendkey $ch" ;;
            [A-Z]) qcmd "sendkey shift-$(echo $ch | tr A-Z a-z)" ;;
            [0-9]) qcmd "sendkey $ch" ;;
            ' ')   qcmd "sendkey spc" ;;
            '/')   qcmd "sendkey slash" ;;
            '-')   qcmd "sendkey minus" ;;
            '.')   qcmd "sendkey dot" ;;
            '_')   qcmd "sendkey shift-minus" ;;
            '=')   qcmd "sendkey equal" ;;
            ',')   qcmd "sendkey comma" ;;
            '!')   qcmd "sendkey shift-1" ;;
            '@')   qcmd "sendkey shift-2" ;;
            '#')   qcmd "sendkey shift-3" ;;
            '*')   qcmd "sendkey shift-8" ;;
        esac
    done
}

enter() {
    qcmd "sendkey ret"
}

echo "Waiting for boot..."
sleep 12

echo "Taking boot screenshot..."
snap  # Desktop at startup
sleep 1
snap

# Demo: type neofetch
echo "Typing: neofetch"
typestr "neofetch"
sleep 0.5
snap
enter
sleep 1
snap
snap

# Demo: type ls /
echo "Typing: ls /"
typestr "ls /"
snap
enter
sleep 1
snap
snap

# Demo: type cat /etc/os-release
echo "Typing: cat /etc/os-release"
typestr "cat /etc/os-release"
snap
enter
sleep 1
snap
snap

# Demo: type ifconfig
echo "Typing: ifconfig"
typestr "ifconfig"
snap
enter
sleep 1
snap
snap

# Demo: type ai
echo "Typing: ai"
typestr "ai"
snap
enter
sleep 1
snap
snap

# Demo: type lspci
echo "Typing: lspci"
typestr "lspci"
snap
enter
sleep 1
snap
snap

# Demo: type security
echo "Typing: security"
typestr "security"
snap
enter
sleep 1
snap
snap

# Final pause
sleep 1
snap
snap
snap

# Cleanup QEMU
echo "quit" > "${FIFO}.in"
wait $QEMU_PID 2>/dev/null || true
rm -f "$FIFO" "${FIFO}.in" "${FIFO}.out"

# Count frames
NFRAMES=$(ls "$OUTDIR"/frame_*.ppm 2>/dev/null | wc -l)
echo "Captured $NFRAMES frames"

if [ "$NFRAMES" -lt 2 ]; then
    echo "Not enough frames captured. Aborting."
    exit 1
fi

# Convert PPM to PNG
echo "Converting frames..."
for f in "$OUTDIR"/frame_*.ppm; do
    convert "$f" "${f%.ppm}.png" 2>/dev/null
done

# Create video with ffmpeg
echo "Creating video..."
ffmpeg -y -framerate 2 -i "$OUTDIR/frame_%04d.png" \
    -vf "scale=1280:800" \
    -c:v libx264 -pix_fmt yuv420p -crf 23 \
    ~/Desktop/opsys_demo.mp4 2>/dev/null

# Also create GIF
echo "Creating GIF..."
ffmpeg -y -framerate 2 -i "$OUTDIR/frame_%04d.png" \
    -vf "scale=960:600,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" \
    ~/Desktop/opsys_demo.gif 2>/dev/null

echo ""
echo "Done!"
echo "  Video: ~/Desktop/opsys_demo.mp4"
echo "  GIF:   ~/Desktop/opsys_demo.gif"
ls -lh ~/Desktop/opsys_demo.mp4 ~/Desktop/opsys_demo.gif 2>/dev/null
