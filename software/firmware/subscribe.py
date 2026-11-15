#!/usr/bin/env python3
"""Subscribe to sensor messages using the firmware's MQTT credentials."""

import os
from pathlib import Path
import shutil
import sys
import tomllib


def main():
    config_path = Path(__file__).resolve().parent / ".cargo" / "esp-config.toml"
    try:
        config = tomllib.loads(config_path.read_text())["env"]
        credentials = []
        for name in ("MQTT_USER", "MQTT_PASSWORD"):
            setting = config[name]
            value = setting["value"] if isinstance(setting, dict) else setting
            force = isinstance(setting, dict) and setting.get("force", False)
            credentials.append(value if force else os.environ.get(name, value))
    except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
        sys.exit(f"Could not read MQTT credentials from {config_path}: {error}")

    executable = shutil.which("mosquitto_sub")
    if executable is None:
        sys.exit("mosquitto_sub is required. Install it with sudo dnf install mosquitto")

    print("Listening using mosquitto_sub")
    os.execv(executable, [
        executable,
        "-h", "192.168.1.2", "-p", "1883",
        "-u", credentials[0], "-P", credentials[1],
        "-t", "intellithings/+/sensors", "-v",
        *sys.argv[1:],
    ])


if __name__ == "__main__":
    main()
