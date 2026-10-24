#!/usr/bin/env bash
set -e

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
elf=target/armv4t-none-eabi/release/ipod
firmware=target/armv4t-none-eabi/release/linux.bin

cargo build --release
arm-none-eabi-objcopy -O binary "$elf" "$firmware"

for mount in "/run/media/$USER"/* "/media/$USER"/* /mnt/*; do
    if [ -d "$mount/iPod_Control" ]; then
        cp "$firmware" "$mount/linux.bin"
        echo "Copied linux.bin to $mount/linux.bin"
        exit 0
    fi
done

echo "iPod is not mounted; firmware remains at $firmware" >&2
exit 1
