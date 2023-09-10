# Configuration options
FEATURES ?=
MEMORY   ?= 2G
PROFILE  ?= release

# Command arguments
override CARGO_ARGS = --bin monoos --no-default-features
override QEMU_ARGS  = -no-reboot -no-shutdown -M q35 -serial stdio -m $(MEMORY) -cdrom $(ISO) -bios $(BUILD_ROOT)/RELEASEX64_OVMF.fd -boot d

# Checks
ifneq ($(PROFILE),$(filter $(PROFILE),debug release))
$(error Error: unsupported cargo profile "$(PROFILE)". supported options are "debug" and "release")
endif

ifneq ($(FEATURES),)
	override CARGO_ARGS += --features $(FEATURES)
endif

ifeq ($(PROFILE), release)
	override CARGO_ARGS += --release
endif

ifeq ($(PROFILE),debug)
	override QEMU_ARGS += -s -S
endif

# Environment variables
export MONOOS_VERSION = v0.1.0
export MONOOS_MEMORY = $(MEMORY)

# Overrides
override BUILD_DIR  := build
override BUILD_ROOT := $(BUILD_DIR)/root
override ISO        := $(BUILD_ROOT)/monoos.iso
override LIMINE_DIR := $(BUILD_DIR)/limine
override ESP        := $(BUILD_ROOT)/EFI/BOOT
override KERNEL_BIN := $(BUILD_ROOT)/monoos.elf

# Targets
.PHONY: all iso run test miri clean
all: iso

gen_build_dirs:
	@# recursively create from the deepest directory
	@mkdir -p $(ESP) 

limine:
	@git clone --depth 1 --branch v5.x-branch-binary https://github.com/limine-bootloader/limine.git $(LIMINE_DIR)
	@$(MAKE) -C $(LIMINE_DIR)

ovmf:
	@wget https://retrage.github.io/edk2-nightly/bin/RELEASEX64_OVMF.fd -P $(BUILD_ROOT)
	
kernel_build:
	@cargo build $(CARGO_ARGS)
	
kernel_test:
	@cargo rustc $(CARGO_ARGS) -- --test

$(ISO): gen_build_dirs limine ovmf 	
	@cp target/x86_64-unknown-none/$(PROFILE)/monoos $(KERNEL_BIN)
	@cp limine.cfg $(LIMINE_DIR)/limine-uefi-cd.bin $(BUILD_ROOT)
	@cp $(LIMINE_DIR)/BOOTX64.EFI $(ESP)
	@xorriso -as mkisofs --efi-boot limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label $(BUILD_ROOT) -o $(ISO)

# Convenience target for $(ISO)
iso: $(ISO)

run: kernel_build iso
	@qemu-system-x86_64 $(QEMU_ARGS)	

test: kernel_test iso
	@qemu-system-x86_64 $(QEMU_ARGS) -display none	

miri:
	@MIRI_NO_STD=1 cargo miri run --target x86_64-unknown-none

clean:
	@cargo clean
	@rm -rf build