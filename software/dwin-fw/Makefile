# display build targets

PY      := .venv/bin/python
UI      := firmware/companion-ui
NODE    := firmware/companion-node
SENSORS := firmware/sensors
SENSOR_NODE := firmware/sensor-node
SENSOR_OUT  := handoff/sensor-node
ESPFLASH ?= espflash
# external target directory
export CARGO_TARGET_DIR ?= $(HOME)/.cache/intellithings-display
# sanitize build paths
SENSOR_STAGE = $(CARGO_TARGET_DIR)/sensor-node-stage
SENSOR_RUSTFLAGS = -C force-frame-pointers --remap-path-prefix=$(SENSOR_STAGE)/sensor-node/source=source --remap-path-prefix=$(or $(CARGO_HOME),$(HOME)/.cargo)=cargo --remap-path-prefix=$(or $(RUSTUP_HOME),$(HOME)/.rustup)=rustup

.PHONY: setup dwin sim test flash firmware sensors sd play clean

setup:
	/opt/homebrew/bin/python3 -m venv .venv || python3 -m venv .venv
	$(PY) -m pip install -q -r requirements.txt

dwin:
	$(PY) -m dgus.build --check
	$(PY) -m dgus.vpmap

sim: dwin
	cd $(UI) && cargo run -q --example scenarios -- ../../build/vp
	$(PY) -m dgus.sim --all --compare

test: dwin
	$(PY) -m pytest -q
	cd $(UI) && cargo test -q
	cd $(NODE) && cargo build --release
	cd $(SENSORS) && cargo test -q
	cd $(SENSOR_NODE) && cargo build --release

flash: dwin
	cd $(NODE) && cargo run --release

firmware:
	cd $(NODE) && cargo build --release
	mkdir -p build
	cp $(CARGO_TARGET_DIR)/riscv32imac-unknown-none-elf/release/companion-node build/companion-node.elf

sensors:
	cd $(SENSORS) && cargo test -q
	rm -rf "$(SENSOR_STAGE)" && mkdir -p "$(SENSOR_STAGE)/sensor-node/source"
	rsync -a --exclude target --exclude .DS_Store $(SENSORS) $(SENSOR_NODE) "$(SENSOR_STAGE)/sensor-node/source/"
	cd "$(SENSOR_STAGE)/sensor-node/source/sensor-node" && RUSTFLAGS="$(SENSOR_RUSTFLAGS)" cargo build --release
	cp "$(CARGO_TARGET_DIR)/riscv32imac-unknown-none-elf/release/sensor-node" $(SENSOR_OUT)/sensor-node.elf
	$(ESPFLASH) save-image --chip esp32c6 --merge --skip-padding --flash-size 4mb $(SENSOR_OUT)/sensor-node.elf $(SENSOR_OUT)/sensor-node.bin
	cp $(SENSORS)/licenses.md $(SENSOR_OUT)/
	rsync -a --exclude .DS_Store $(SENSOR_OUT)/ "$(SENSOR_STAGE)/sensor-node/"
	rm -f handoff/sensor-node.zip
	cd "$(SENSOR_STAGE)" && zip -qrX "$(CURDIR)/handoff/sensor-node.zip" sensor-node

sd:
	tools/sdcard.sh $(DISK)

play:
	$(PY) tools/companion.py --port $(PORT) play

clean:
	rm -rf build
