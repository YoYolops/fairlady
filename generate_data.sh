#!/bin/bash

set -e

# Define configuration and fallbacks
# Supports floats (e.g., 1.5, 0.5, 2)
SIZE_GB="${FILE_SIZE_GB:-1}"
OUTPUT_DIR="./data"
OUTPUT_FILE="$OUTPUT_DIR/file_${SIZE_GB}gb.not_txt"
BLOCK_SIZE_MB=4

if [ ! -d "$OUTPUT_DIR" ]; then
    mkdir -p "$OUTPUT_DIR"
fi


# Use awk to handle floating-point math safely
# Math: (Size_GB * 1024) / 4. 
# 'int()' to round down to a whole number for dd's count parameter.
COUNT=$(awk "BEGIN { print int(($SIZE_GB * 1024) / $BLOCK_SIZE_MB) }")

# Catch cases where someone types a number so small that count becomes 0
if [ "$COUNT" -le 0 ]; then
    echo "ERROR: Calculated block count is 0. Your FILE_SIZE_GB is too small for a ${BLOCK_SIZE_MB}M block size." >&2
    exit 1
fi

echo "Executing: dd if=/dev/zero of=$OUTPUT_FILE bs=${BLOCK_SIZE_MB}M count=$COUNT"
echo "Generating zero-byte payload stream..."

# Trigger the block copy operation
dd if=/dev/zero of="$OUTPUT_FILE" bs="${BLOCK_SIZE_MB}M" count="$COUNT" status=progress

echo "Done! File generated successfully at: $OUTPUT_FILE"