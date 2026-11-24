SHELL := /usr/bin/env bash
.DEFAULT_GOAL := help
PANEL ?= tianma
SERIAL ?=
JOBS ?= 3
MAX_JOBS ?= 2
# Use the normal daemon store when available; otherwise use a workspace store.
NIX_STORE_ROOT ?= $(if $(wildcard /nix/store),,$(abspath ../nix-store))
ifneq ($(NIX_STORE_ROOT),)
NIX_FLAGS ?= --store "$(NIX_STORE_ROOT)" --option build-users-group ""
else
NIX_FLAGS ?=
endif
# Continue independent packages after a failure; blocked dependents still fail.
NIX_BUILD := nix-build --keep-going $(NIX_FLAGS) --cores $(JOBS) --max-jobs $(MAX_JOBS)
PROJECT_VERSION := $(shell python3 scripts/version.py)
PROJECT_REVISION := $(shell git rev-parse HEAD)
DIST_FLAGS := --panel "$(PANEL)" --version "$(PROJECT_VERSION)" --revision "$(PROJECT_REVISION)"
ARGS := --argstr panel "$(PANEL)"
BOOT_TARGET = $(if $(wildcard result-$(PANEL)/boot.img),result-$(PANEL),result-boot-$(PANEL))
.PHONY: help check verify eval build kernel boot install devices fetch
help:
	@echo 'Targets: fetch check verify test eval kernel boot-image build inspect devices boot install version hooks optimise-store gc-store fetch-uboot uboot uboot-package'
	@echo 'PANEL=tianma|ebbg (confirm panel before flashing); JOBS=threads per package (default 3); MAX_JOBS=parallel packages (default 2); SERIAL=<fastboot serial>'
	@echo 'install ERASE_USERDATA=beryllium overwrites boot and userdata, erasing Android data.'
check:
	@command -v nix-build >/dev/null || { echo 'Install Nix first'; exit 1; }
verify:
	@cd kernel && sha256sum -c SHA256SUMS
	@cd firmware && sha256sum -c SHA256SUMS
	@python3 scripts/verify-firmware.py
eval: check verify
	nix-instantiate $(NIX_FLAGS) . $(ARGS) -A outputs.default >/dev/null
kernel: check verify
	@mkdir -p .build/roots
	$(NIX_BUILD) . $(ARGS) -A config.mobile.boot.stage-1.kernel.package -o .build/roots/result-kernel
	bash scripts/link-result.sh .build/roots/result-kernel result-kernel "$(NIX_STORE_ROOT)"
	python3 scripts/dist.py kernel $(DIST_FLAGS)
build: kernel check verify
	@mkdir -p .build/roots
	$(NIX_BUILD) . $(ARGS) -A outputs.default -o .build/roots/result-$(PANEL)
	bash scripts/link-result.sh .build/roots/result-$(PANEL) result-$(PANEL) "$(NIX_STORE_ROOT)"
	python3 scripts/inspect-images.py "result-$(PANEL)" --panel "$(PANEL)"
	python3 scripts/dist.py system $(DIST_FLAGS)
devices:
	fastboot devices
boot: inspect-boot
	@SERIAL="$(SERIAL)" bash scripts/flash.sh boot "$(BOOT_TARGET)"
install: inspect
	@SERIAL="$(SERIAL)" ERASE_USERDATA="$(ERASE_USERDATA)" bash scripts/flash.sh install "result-$(PANEL)"

fetch:
	bash scripts/fetch.sh

.PHONY: test
test:
	python3 tests/test_supervisor.py
	python3 tests/test_flash.py
	python3 tests/test_version.py
	python3 tests/test_dist.py
	python3 tests/test_release_install.py
	python3 tests/test_usb_ssh.py
	python3 tests/test_uboot.py
	python3 tests/test_phone_debug.py
	bash -n scripts/flash.sh scripts/fetch.sh scripts/link-result.sh

.PHONY: inspect
inspect:
	python3 scripts/inspect-images.py "result-$(PANEL)" --panel "$(PANEL)"

