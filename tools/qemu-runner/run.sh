#!/bin/bash
# QEMU runner for OpSys kernel development
set -e

ISO="${1:-build/opsys.iso}"

qemu-system-x86_64 \
    -machine q35 \
    -m 256M \
    -serial stdio \
    -cdrom "$ISO" \
    -no-reboot \
    -no-shutdown \
    "$@"
