#!/usr/bin/env bash
set -e

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
elf=target/armv4t-none-eabi/release/portal
firmware=target/armv4t-none-eabi/release/linux.bin

# from firmware/, so its .cargo/config.toml (ARM target, build-std) applies
(cd firmware && cargo build --release)
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
