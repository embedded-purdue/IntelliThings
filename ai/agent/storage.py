"""Timestamped, atomic runtime-file storage shared by pipeline stages."""

import json
import os
import tempfile
from datetime import datetime, timezone


def timestamp():
    return datetime.now(timezone.utc).isoformat()


def write_json(path, value):
    """Publish a complete file atomically, with owner-only permissions."""
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    name = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as f:
            name = f.name
            json.dump(value, f, indent=2, ensure_ascii=False)
            f.write("\n")
            f.flush()
            os.fsync(f.fileno())
        os.replace(name, path)
    finally:
        if name and os.path.exists(name):
            os.unlink(name)
