#!/usr/bin/env python3
"""Gzip a web asset for embedding in the firmware: gzip_asset.py <in> <out>.

The output is reproducible (no timestamp or file name in the header), so unchanged
assets give byte-identical firmware images.
"""

import gzip
import sys


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    src, dst = sys.argv[1], sys.argv[2]
    with open(src, "rb") as f:
        data = f.read()
    with open(dst, "wb") as raw, gzip.GzipFile(filename="", mode="wb", fileobj=raw,
                                               compresslevel=9, mtime=0) as gz:
        gz.write(data)


if __name__ == "__main__":
    main()
