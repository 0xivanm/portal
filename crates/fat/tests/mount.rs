use std::io::Cursor;

use fat::{BlockDevice, Error, SECTOR_SIZE, SectorBuffer, Volume};
use fatfs::{FatType, FormatVolumeOptions};

// a disk image in memory
struct Image(Vec<u8>);

impl BlockDevice for Image {
    type Error = ();

    fn sectors(&self) -> u64 {
        (self.0.len() / SECTOR_SIZE) as u64
    }

    fn read(&mut self, lba: u64, buffers: &mut [SectorBuffer]) -> Result<(), ()> {
        for (index, buffer) in buffers.iter_mut().enumerate() {
            let at = (lba as usize + index) * SECTOR_SIZE;
            buffer.0.copy_from_slice(self.0.get(at..at + SECTOR_SIZE).ok_or(())?);
        }
        Ok(())
    }
}

const LABEL: [u8; 11] = *b"PORTAL TEST";
const VOLUME_ID: u32 = 0x1234_5678;

// formats bytes [offset, offset + size) of the image with fatfs, the reference implementation
fn format(image: &mut [u8], offset: usize, size: usize, fat_type: FatType, bytes_per_sector: u16) {
    let options = FormatVolumeOptions::new()
        .fat_type(fat_type)
        .bytes_per_sector(bytes_per_sector)
        .bytes_per_cluster(u32::from(bytes_per_sector))
        .volume_label(LABEL)
        .volume_id(VOLUME_ID);
    fatfs::format_volume(&mut Cursor::new(&mut image[offset..offset + size]), options).unwrap();
}

// what fatfs itself counts as free on the volume at offset
fn fatfs_free(image: &mut [u8], offset: usize, size: usize) -> u32 {
    let fs = fatfs::FileSystem::new(Cursor::new(&mut image[offset..offset + size]), fatfs::FsOptions::new()).unwrap();
    fs.stats().unwrap().free_clusters()
}

// an MBR with (type, start, size) entries, in whatever unit the caller uses
fn partition_table(image: &mut [u8], entries: &[(u8, u32, u32)]) {
    for (index, &(kind, start, size)) in entries.iter().enumerate() {
        let entry = 446 + index * 16;
        image[entry + 4] = kind;
        image[entry + 8..entry + 12].copy_from_slice(&start.to_le_bytes());
        image[entry + 12..entry + 16].copy_from_slice(&size.to_le_bytes());
    }
    image[510] = 0x55;
    image[511] = 0xAA;
}

// big enough for FAT32's minimum of 65,525 clusters with one sector per cluster
fn fat32_size(bytes_per_sector: usize) -> usize {
    66_000 * bytes_per_sector + 2 * 1024 * 1024
}

#[test]
fn superfloppy() {
    let size = fat32_size(512);
    let mut image = vec![0; size];
    format(&mut image, 0, size, FatType::Fat32, 512);
    let free = fatfs_free(&mut image, 0, size);

    let volume = Volume::mount(Image(image)).unwrap();
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

    let volume = Volume::mount(Image(image)).unwrap();
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

    let volume = Volume::mount(Image(image)).unwrap();
    assert_eq!(volume.start(), u64::from(fat_start));
    assert_eq!(volume.cluster_size(), 512);
}

#[test]
fn fat16_is_not_fat32() {
    let size = 32 * 1024 * 1024;
    let mut image = vec![0; size];
    format(&mut image, 0, size, FatType::Fat16, 512);
    assert!(matches!(Volume::mount(Image(image)), Err(Error::NotFat32)));
}

// a partition that points past the end of the disk is skipped, not a read error
#[test]
fn partition_past_the_end() {
    let mut image = vec![0; 64 * 512];
    partition_table(&mut image, &[(0x0B, 1_000_000, 100)]);
    assert!(matches!(Volume::mount(Image(image)), Err(Error::NoVolume)));
}