.PHONY: boot-image
boot-image: check verify
	@mkdir -p .build/roots
	$(NIX_BUILD) . $(ARGS) -A outputs.android-bootimg -o .build/roots/result-boot-$(PANEL)
	bash scripts/link-result.sh .build/roots/result-boot-$(PANEL) result-boot-$(PANEL) "$(NIX_STORE_ROOT)"
	python3 scripts/inspect-images.py "result-boot-$(PANEL)" --panel "$(PANEL)" --boot-only
	python3 scripts/dist.py boot $(DIST_FLAGS)

.PHONY: inspect-boot
inspect-boot:
	python3 scripts/inspect-images.py "$(BOOT_TARGET)" --panel "$(PANEL)" --boot-only

.PHONY: version hooks optimise-store gc-store
version:
	python3 scripts/version.py
hooks:
	git config core.hooksPath .githooks
optimise-store:
	nix-store $(NIX_FLAGS) --optimise
gc-store:
	nix-store $(NIX_FLAGS) --gc

# U-Boot is an optional host-controlled boot layer; it does not replace ABL/XBL.
.PHONY: fetch-uboot uboot uboot-package
fetch-uboot:
	python3 scripts/fetch-uboot.py
uboot: fetch-uboot
	@mkdir -p .build/roots
	$(NIX_BUILD) --expr '(import ./. {}).pkgs.callPackage ./bootloader {}' -o .build/roots/result-uboot
	bash scripts/link-result.sh .build/roots/result-uboot result-uboot "$(NIX_STORE_ROOT)"
uboot-package: uboot
	@test "$(PANEL)" = tianma || { echo 'U-Boot currently supports the confirmed Tianma panel only'; exit 1; }
	@test -f result-boot-tianma || { echo 'Build the Linux boot image first'; exit 1; }
	$(NIX_BUILD) --expr '(import ./. {}).pkgs.callPackage ./bootloader/package.nix { linuxBoot = builtins.path { path = ./result-boot-tianma; name = "spectre-linux-boot.img"; }; }' -o .build/roots/result-uboot-package
	bash scripts/link-result.sh .build/roots/result-uboot-package result-uboot-package "$(NIX_STORE_ROOT)"
	python3 scripts/dist-uboot.py

.PHONY: fetch-phosh phone-debug
fetch-phosh:
	python3 scripts/fetch-phosh.py
phone-debug:
	@mkdir -p .build/roots
	$(NIX_BUILD) --expr 'import ./debug { pkgs = (import ./. {}).pkgs; }' -o .build/roots/phone-debug

.PHONY: camera
camera: check
	@mkdir -p .build/roots
	$(NIX_BUILD) --expr 'import ./modules/camera/live.nix {}' -o .build/roots/result-camera
	bash scripts/link-result.sh .build/roots/result-camera result-camera "$(NIX_STORE_ROOT)"

.PHONY: qt-camera qt-camera-test camera-eeprom
camera-eeprom: check
	@mkdir -p .build/roots
	$(NIX_BUILD) modules/camera/read-eeprom.nix -o .build/roots/result-camera-eeprom
	bash scripts/link-result.sh .build/roots/result-camera-eeprom result-camera-eeprom "$(NIX_STORE_ROOT)"
qt-camera: check
	@mkdir -p .build/roots
	$(NIX_BUILD) --expr 'import ./apps/spectre-camera/live.nix {}' -o .build/roots/result-qt-camera
	bash scripts/link-result.sh .build/roots/result-qt-camera result-qt-camera "$(NIX_STORE_ROOT)"
qt-camera-test: check
	@mkdir -p .build/roots
	$(NIX_BUILD) --expr 'import ./apps/spectre-camera/live.nix { native = true; }' -o .build/roots/result-qt-camera-test
	bash scripts/link-result.sh .build/roots/result-qt-camera-test result-qt-camera-test "$(NIX_STORE_ROOT)"
	NIX_STORE_ROOT="$(NIX_STORE_ROOT)" bash scripts/test-qt-camera.sh
