ROOT_DIR := $(CURDIR)
APP_DIR := $(ROOT_DIR)/apps/flutter_client
RUST_CRATE := syncer-ffi
RUST_TARGET_DIR := $(ROOT_DIR)/target
WINDOWS_RELEASE_DIR := $(APP_DIR)/build/windows/x64/runner/Release
WINDOWS_DLL := $(RUST_TARGET_DIR)/release/syncer_ffi.dll
CARGO ?= cargo
UNAME_S := $(shell uname -s 2>/dev/null || echo Windows_NT)
ifeq ($(findstring Darwin,$(UNAME_S)),Darwin)
RUN_DEVICE := macos
else
RUN_DEVICE := windows
endif

ifneq ("$(wildcard $(ROOT_DIR)/.puro/bin/puro.bat)","")
FLUTTER_CMD ?= $(ROOT_DIR)/.puro/bin/puro.bat flutter
else
FLUTTER_CMD ?= flutter
endif

.PHONY: help all windows run rust-ffi rust-ffi-debug flutter-pub windows-build windows-package android macos ios clean

help:
	@echo "make windows      -> 构建 Windows 应用并打入 syncer_ffi.dll"
	@echo "make run          -> 本地启动桌面应用（Windows/macOS）"
	@echo "make android      -> 构建 Android APK"
	@echo "make macos        -> 构建 macOS 应用"
	@echo "make ios          -> 构建 iOS 应用"
	@echo "make clean        -> 清理 Flutter 构建目录"

all: windows

windows: rust-ffi flutter-pub windows-build windows-package

run: rust-ffi-debug flutter-pub
	cd "$(APP_DIR)" && $(FLUTTER_CMD) run -d $(RUN_DEVICE)

rust-ffi:
	cd "$(ROOT_DIR)" && $(CARGO) build -p $(RUST_CRATE) --release

rust-ffi-debug:
	cd "$(ROOT_DIR)" && $(CARGO) build -p $(RUST_CRATE)

flutter-pub:
	cd "$(APP_DIR)" && $(FLUTTER_CMD) pub get

windows-build:
	cd "$(APP_DIR)" && $(FLUTTER_CMD) build windows --release

windows-package:
	powershell -NoProfile -Command "$$src='$(WINDOWS_DLL)'; $$dst='$(WINDOWS_RELEASE_DIR)'; if (!(Test-Path $$src)) { throw 'missing ffi dll: ' + $$src }; New-Item -ItemType Directory -Force -Path $$dst | Out-Null; Copy-Item -Path $$src -Destination $$dst -Force"

android: flutter-pub
	cd "$(APP_DIR)" && $(FLUTTER_CMD) build apk --release

macos: flutter-pub
	cd "$(APP_DIR)" && $(FLUTTER_CMD) build macos --release

ios: flutter-pub
	cd "$(APP_DIR)" && $(FLUTTER_CMD) build ios --release --no-codesign

clean:
	cd "$(APP_DIR)" && $(FLUTTER_CMD) clean
