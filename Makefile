PACKAGE_NAME := fwa
VERSION := 1.0
ARCH := armhf
BUILD_DIR := build
DEB_DIR := $(BUILD_DIR)/$(PACKAGE_NAME)-$(VERSION)
BIN_DIR := $(DEB_DIR)/usr/local/bin
SERVICE_DIR := $(DEB_DIR)/lib/systemd/system
CONFIG_DIR := $(DEB_DIR)/etc
PM3_DIR := $(DEB_DIR)/usr/share/pm3

.PHONY: all build clean package prepare_package binary

all: build

$(BUILD_DIR)/fwa: 
	cross build --release --target arm-unknown-linux-gnueabihf
	cp ./target/arm-unknown-linux-gnueabihf/release/fw-anwesenheit $@

prepare_package: $(DEB_DIR)/DEBIAN $(BIN_DIR)/fwa
	mkdir -p $(SERVICE_DIR)
	cp ./service/fwa.service $(SERVICE_DIR)/
	cp ./service/fwa-fail.service $(SERVICE_DIR)/

	mkdir -p $(CONFIG_DIR)
	cp ./service/fwa.env $(CONFIG_DIR)/

	mkdir -p $(PM3_DIR)
	cp -r ./pre-compiled/* $(PM3_DIR)/

	mkdir -p $(DEB_DIR)/var/lib/fwa/

$(BIN_DIR)/fwa: $(BUILD_DIR)/fwa
	mkdir -p $(BIN_DIR)
	cp $< $@

$(DEB_DIR)/DEBIAN:
	mkdir -p $(DEB_DIR)/DEBIAN
	echo "Package: $(PACKAGE_NAME)" > $(DEB_DIR)/DEBIAN/control
	echo "Version: $(VERSION)" >> $(DEB_DIR)/DEBIAN/control
	echo "Section: utils" >> $(DEB_DIR)/DEBIAN/control
	echo "Priority: optional" >> $(DEB_DIR)/DEBIAN/control
	echo "Architecture: $(ARCH)" >> $(DEB_DIR)/DEBIAN/control
	echo "Depends: libc6 (>= 2.28)" >> $(DEB_DIR)/DEBIAN/control
	echo "Maintainer: Niklas Kapelle <niklas@kapelle.org>" >> $(DEB_DIR)/DEBIAN/control
	echo "Description: Feuerwehr anwesenheit" >> $(DEB_DIR)/DEBIAN/control

package: prepare_package
	dpkg-deb --build $(DEB_DIR)

clean:
	cargo clean
	rm -rf $(BUILD_DIR)
