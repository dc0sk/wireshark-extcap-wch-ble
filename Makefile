.PHONY: all release clean install uninstall

all:
	cargo build

release:
	cargo build --release

clean:
	cargo clean

# Install to Wireshark's system extcap directory
EXTCAP_DIR ?= /usr/lib/wireshark/extcap

install: release
	install -Dm755 target/release/wch-ble-extcap $(DESTDIR)$(EXTCAP_DIR)/wch-ble-extcap
	@echo "Installed to $(DESTDIR)$(EXTCAP_DIR)/wch-ble-extcap"
	@echo "Restart Wireshark to see the WCH BLE Analyzer Pro in the interface list."

uninstall:
	rm -f $(DESTDIR)$(EXTCAP_DIR)/wch-ble-extcap
