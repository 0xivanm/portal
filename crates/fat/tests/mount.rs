mod common;

use common::{Image, LABEL, VOLUME_ID, fat32_size, fatfs_free, format, partition_table};
use fat::{Cache, Error, Volume};
use fatfs::FatType;

#[test]
fn superfloppy() {
    let size = fat32_size(512);
    let mut image = vec![0; size];
    format(&mut image, 0, size, FatType::Fat32, 512);
    let free = fatfs_free(&mut image, 0, size);

    let mut cache = Cache::new();
    let volume = Volume::mount(Image(image), &mut cache).unwrap();
    assert_eq!(volume.start(), 0);
    assert_eq!(volume.cluster_size(), 512);
    assert!(volume.clusters() >= 65_525);
    assert_eq!(volume.free_clusters(), Some(free));
    assert_eq!(*volume.label(), LABEL);
    assert_eq!(volume.volume_id(), VOLUME_ID);
}

// like the iPod: a firmware partition, then FAT32 with 2048-byte sectors, and the
// partition table counting in 2048-byte units
#[test]
fn ipod_layout() {
    let size = fat32_size(2048);
    let (firmware, fat_start) = (63u32, 1000u32);
    let offset = fat_start as usize * 2048;
    let mut image = vec![0; offset + size];
    partition_table(&mut image, &[(0x00, firmware, fat_start - firmware), (0x0B, fat_start, (size / 2048) as u32)]);
    format(&mut image, offset, size, FatType::Fat32, 2048);
    let free = fatfs_free(&mut image, offset, size);

    let mut cache = Cache::new();
    let volume = Volume::mount(Image(image), &mut cache).unwrap();
    assert_eq!(volume.start(), u64::from(fat_start) * 4);
    assert_eq!(volume.cluster_size(), 2048);
    assert_eq!(volume.free_clusters(), Some(free));
    assert_eq!(*volume.label(), LABEL);
    assert_eq!(volume.volume_id(), VOLUME_ID);
}

#[test]
fn partition_in_512_byte_units() {
    let size = fat32_size(512);
    let fat_start = 2048u32;
    let offset = fat_start as usize * 512;
    let mut image = vec![0; offset + size];
    partition_table(&mut image, &[(0x0C, fat_start, (size / 512) as u32)]);
    format(&mut image, offset, size, FatType::Fat32, 512);

    let mut cache = Cache::new();
    let volume = Volume::mount(Image(image), &mut cache).unwrap();
    assert_eq!(volume.start(), u64::from(fat_start));
    assert_eq!(volume.cluster_size(), 512);
}

#[test]
fn fat16_is_not_fat32() {
    let size = 32 * 1024 * 1024;
    let mut image = vec![0; size];
    format(&mut image, 0, size, FatType::Fat16, 512);
    assert!(matches!(Volume::mount(Image(image), &mut Cache::new()), Err(Error::NotFat32)));
}

// a partition that points past the end of the disk is skipped, not a read error
#[test]
fn partition_past_the_end() {
    let mut image = vec![0; 64 * 512];
    partition_table(&mut image, &[(0x0B, 1_000_000, 100)]);
    assert!(matches!(Volume::mount(Image(image), &mut Cache::new()), Err(Error::NoVolume)));
}
