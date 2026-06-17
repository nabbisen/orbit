#!/bin/sh
# Generate SHA-256 checksums for release archives (RFC-017 §16 test 8).
set -e
SUMS_FILE="SHA256SUMS"
for f in "$@"; do
    sha256sum "$f" >> "$SUMS_FILE"
    echo "  $(basename "$f"): checksummed"
done
echo "Written to $SUMS_FILE"
