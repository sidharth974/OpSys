export PATH := $(HOME)/.cargo/bin:$(PATH)

KERNEL_ELF = target/x86_64-unknown-none/debug/opsys-kernel
KERNEL_RELEASE = target/x86_64-unknown-none/release/opsys-kernel
ISO_DIR = build/iso_root
ISO = build/opsys.iso
LIMINE_DIR = build/limine

.PHONY: all kernel kernel-release iso run run-release debug clean limine

all: run

# Build kernel (debug)
kernel:
	cargo build -p opsys-kernel

# Build kernel (release)
kernel-release:
	cargo build -p opsys-kernel --release

# Clone/update Limine bootloader
limine:
	@if [ ! -d "$(LIMINE_DIR)" ]; then \
		git clone https://github.com/limine-bootloader/limine.git --branch=v8.x-binary --depth=1 $(LIMINE_DIR); \
	fi
	@if [ ! -f "$(LIMINE_DIR)/limine" ]; then \
		$(MAKE) -C $(LIMINE_DIR); \
	fi

# Build bootable ISO
iso: kernel limine
	@mkdir -p $(ISO_DIR)/boot
	@mkdir -p $(ISO_DIR)/boot/limine
	@mkdir -p $(ISO_DIR)/EFI/BOOT
	@cp $(KERNEL_ELF) $(ISO_DIR)/boot/kernel
	@cp limine.conf $(ISO_DIR)/boot/limine/limine.conf
	@cp $(LIMINE_DIR)/limine-bios.sys $(ISO_DIR)/boot/limine/
	@cp $(LIMINE_DIR)/limine-bios-cd.bin $(ISO_DIR)/boot/limine/
	@cp $(LIMINE_DIR)/limine-uefi-cd.bin $(ISO_DIR)/boot/limine/
	@cp $(LIMINE_DIR)/BOOTX64.EFI $(ISO_DIR)/EFI/BOOT/
	@cp $(LIMINE_DIR)/BOOTIA32.EFI $(ISO_DIR)/EFI/BOOT/
	xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		$(ISO_DIR) -o $(ISO)
	@$(LIMINE_DIR)/limine bios-install $(ISO)
	@echo "ISO built: $(ISO)"

# Build release ISO
iso-release: kernel-release limine
	@mkdir -p $(ISO_DIR)/boot
	@mkdir -p $(ISO_DIR)/boot/limine
	@mkdir -p $(ISO_DIR)/EFI/BOOT
	@cp $(KERNEL_RELEASE) $(ISO_DIR)/boot/kernel
	@cp limine.conf $(ISO_DIR)/boot/limine/limine.conf
	@cp $(LIMINE_DIR)/limine-bios.sys $(ISO_DIR)/boot/limine/
	@cp $(LIMINE_DIR)/limine-bios-cd.bin $(ISO_DIR)/boot/limine/
	@cp $(LIMINE_DIR)/limine-uefi-cd.bin $(ISO_DIR)/boot/limine/
	@cp $(LIMINE_DIR)/BOOTX64.EFI $(ISO_DIR)/EFI/BOOT/
	@cp $(LIMINE_DIR)/BOOTIA32.EFI $(ISO_DIR)/EFI/BOOT/
	xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		$(ISO_DIR) -o $(ISO)
	@$(LIMINE_DIR)/limine bios-install $(ISO)

# Run in QEMU
# -machine usb=off: forces PS/2 mouse (our driver)
# -display gtk,show-cursor=off: hides QEMU's host cursor so only our OS cursor shows
# Click inside QEMU window to grab mouse, Ctrl+Alt+G to release
run: iso
	qemu-system-x86_64 \
		-machine q35,usb=off \
		-m 256M \
		-serial stdio \
		-cdrom $(ISO) \
		-display gtk,show-cursor=off \
		-no-reboot \
		-no-shutdown

# Run with KVM acceleration (real CPU features: AVX, SMEP, SMAP)
run-kvm: iso
	qemu-system-x86_64 \
		-machine q35,usb=off \
		-cpu host \
		-enable-kvm \
		-m 4G \
		-smp 4 \
		-serial stdio \
		-cdrom $(ISO) \
		-no-reboot

# Run with GDB server (paused)
debug: iso
	qemu-system-x86_64 \
		-machine q35,usb=off \
		-m 256M \
		-serial stdio \
		-cdrom $(ISO) \
		-s -S \
		-no-reboot

# Clippy lint
clippy:
	cargo clippy -p opsys-kernel -- -D warnings

# Clean build artifacts
clean:
	cargo clean
	rm -rf build/
