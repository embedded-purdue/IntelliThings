# Emulator v0.1.0 (frozen)

The emulator firmware that ran on the board from Sep 27 until v2.1.0. Keep it so you can
**fall back** if a newer version misbehaves. Don't edit anything in this folder; new work happens
in [`../../esp32_c3_emulator/`](../../esp32_c3_emulator/).

| | |
|---|---|
| Git tag | `emulator-v0.1.0` (commit `967fe5e`) |
| Source | [`esp32_c3_emulator/`](esp32_c3_emulator/): the ESP-IDF project at that tag, plus `dependencies.lock` so it rebuilds with the same MQTT, cJSON and mDNS component versions |
| User guide | [`USER_GUIDE.md`](USER_GUIDE.md): the v0.1.0 README (control panel, commands, topics, troubleshooting) |
| Test log | [`TODO.md`](TODO.md): the Sep 27 network tests and known limitations |
| Prebuilt image | `emulator-v0.1.0-merged.bin` (1 188 176 bytes) in the **team Google Drive folder**. **Not in git**: it has the team Wi-Fi and broker passwords compiled in. SHA-256: `b2ddb41407a15483878afef8aca810cdf2b067f0e0db124220357ddc9827c4c8` |
| Verified | Oct 18: rebuilt from this folder (0 warnings, app 1.12 MB, 27 % of the partition free), flashed as the merged image over USB, booted and ran the simulation. The boot log's `App version` reads `emulator-v0.1.0-36-g…` (taken from `git describe`); the panel shows `0.1.0` |
| Toolchain | ESP-IDF v6.1, target `esp32c3` (LuatOS ESP32-C3 Core, 4 MB flash) |

## What v0.1.0 does

Three emulated nodes (`emu-kitchen`, `emu-bedroom`, `emu-living-room`) publish full sensor
snapshots every 3 s on the real node topics. They follow the node contract in
[`docs/interfaces/mqtt.md`](../../../../docs/interfaces/mqtt.md). A web control panel at
`http://192.168.1.4/` offers 12 one-click scenarios and custom values. The board receives AI
messages on each node's `…/display` topic. Control topics live under `intellithings/emulator/#`.
The broker address is compiled in; there are no runtime settings and no demo mode.

## Falling back

### A. Flash the prebuilt image (no ESP-IDF needed)

The merged image contains the bootloader, partition table and app, so a single write restores
everything. That includes the partition table, if a later version changed it.

1. Download `emulator-v0.1.0-merged.bin` from the team Drive folder.
2. Check that its SHA-256 matches the one stored next to it:
   ```bash
   sha256sum emulator-v0.1.0-merged.bin
   ```
   On Windows PowerShell: `Get-FileHash emulator-v0.1.0-merged.bin`.
3. Install esptool once (`pip install esptool`), plug in the board, then flash it. Use your
   port, e.g. `COM8`, `/dev/ttyUSB0` or `/dev/cu.usbmodem…`:
   ```bash
   python -m esptool --chip esp32c3 -p COM8 -b 460800 write_flash 0x0 emulator-v0.1.0-merged.bin
   ```
4. Unplug and replug the board, or press RST. After ~20 s the panel at `http://192.168.1.4/`
   shows **Firmware 0.1.0** under System status.

The image only knows the Wi-Fi network and broker it was built with. On a different network,
rebuild it with option B.

### B. Rebuild from this folder

```bash
cd ai/emulator/releases/v0.1.0/esp32_c3_emulator
cp ../../../esp32_c3_emulator/sdkconfig.local .   # or: cp sdkconfig.local.example sdkconfig.local, then edit
. $HOME/esp/esp-idf/export.sh                      # Windows: the ESP-IDF v6.1 PowerShell profile
idf.py set-target esp32c3
idf.py build
idf.py -p COM8 flash monitor
```

`sdkconfig.local` holds the Wi-Fi and broker settings. It's gitignored here too, so never commit
it.

### C. From the git tag

```bash
git worktree add ../emulator-v0.1.0 emulator-v0.1.0
```

Then build `../emulator-v0.1.0/ai/emulator/esp32_c3_emulator` as in option B. The tag has no
`dependencies.lock`, so the component manager may fetch newer component versions. To pin them,
copy the lock file from this folder first.

## Making the prebuilt image (maintainer)

After a build (option B):

```bash
idf.py merge-bin -o emulator-v0.1.0-merged.bin    # written to build/
sha256sum build/emulator-v0.1.0-merged.bin
```

Upload the `.bin` and a `.sha256` text file with the hash to the team Drive folder. Never commit
them.

## Settings left behind by newer versions

v2.1.0 stores its runtime settings (broker address and so on) in the board's NVS, under its own
namespace. v0.1.0 never reads them and uses its compiled-in values. Going back to v2.1.0 later
picks the saved settings up again.
